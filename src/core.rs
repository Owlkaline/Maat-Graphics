use vk;
use winit;
use image;
use cgmath::{Vector2, Vector3};
use winit::dpi::LogicalSize; 

use crate::camera::Camera;
use crate::drawcalls::DrawCall; 
use crate::graphics::CoreRender;
use crate::font::GenericFont;
use crate::graphics;

#[macro_use]
use crate::vulkan::vkenums::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout, ImageUsage, ImageType, ImageViewType, ImageTiling, Sample, Filter, AddressMode, MipmapMode, VkBool};

use crate::vulkan::VkWindow;
use crate::vulkan::Shader;
use crate::vulkan::pool::CommandPool;
use crate::vulkan::Instance;
use crate::vulkan::Device;
use crate::vulkan::pool::DescriptorPool;
use crate::vulkan::DescriptorSet;
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

// Simple offset_of macro akin to C++ offsetof
#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::uninitialized();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

#[derive(Clone)]
struct Vertex {
  pos: Vector2<f32>,
  colour: Vector3<f32>,
  uvs: Vector2<f32>,
}

impl Vertex {
  pub fn vertex_input_binding() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription {
      binding: 0,
      stride: (mem::size_of::<Vertex>()) as u32,
      inputRate: vk::VERTEX_INPUT_RATE_VERTEX,
    }
  }
  
  pub fn vertex_input_attributes() -> Vec<vk::VertexInputAttributeDescription> {
    let mut vertex_input_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> = Vec::with_capacity(2);
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::FORMAT_R32G32_SFLOAT,
        offset: offset_of!(Vertex, pos) as u32,//0,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::FORMAT_R32G32B32_SFLOAT,
        offset: offset_of!(Vertex, colour) as u32,//(mem::size_of::<f32>()*2) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 2,
        binding: 0,
        format: vk::FORMAT_R32G32_SFLOAT,
        offset: offset_of!(Vertex, uvs) as u32,
      }
    );
    
    vertex_input_attribute_descriptions
  }
}

pub struct CoreMaat {
  window: VkWindow,
  window_dimensions: vk::Extent2D,
  recreate_swapchain: bool,
  fences: Vec<Fence>,
  semaphore_image_available: Semaphore,
  semaphore_render_finished: Semaphore,
  command_pool: CommandPool,
  command_buffers: Vec<Arc<CommandBuffer>>,
  render_pass: RenderPass,
  framebuffers: Vec<Framebuffer>,
  vertex_shader: Shader,
  fragment_shader: Shader,
  descriptor_set_pool: DescriptorPool,
  descriptor_set: DescriptorSet,
  pipeline: Pipeline,
  vertex_buffer: Buffer<Vertex>,
  index_buffer: Buffer<u32>,
  uniform_buffer: Vec<Buffer<f32>>,
  texture: Image,
  sampler: Sampler,
}

impl CoreMaat {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> CoreMaat {
    let window = VkWindow::new(app_name, app_version, width, height, should_debug);
    
    let fences: Vec<Fence>;
    let semaphore_image_available: Semaphore;
    let semaphore_render_finished: Semaphore;
    let command_pool: CommandPool;
    let command_buffers: Vec<Arc<CommandBuffer>>;
    let render_pass: RenderPass;
    let framebuffers: Vec<Framebuffer>;
    let vertex_shader: Shader;
    let fragment_shader: Shader;
    let descriptor_set_pool: DescriptorPool;
    let descriptor_set: DescriptorSet;
    let pipelines: Pipeline;
    let vertex_buffer: Buffer<Vertex>;
    let index_buffer: Buffer<u32>;
    let mut uniform_buffer: Vec<Buffer<f32>> = Vec::new();
    
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
      
      vertex_shader = Shader::new(device, include_bytes!("./shaders/test_vert.spv"));
      fragment_shader = Shader::new(device, include_bytes!("./shaders/test_frag.spv"));
      
      semaphore_image_available = Semaphore::new(device);
      semaphore_render_finished = Semaphore::new(device);
      
      let colour_attachment = AttachmentInfo::new()
                                .format(format)
                                .multisample(0)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::Store)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::PresentSrcKHR)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
      let subpass = SubpassInfo::new().add_colour_attachment(0);
      render_pass = RenderPassBuilder::new()
                      .add_attachment(colour_attachment)
                      .add_subpass(subpass)
                      .build(device);
      
      framebuffers = CoreMaat::create_frame_buffers(device, &render_pass, &current_extent, image_views);
      fences = CoreMaat::create_fences(device, framebuffers.len() as u32);
      command_pool = CommandPool::new(device, graphics_family);
      command_buffers = command_pool.create_command_buffers(device, framebuffers.len() as u32);
      
      descriptor_set_pool = DescriptorPool::new(device, image_views.len() as u32, 1, 0);
      descriptor_set = DescriptorSet::new(device, &descriptor_set_pool, image_views.len() as u32);
      
      pipelines = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .vertex_binding(vec!(Vertex::vertex_input_binding()))
                  .vertex_attributes(Vertex::vertex_input_attributes())
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  .cull_mode_back()
                  .build(device);
      
      vertex_buffer = CoreMaat::create_vertex_buffer(instance, device, &command_pool, graphics_queue);
      index_buffer = CoreMaat::create_index_buffer(instance, device, &command_pool, graphics_queue);
      
      let image_usage = ImageUsage::transfer_dst_sampled();
      
      texture_image = Image::new(instance, &device, "./resources/Textures/Logo.png".to_string(), ImageType::Type2D, ImageViewType::Type2D, image_usage, &format, Sample::Count1Bit, ImageLayout::Undefined, ImageTiling::Optimal);
      sampler = SamplerBuilder::new()
                       .min_filter(Filter::Linear)
                       .mag_filter(Filter::Linear)
                       .address_mode(AddressMode::Repeat)
                       .mipmap_mode(MipmapMode::Linear)
                       .anisotropy(VkBool::True)
                       .max_anisotropy(8.0)
                       .build(device);
      
      for _ in 0..image_views.len() {
        uniform_buffer.push(CoreMaat::create_uniform_buffer(instance, device, &descriptor_set));
      }
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
      render_pass: render_pass,
      framebuffers: framebuffers,
      vertex_shader: vertex_shader,
      fragment_shader: fragment_shader,
      descriptor_set_pool: descriptor_set_pool,
      descriptor_set: descriptor_set,
      pipeline: pipelines,
      vertex_buffer: vertex_buffer,
      index_buffer: index_buffer,
      uniform_buffer: uniform_buffer,
      texture: texture_image,
      sampler: sampler,
    }
  }
  
  fn begin_single_time_command(device: &Device, command_pool: &CommandPool) -> CommandBuffer {
    let command_buffer = CommandBuffer::primary(device, command_pool);
    command_buffer.begin_command_buffer(device, vk::COMMAND_BUFFER_LEVEL_PRIMARY);
    command_buffer
  }
  
  fn end_single_time_command(device: &Device, command_buffer: CommandBuffer, command_pool: &CommandPool, graphics_queue: &vk::Queue) {
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
    
    command_buffer.end_command_buffer(device);
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      let command_pool = command_pool.local_command_pool();
      vk.QueueSubmit(*graphics_queue, 1, &submit_info, 0);
      vk.QueueWaitIdle(*graphics_queue);
      vk.FreeCommandBuffers(*device, *command_pool, 1, command_buffer.internal_object());
    }
  }
  
  fn create_uniform_buffer(instance: &Instance, device: &Device, descriptor_set: &DescriptorSet) -> Buffer<f32> {
    let mut uniform_buffer = UniformBufferBuilder::new().add_vector2();
    let mut buffer = uniform_buffer.build(instance, device);
    
    let data = UniformData::new().add_vector2(Vector2::new(0.4, 0.4));
    buffer.fill_buffer(device, data.build());
    
    descriptor_set.update_sets(device, &buffer);
    
    buffer
  }
  
  fn create_index_buffer(instance: &Instance, device: &Device, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<u32> {
    let indices = vec!(0, 1, 2, 2, 3, 0);
    
    let usage_src = BufferUsage::index_transfer_src_buffer();
    let usage_dst = BufferUsage::index_transfer_dst_buffer();
    
    let staging_buffer: Buffer<u32> = Buffer::cpu_buffer(instance, device, usage_src, indices.clone());
    let buffer: Buffer<u32> = Buffer::device_local_buffer(instance, device, usage_dst, indices);
    
    let command_buffer = CoreMaat::begin_single_time_command(device, command_pool);
    command_buffer.copy_buffer(device, &staging_buffer, &buffer);
    CoreMaat::end_single_time_command(device, command_buffer, command_pool, graphics_queue);
    
    staging_buffer.destroy(device);
    
    buffer
  }
  
  fn create_vertex_buffer(instance: &Instance, device: &Device, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<Vertex> {
    let triangle = vec!(
      Vertex { pos: Vector2::new(0.0, 0.5), colour: Vector3::new(1.0, 0.0, 0.0), uvs: Vector2::new(0.0, 0.0) },
      Vertex { pos: Vector2::new(-0.5, 0.5), colour: Vector3::new(0.0, 1.0, 0.0), uvs: Vector2::new(1.0, 0.0) },
      Vertex { pos: Vector2::new(-0.5, -0.5), colour: Vector3::new(0.0, 0.0, 1.0), uvs: Vector2::new(0.0, 1.0) },
      Vertex { pos: Vector2::new(0.0, -0.5), colour: Vector3::new(1.0, 0.0, 1.0), uvs: Vector2::new(1.0, 1.0) },
    );
    
    let usage_src = BufferUsage::vertex_transfer_src_buffer();
    let usage_dst = BufferUsage::vertex_transfer_dst_buffer();
    
    let staging_buffer: Buffer<Vertex> = Buffer::cpu_buffer(instance, device, usage_src, triangle.clone());
    let buffer: Buffer<Vertex> = Buffer::device_local_buffer(instance, device, usage_dst, triangle);
    
    let command_buffer = CoreMaat::begin_single_time_command(device, command_pool);
    command_buffer.copy_buffer(device, &staging_buffer, &buffer);
    CoreMaat::end_single_time_command(device, command_buffer, command_pool, graphics_queue);
    
    staging_buffer.destroy(device);
    
    buffer
  }
  
  fn create_frame_buffers(device: &Device, render_pass: &RenderPass, swapchain_extent: &vk::Extent2D, image_views: &Vec<vk::ImageView>) -> Vec<Framebuffer> {
    let mut framebuffers: Vec<Framebuffer> = Vec::with_capacity(image_views.len());
    
    for i in 0..image_views.len() {
      let framebuffer: Framebuffer = Framebuffer::new(device, render_pass, swapchain_extent, &image_views[i]);
      
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  fn create_fences(device: &Device, num_fences: u32) -> Vec<Fence> {
    let mut fences: Vec<Fence> = Vec::with_capacity(num_fences as usize);
    
    for _ in 0..num_fences {
      let fence: Fence = Fence::new(device);
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
    
  }
  
  fn add_texture(&mut self, reference: String, location: String) {
    
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
    if !self.recreate_swapchain {
      return;
    }
    
    println!("Reszing window");
    self.recreate_swapchain = false;
    
    self.window.device().wait();
    
    for fence in &self.fences {
      let device = self.window.device();
      fence.wait(device);
    }
    
    self.window.recreate_swapchain();
    self.window_dimensions = self.window.get_current_extent();
    
    for i in 0..self.framebuffers.len() {
      let device = self.window.device();
      self.framebuffers[i].destroy(device);
    }
    self.framebuffers.clear();
    
    let image_views = self.window.swapchain_image_views();
    for i in 0..image_views.len() {
      let device = self.window.device();
      self.framebuffers.push(Framebuffer::new(device, &self.render_pass, &self.window_dimensions, &image_views[i]));
    }
    
    for i in 0..self.command_buffers.len() {
      let device = self.window.device();
      self.command_buffers[i].free(device, &self.command_pool)
    }
    self.command_buffers.clear();
    
    {
      let device = self.window.device();
      self.command_buffers = self.command_pool.create_command_buffers(device, image_views.len() as u32);
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
    let window_size = &self.window_dimensions;
    
    let index_count = 6;
    
    let clear_values: Vec<vk::ClearValue> = {
      vec!(
        vk::ClearValue { 
          color: vk::ClearColorValue { float32: [0.0, 0.0, 0.2, 1.0] }
        }
      )
    };
    
    for i in 0..self.command_buffers.len() {
      let mut cmd = CommandBufferBuilder::primary_one_time_submit(Arc::clone(&self.command_buffers[i]));
      cmd = cmd.begin_command_buffer(device);
      cmd = cmd.begin_render_pass(device, &clear_values, &self.render_pass, &self.framebuffers[i].internal_object(), &window_size);
      
      cmd = cmd.set_viewport(device, 0.0, 0.0, window_size.width as f32, window_size.height as f32);
      cmd = cmd.set_scissor(device, 0, 0, window_size.width, window_size.height);
      cmd = cmd.draw_indexed(device, &self.vertex_buffer.internal_object(), &self.index_buffer.internal_object(), index_count, &self.pipeline, &self.descriptor_set.sets()[i]);
      
      cmd = cmd.end_render_pass(device);
      cmd.end_command_buffer(device);
    }
    
    //
    // Actually Draw stuff
    //
    let device = self.window.device();
    let swapchain = self.window.get_swapchain();
    let graphics_queue = self.window.get_graphics_queue();
    
    let mut current_buffer = self.window.aquire_next_image(device, &self.semaphore_image_available);
    
    self.fences[current_buffer].wait(device);
    self.fences[current_buffer].reset(device);
    
    // update uniform variables
    let data = UniformData::new().add_vector2(Vector2::new(-0.4, 0.1));
    self.uniform_buffer[current_buffer].fill_buffer(device, data.build());
    
    match self.command_buffers[current_buffer].submit(device, swapchain, current_buffer as u32, &self.semaphore_image_available, &self.semaphore_render_finished, &self.fences[current_buffer], &graphics_queue) {
      vk::ERROR_OUT_OF_DATE_KHR => {
        self.recreate_swapchain = true;
      },
      e => { check_errors(e); },
    }
    
    if self.recreate_swapchain {
      return;
    }
      
    self.command_buffers[current_buffer].finish(device, &graphics_queue);
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
      fence.wait(device);
      fence.destroy(device);
    }
    
    let device = self.window.device();
    
    self.texture.destroy(device);
    self.sampler.destroy(device);
    
    for uniform in &self.uniform_buffer {
      uniform.destroy(device);
    }
    
    self.index_buffer.destroy(device);
    self.vertex_buffer.destroy(device);
    
    self.pipeline.destroy(device);
    
    self.descriptor_set.destroy(device);
    self.descriptor_set_pool.destroy(device);
    
    self.vertex_shader.destroy(device);
    self.fragment_shader.destroy(device);
    
    for framebuffer in &self.framebuffers {
     framebuffer.destroy(device);
    }
    self.render_pass.destroy(device);
    
    self.command_pool.destroy(device);
    self.semaphore_image_available.destroy(device);
    self.semaphore_render_finished.destroy(device);
  }
}
