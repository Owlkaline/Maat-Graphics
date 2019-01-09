use vk;
use winit;
use image;
use cgmath::{Vector2, Vector3, Vector4, Matrix4, ortho, SquareMatrix};
use winit::dpi::LogicalSize;

use crate::math;
use crate::ResourceManager;
use crate::camera::Camera;
use crate::drawcalls::DrawCall; 
use crate::drawcalls::DrawType;
use crate::graphics::CoreRender;
use crate::font::GenericFont;
use crate::TextureShader;
use crate::graphics;

use crate::vulkan::vkenums::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout, ImageUsage, ImageType, ImageViewType, ImageTiling, Sample, Filter, AddressMode, MipmapMode, VkBool, ShaderStageFlagBits};

use crate::vulkan::VkWindow;
use crate::vulkan::Shader;
use crate::vulkan::pool::CommandPool;
use crate::vulkan::Instance;
use crate::vulkan::Device;
use crate::vulkan::pool::DescriptorPool;
use crate::vulkan::DescriptorSet;
use crate::vulkan::UpdateDescriptorSets;
use crate::vulkan::DescriptorSetBuilder;
use crate::vulkan::Pipeline;
use crate::vulkan::PipelineBuilder;
use crate::vulkan::RenderPass;
use crate::vulkan::RenderPassBuilder;
use crate::vulkan::AttachmentInfo;
use crate::vulkan::SubpassInfo;
use crate::vulkan::Image;
use crate::vulkan::Sampler;
use crate::vulkan::SamplerBuilder;
use crate::vulkan::sync::Fence;
use crate::vulkan::sync::Semaphore;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::buffer::BufferUsage;
use crate::vulkan::buffer::Framebuffer;
use crate::vulkan::buffer::UniformData;
use crate::vulkan::buffer::CommandBuffer;
use crate::vulkan::buffer::UniformBufferBuilder;
use crate::vulkan::buffer::CommandBufferBuilder;
use crate::vulkan::check_errors;

use std::ptr;
use std::mem;
use std::sync::Arc;
use std::collections::HashMap;

pub struct CoreMaat {
  window: VkWindow,
  window_dimensions: vk::Extent2D,
  recreate_swapchain: bool,
  fences: Vec<Fence>,
  semaphore_image_available: Semaphore,
  semaphore_render_finished: Semaphore,
  command_pool: CommandPool,
  command_buffers: Vec<Arc<CommandBuffer>>,
  descriptor_set_pool: DescriptorPool,
  
  clear_colour: Vector4<f32>,
  
  texture: Image,
  sampler: Sampler,
  texture_shader: TextureShader,
  
  resources: ResourceManager,
}

impl CoreMaat {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> CoreMaat {
    let window = VkWindow::new(app_name, app_version, width, height, should_debug);
    
    let mut resource_manager = ResourceManager::new();
    
    let fences: Vec<Fence>;
    let semaphore_image_available: Semaphore;
    let semaphore_render_finished: Semaphore;
    let command_pool: CommandPool;
    let command_buffers: Vec<Arc<CommandBuffer>>;
    let descriptor_set_pool: DescriptorPool;
    
    let texture_shader: TextureShader;
    
    let texture_image: Image;
    let sampler: Sampler;
    
    let current_extent = window.get_current_extent();
    
    {
      let instance = window.instance();
      let device = window.device();
      let format = window.swapchain_format();
      let graphics_family = window.get_graphics_family();
      let graphics_queue = window.get_graphics_queue();
      let image_views = window.swapchain_image_views();
      
      semaphore_image_available = Semaphore::new(Arc::clone(&device));
      semaphore_render_finished = Semaphore::new(Arc::clone(&device));
      
      fences = CoreMaat::create_fences(Arc::clone(&device), image_views.len() as u32);
      command_pool = CommandPool::new(Arc::clone(&device), graphics_family);
      command_buffers = command_pool.create_command_buffers(Arc::clone(&device), image_views.len() as u32);
      
      descriptor_set_pool = DescriptorPool::new(Arc::clone(&device), image_views.len() as u32, 40, 40);
      
      texture_image = Image::device_local(Arc::clone(&instance), Arc::clone(&device), "./resources/Textures/statue.png".to_string(), ImageType::Type2D, ImageViewType::Type2D, &vk::FORMAT_R8G8B8A8_UNORM, Sample::Count1Bit, ImageTiling::Optimal, &command_pool, graphics_queue);
      
      sampler = SamplerBuilder::new()
                       .min_filter(Filter::Linear)
                       .mag_filter(Filter::Linear)
                       .address_mode(AddressMode::ClampToEdge)
                       .mipmap_mode(MipmapMode::Nearest)
                       .anisotropy(VkBool::True)
                       .max_anisotropy(8.0)
                       .build(Arc::clone(&device));
      
      texture_shader = TextureShader::new(Arc::clone(&instance), Arc::clone(&device), &current_extent, &format, &sampler, image_views, &texture_image, &descriptor_set_pool, &command_pool, graphics_queue);
      
    }
    
    CoreMaat {
      window: window,
      window_dimensions: current_extent,
      recreate_swapchain: false,
      fences: fences,
      semaphore_image_available: semaphore_image_available,
      semaphore_render_finished: semaphore_render_finished,
      command_pool: command_pool,
      command_buffers: command_buffers,
      descriptor_set_pool: descriptor_set_pool,
      
      clear_colour: Vector4::new(0.0, 0.0, 0.2, 1.0),
      
      texture: texture_image,
      sampler: sampler,
      
      texture_shader,
      resources: resource_manager,
    }
  }
  
  pub fn begin_single_time_command(device: Arc<Device>, command_pool: &CommandPool) -> CommandBuffer {
    let command_buffer = CommandBuffer::primary(Arc::clone(&device), command_pool);
    command_buffer.begin_command_buffer(Arc::clone(&device), vk::COMMAND_BUFFER_LEVEL_PRIMARY);
    command_buffer
  }
  
  pub fn end_single_time_command(device: Arc<Device>, command_buffer: CommandBuffer, command_pool: &CommandPool, graphics_queue: &vk::Queue) {
    let submit_info = {
      vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 0,
        pWaitSemaphores: ptr::null(),
        pWaitDstStageMask: ptr::null(),
        commandBufferCount: 1,
        pCommandBuffers: command_buffer.internal_object(),
        signalSemaphoreCount: 0,
        pSignalSemaphores: ptr::null(),
      }
    };
    
    command_buffer.end_command_buffer(Arc::clone(&device));
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      let command_pool = command_pool.local_command_pool();
      vk.QueueSubmit(*graphics_queue, 1, &submit_info, 0);
      vk.QueueWaitIdle(*graphics_queue);
      vk.FreeCommandBuffers(*device, *command_pool, 1, command_buffer.internal_object());
    }
  }
  
  fn create_fences(device: Arc<Device>, num_fences: u32) -> Vec<Fence> {
    let mut fences: Vec<Fence> = Vec::with_capacity(num_fences as usize);
    
    for _ in 0..num_fences {
      let fence: Fence = Fence::new(Arc::clone(&device));
      fences.push(fence);
    }
    
    fences
  }
}

impl CoreRender for CoreMaat {
  fn preload_model(&mut self, reference: String, location: String) {
    
  }
  
  fn add_model(&mut self, reference: String, location: String) {
    
  }
  
  fn preload_texture(&mut self, reference: String, location: String) {
    let graphics_queue = self.window.get_graphics_queue();
    let device = self.window.device();
    let instance = self.window.instance();
    self.resources.sync_load_texture(reference.to_string(), location, Arc::clone(&device), Arc::clone(&instance), &self.command_pool, *graphics_queue);
    self.texture_shader.add_texture(Arc::clone(&instance), Arc::clone(&device), &self.descriptor_set_pool, reference.to_string(), &self.resources.get_texture(reference).unwrap(), &self.sampler, &self.window_dimensions);
  }
  
  fn add_texture(&mut self, reference: String, location: String) {
    let graphics_queue = self.window.get_graphics_queue();
    let device = self.window.device();
    let instance = self.window.instance();
    self.resources.sync_load_texture(reference.to_string(), location, Arc::clone(&device), Arc::clone(&instance), &self.command_pool, *graphics_queue);
    self.texture_shader.add_texture(Arc::clone(&instance), Arc::clone(&device), &self.descriptor_set_pool, reference.to_string(), &self.resources.get_texture(reference).unwrap(), &self.sampler, &self.window_dimensions);
  }
  
  fn preload_font(&mut self, reference: String, font_texture: String, font: &[u8]) {
    
  }
  
  fn add_font(&mut self, reference: String, font_texture: String, font: &[u8]) {
    
  }
  
  fn load_static_geometry(&mut self, reference: String, verticies: Vec<graphics::Vertex2d>, indicies: Vec<u32>) {
    
  }
  
  fn load_dynamic_geometry(&mut self, reference: String, verticies: Vec<graphics::Vertex2d>, indicies: Vec<u32>) {
    
  }
  
  fn load_shaders(&mut self) {
    
  }
  
  fn init(&mut self) {
    
  }
  
  fn pre_draw(&mut self) {
    self.resources.recieve_objects();
    
    if !self.recreate_swapchain {
      return;
    }
    
    println!("Reszing window");
    self.recreate_swapchain = false;
    
    self.window.device().wait();
    
    for fence in &self.fences {
      let device = self.window.device();
      fence.wait(Arc::clone(&device));
    }
    
    self.window.recreate_swapchain();
    self.window_dimensions = self.window.get_current_extent();
    
    for i in 0..self.command_buffers.len() {
      let device = self.window.device();
      self.command_buffers[i].free(Arc::clone(&device), &self.command_pool)
    }
    self.command_buffers.clear();
    
    {
      let device = self.window.device();
      let instance = self.window.instance();
      let image_views = self.window.swapchain_image_views();
      
      self.command_buffers = self.command_pool.create_command_buffers(Arc::clone(&device), image_views.len() as u32);
      
      self.texture_shader.recreate(Arc::clone(&instance), Arc::clone(&device), image_views, &self.window_dimensions, &self.texture, &self.sampler);
    }
    
    self.draw(&Vec::new());
    
    self.window.device().wait();
    println!("Finished resize");
  }
  
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
    //
    // Build drawcalls
    //
    if self.recreate_swapchain {
      return;
    }
    
    let device = self.window.device();
    let instance = self.window.instance();
    let window_size = &self.window_dimensions;
    
    let index_count = 6;
    
    let clear_values: Vec<vk::ClearValue> = {
      vec!(
        vk::ClearValue { 
          color: vk::ClearColorValue { float32: [self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w] }
        }
      )
    };
    
    for i in 0..self.command_buffers.len() {
      let mut cmd = CommandBufferBuilder::primary_one_time_submit(Arc::clone(&self.command_buffers[i]));
      cmd = cmd.begin_command_buffer(Arc::clone(&device));
      cmd = self.texture_shader.begin_renderpass(Arc::clone(&device), cmd, &clear_values, &window_size, i);
      
      cmd = cmd.set_viewport(Arc::clone(&device), 0.0, 0.0, window_size.width as f32, window_size.height as f32);
      cmd = cmd.set_scissor(Arc::clone(&device), 0, 0, window_size.width, window_size.height);
      
      for draw in draw_calls {
        let black_and_white = draw.is_black_and_white();
        match draw.get_type() {
          DrawType::DrawTextured(ref info) => {
            let (reference, position, scale, rotation, alpha) = info.clone(); 
            
            let texture_resource = self.resources.get_texture(reference.clone());
            if let Some(texture) = texture_resource {
              cmd = self.texture_shader.draw_texture(Arc::clone(&instance), Arc::clone(&device), cmd, position, scale, rotation, None, Some(Vector4::new(0.0, 0.0, 0.0, alpha)), black_and_white, true, reference.to_string(), &texture, &self.sampler, &self.window_dimensions, &self.descriptor_set_pool);
            }
          },
          DrawType::DrawSpriteSheet(ref info) => {
            let (reference, position, scale, rotation, alpha, sprite_details) = info.clone(); 
            
            let texture_resource = self.resources.get_texture(reference.clone());
            if let Some(texture) = texture_resource {
              cmd = self.texture_shader.draw_texture(Arc::clone(&instance), Arc::clone(&device), cmd, position, scale, rotation, Some(sprite_details), Some(Vector4::new(0.0, 0.0, 0.0, alpha)), black_and_white, true, reference.to_string(), &texture, &self.sampler, &self.window_dimensions, &self.descriptor_set_pool);
            }
          },
          DrawType::DrawColoured(ref info) => {
            let (position, scale, colour, rotation) = info.clone(); 
            
            cmd = self.texture_shader.draw_texture(Arc::clone(&instance), Arc::clone(&device), cmd, position, scale, rotation, None, Some(colour), black_and_white, false, "".to_string(), &self.texture, &self.sampler, &self.window_dimensions, &self.descriptor_set_pool);
          },
          _ => {
            
          }
        }
      }
      
      cmd = cmd.end_render_pass(Arc::clone(&device));
      cmd.end_command_buffer(Arc::clone(&device));
    }
    
    //
    // Actually Draw stuff
    //
    let device = self.window.device();
    let swapchain = self.window.get_swapchain();
    let graphics_queue = self.window.get_graphics_queue();
    
    let mut current_buffer = self.window.aquire_next_image(Arc::clone(&device), &self.semaphore_image_available);
    
    self.fences[current_buffer].wait(Arc::clone(&device));
    self.fences[current_buffer].reset(Arc::clone(&device));
    
    match self.command_buffers[current_buffer].submit(Arc::clone(&device), swapchain, current_buffer as u32, &self.semaphore_image_available, &self.semaphore_render_finished, &self.fences[current_buffer], &graphics_queue) {
      vk::ERROR_OUT_OF_DATE_KHR => {
        self.recreate_swapchain = true;
      },
      e => { check_errors(e); },
    }
    
    if self.recreate_swapchain {
      return;
    }
      
    self.command_buffers[current_buffer].finish(Arc::clone(&device), &graphics_queue);
  }
  
  fn post_draw(&self) {
    
  }
  
  fn screen_resized(&mut self) {
    self.recreate_swapchain = true;
  }
  
  fn get_dimensions(&self) -> LogicalSize {
    LogicalSize::new(self.window_dimensions.width as f64, self.window_dimensions.height as f64)
  }
  
  fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn get_fonts(&self) -> HashMap<String, GenericFont> {
    HashMap::new()
  }
  
  fn get_dpi_scale(&self) -> f64 {
    1.0
  }
  
  fn is_ready(&self) -> bool {
    true
  }
  
  fn set_cursor_position(&mut self, x: f32, y: f32) {
    
  }
  
  fn show_cursor(&mut self) {
    
  }
  
  fn hide_cursor(&mut self) {
    
  }
  
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
    self.clear_colour = Vector4::new(r,g,b,a);
  }
  
  fn set_camera(&mut self, camera: Camera) {
    
  }
  
  fn get_camera(&self) -> Camera {
    Camera::default_vk()
  }
  
  fn num_drawcalls(&self) -> u32 {
    0
  }
}


impl Drop for CoreMaat {
  fn drop(&mut self) {
    self.window.device().wait();
    
    println!("Destroying Fences");
    for fence in &self.fences {
      let device = self.window.device();
      fence.wait(Arc::clone(&device));
      fence.destroy(Arc::clone(&device));
    }
    
    let device = self.window.device();
    
    self.resources.destroy(Arc::clone(&device));
    self.texture.destroy(Arc::clone(&device));
    self.sampler.destroy(Arc::clone(&device));
    
    self.texture_shader.destroy(Arc::clone(&device));
    
    self.descriptor_set_pool.destroy(Arc::clone(&device));
    
    self.command_pool.destroy(Arc::clone(&device));
    self.semaphore_image_available.destroy(Arc::clone(&device));
    self.semaphore_render_finished.destroy(Arc::clone(&device));
  }
}
