/*
VkResult vkAcquireNextImageKHR(
    VkDevice                                    device,
    VkSwapchainKHR                              swapchain,
    uint64_t                                    timeout,
    VkSemaphore                                 semaphore,
    VkFence                                     fence,
    uint32_t*                                   pImageIndex);
*/

use vk;
use winit;
use image;
use cgmath::{perspective, Matrix4, Deg, Rad, Vector2, Vector3};
use cgmath::prelude::SquareMatrix;

use libc::memcpy;

use crate::modules::VkWindow;
use crate::modules::Shader;
use crate::modules::pool::CommandPool;
use crate::modules::Instance;
use crate::modules::Device;
use crate::modules::pool::DescriptorPool;
use crate::modules::DescriptorSet;
use crate::modules::Pipeline;
use crate::modules::RenderPass;
use crate::modules::sync::Fence;
use crate::modules::sync::Semaphore;
use crate::modules::buffer::Buffer;
use crate::modules::buffer::BufferUsage;
use crate::modules::buffer::Framebuffer;
use crate::modules::buffer::CommandBuffer;
use crate::modules::buffer::CommandBufferBuilder;
use crate::ownage::check_errors;

use std::ptr;
use std::mem;
use std::sync::Arc;
use std::ffi::c_void;
use std::ffi::CString;

#[derive(Clone)]
struct Vertex {
  pos: Vector2<f32>,
  colour: Vector3<f32>,
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
        offset: 0,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::FORMAT_R32G32B32_SFLOAT,
        offset: (mem::size_of::<f32>()*2) as u32,
      }
    );
    
    vertex_input_attribute_descriptions
  }
}

pub struct Vulkan {
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
  /*texture_image: vk::Image,
  texture_image_memory: vk::DeviceMemory,
  texture_image_view: vk::ImageView,
  texture_sampler: vk::Sampler,*/
  vertex_buffer: Buffer<Vertex>,
  index_buffer: Buffer<u32>,
  uniform_buffer: Vec<Buffer<f32>>,
}

impl Vulkan {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> Vulkan {
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
   /* let texture_image: vk::Image;
    let texture_image_memory: vk::DeviceMemory;
    let texture_image_view: vk::ImageView;
    let texture_sampler: vk::Sampler;*/
    let vertex_buffer: Buffer<Vertex>;
    let index_buffer: Buffer<u32>;
    let mut uniform_buffer: Vec<Buffer<f32>> = Vec::new();
    
    let current_extent = window.get_current_extent();
    
    {
      let instance = window.instance();
      let device = window.device();
      let format = window.swapchain_format();
      let graphics_family = window.get_graphics_family();
      let graphics_queue = window.get_graphics_queue();
      let image_views = window.swapchain_image_views();
      let phys_device = window.physical_device();
      
      vertex_shader = Shader::new(device, include_bytes!("../shaders/test_vert.spv"));
      fragment_shader = Shader::new(device, include_bytes!("../shaders/test_frag.spv"));
      
      semaphore_image_available = Semaphore::new(device);
      semaphore_render_finished = Semaphore::new(device);
      render_pass = RenderPass::new(device, &format);
      framebuffers = Vulkan::create_frame_buffers(device, &render_pass, &current_extent, image_views);
      fences = Vulkan::create_fences(device, framebuffers.len() as u32);
      command_pool = CommandPool::new(device, graphics_family);
      command_buffers = command_pool.create_command_buffers(device, framebuffers.len() as u32);
      
      descriptor_set_pool = DescriptorPool::new(device, image_views.len() as u32, 1, 0);
      descriptor_set = DescriptorSet::new(device, &descriptor_set_pool, image_views.len() as u32);
      
      pipelines = Pipeline::new(device, vertex_shader.get_shader(), &fragment_shader.get_shader(), &render_pass, &current_extent, &format, &descriptor_set, vec!(Vertex::vertex_input_binding()), Vertex::vertex_input_attributes());
      
      /*
      let (texture, texture_memory, texture_view) = Vulkan::create_texture_image(vk, vk_instance, device, phys_device, &format, "./src/shaders/statue.jpg".to_string());
      texture_image = texture;
      texture_image_memory = texture_memory;
      texture_image_view = texture_view;
      
      texture_sampler = Vulkan::create_texture_sampler(vk, device);*/
      
      vertex_buffer = Vulkan::create_vertex_buffer(instance, device, &command_pool, graphics_queue);
      index_buffer = Vulkan::create_index_buffer(instance, device, &command_pool, graphics_queue);
      
      for i in 0..image_views.len() {
        uniform_buffer.push(Vulkan::create_uniform_buffer(instance, device, &current_extent, &descriptor_set));
      }
    }
    
    Vulkan {
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
     /* texture_image: texture_image,
      texture_image_memory: texture_image_memory,
      texture_image_view: texture_image_view,
      texture_sampler: texture_sampler*/
      vertex_buffer: vertex_buffer,
      index_buffer: index_buffer,
      uniform_buffer: uniform_buffer,
    }
  }
  
  pub fn setup(&mut self) {
    /*
    (Success, Not ready, device lost)
    VkResult vkGetFenceStatus(
    VkDevice                                    device,
    VkFence                                     fence);
    */ 
    
    /*
    Sets to unsignaled from host
    VkResult vkResetFences(
    VkDevice                                    device,
    uint32_t                                    fenceCount,
    const VkFence*                              pFences);
    */
    
    /*
    VkResult vkWaitForFences(
    VkDevice                                    device,
    uint32_t                                    fenceCount,
    const VkFence*                              pFences,
    VkBool32                                    waitAll,
    uint64_t                                    timeout);
    */
  }
  
  pub fn build(&mut self) {
    if self.recreate_swapchain {
    //  return;
    }
    
    let device = self.window.device();
    let window_size = self.window.get_current_extent();
    
    let index_count = 3;
    
    let clear_values: Vec<vk::ClearValue> = {
      vec!(
        vk::ClearValue { 
          color: vk::ClearColorValue { float32: [0.0, 0.0, 0.2, 1.0] }
        }
      )
    };
    
    for i in 0..self.command_buffers.len() {
      let mut cmd = CommandBufferBuilder::primary_one_time_submit(device, Arc::clone(&self.command_buffers[i]));
      cmd = cmd.begin_command_buffer(device);
      cmd = cmd.begin_render_pass(device, &clear_values, &self.render_pass, &self.framebuffers[i].internal_object(), &window_size);
      cmd = cmd.draw_indexed(device, &self.vertex_buffer.internal_object(), &self.index_buffer.internal_object(), index_count, &self.pipeline, &self.descriptor_set.sets()[i]);
      
      cmd = cmd.draw_indexed(device, &self.vertex_buffer.internal_object(), &self.index_buffer.internal_object(), index_count, &self.pipeline, &self.descriptor_set.sets()[i]);
      
      cmd = cmd.end_render_pass(device);
      cmd = cmd.end_command_buffer(device);
    }
  }
  
  pub fn draw(&mut self) {
    if self.recreate_swapchain {
    //  return;
    }
    
    let vk = self.window.device_pointers();
    let the_device = self.window.device();
    let device = the_device.internal_object();
    let swapchain = self.window.get_swapchain();
    let graphics_queue = self.window.get_graphics_queue();
    let present_queue = self.window.get_present_queue();
    
    let mut current_buffer = 0;
    unsafe {
      check_errors(vk.AcquireNextImageKHR(*device, *swapchain, 0, *self.semaphore_image_available.internal_object(), 0, &mut current_buffer));
    }
    self.fences[current_buffer as usize].wait(the_device);
    self.fences[current_buffer as usize].reset(the_device);
    
    let current_buffer = current_buffer as usize;
    
    // update uniform variables
    let mut data = self.uniform_buffer[current_buffer].internal_data();
    data[0] = -0.4;
    data[1] = 0.1;
    self.uniform_buffer[current_buffer].fill_buffer(the_device, data);
    
    let pipeline_stage_flags = vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    
    let mut submit_info: vk::SubmitInfo = {
      vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 1,
        pWaitSemaphores: self.semaphore_image_available.internal_object(),
        pWaitDstStageMask: &pipeline_stage_flags,
        commandBufferCount: 1,
        pCommandBuffers: self.command_buffers[current_buffer].internal_object(),
        signalSemaphoreCount: 1,
        pSignalSemaphores: self.semaphore_render_finished.internal_object(),
      }
    };
    unsafe {
      check_errors(vk.QueueSubmit(*graphics_queue, 1, &submit_info, *self.fences[current_buffer].internal_object()));
    }
    
    let present_info_khr = {
      vk::PresentInfoKHR {
        sType: vk::STRUCTURE_TYPE_PRESENT_INFO_KHR,
        pNext: ptr::null(),
        waitSemaphoreCount: 1,
        pWaitSemaphores: self.semaphore_render_finished.internal_object(),
        swapchainCount: 1,
        pSwapchains: swapchain,
        pImageIndices: &(current_buffer as u32),
        pResults: ptr::null_mut(),
      }
    };
    
    unsafe {
      match vk.QueuePresentKHR(*graphics_queue, &present_info_khr) {
        vk::ERROR_OUT_OF_DATE_KHR => {
          self.recreate_swapchain = true;
        },
        e => { check_errors(e); },
      }
    //  vk.DeviceWaitIdle(*device);
      vk.QueueWaitIdle(*graphics_queue);
    }
  }
  
  /*
  pub fn check_errors(result: vk::Result) -> bool {
    match result {
        vk::SUCCESS => true,
        vk::NOT_READY => { println!("Success: A fence or query has not yet completed"); true },
        vk::TIMEOUT => { println!("Success: A wait operation has not completed in the specified time"); true },
        vk::EVENT_SET => { println!("Success: An event is signaled"); true },
        vk::EVENT_RESET => { println!("Success: An event is unsignaled"); true },
        vk::INCOMPLETE => {println!("Success: A return array was too small for the result"); true },
        vk::ERROR_OUT_OF_HOST_MEMORY => panic!("Vulkan out of host memory"),
        vk::ERROR_OUT_OF_DEVICE_MEMORY => panic!("Vulkan out of device memory"),
        vk::ERROR_INITIALIZATION_FAILED => panic!("Vulkan initialization failed"),
        vk::ERROR_DEVICE_LOST => panic!("Vulkan device lost"),
        vk::ERROR_MEMORY_MAP_FAILED => panic!("Vulkan memorymap failed"),
        vk::ERROR_LAYER_NOT_PRESENT => panic!("Vulkan layer not present"),
        vk::ERROR_EXTENSION_NOT_PRESENT => panic!("Vulkan extension not present"),
        vk::ERROR_FEATURE_NOT_PRESENT => panic!("Vulkan feature not present"),
        vk::ERROR_INCOMPATIBLE_DRIVER => panic!("Vulkan incompatable driver"),
        vk::ERROR_TOO_MANY_OBJECTS => panic!("Vulkan too many objects"),
        vk::ERROR_FORMAT_NOT_SUPPORTED => panic!("Vulkan format not supported"),
        vk::ERROR_SURFACE_LOST_KHR => panic!("Vulkan surface last khr"),
        vk::ERROR_NATIVE_WINDOW_IN_USE_KHR => panic!("Vulkan window in use khr"),
        vk::SUBOPTIMAL_KHR => panic!("Vulkan suboptimal khr"),
        vk::ERROR_OUT_OF_DATE_KHR => panic!("Vulkan out of date khr"),
        vk::ERROR_INCOMPATIBLE_DISPLAY_KHR => panic!("Vulkan incompatable display khr"),
        vk::ERROR_VALIDATION_FAILED_EXT => panic!("Vulkan validation failed ext"),
        vk::ERROR_OUT_OF_POOL_MEMORY_KHR => panic!("Vulkan of out pool memory khr"),
        vk::ERROR_INVALID_SHADER_NV => panic!("Vulkan function returned \
                                               VK_ERROR_INVALID_SHADER_NV"),
        c => unreachable!("Unexpected error code returned by Vulkan: {}", c),
    }
  }
  */
  
  /*
  pub fn resize_window(&mut self) {
    if !self.recreate_swapchain {
      return;
    }
    println!("Reszing window");
    self.recreate_swapchain = false;
    
    self.window_dimensions = self.window.get_current_extent();
    
    self.window.recreate_swapchain_images(&self.window_dimensions);
    
    {
      let vk = self.window.device_pointers();
      let device = self.window.device();
      
      unsafe {
        for i in 0..self.fences.len() {
          check_errors(vk.WaitForFences(*device, 1, &self.fences[i], vk::TRUE, u64::max_value()));
          check_errors(vk.ResetFences(*device, 1, &self.fences[i]));
        }
        
        vk.DeviceWaitIdle(*device);
        
        for i in 0..self.framebuffers.len() {
          vk.DestroyFramebuffer(*device, self.framebuffers[i], ptr::null());
        }
        
        vk.FreeCommandBuffers(*device, self.command_pool, self.command_buffers.len() as u32, self.command_buffers.as_mut_ptr());
      }
    }
    
    self.window.recreate_swapchain_images(&self.window_dimensions);
    let image_views = self.window.swapchain_image_views();
    
    {
      let vk = self.window.device_pointers();
      let device = self.window.device();
      
      
      self.framebuffers = Vulkan::create_frame_buffers(vk, device, &self.render_pass, &self.window_dimensions, image_views);
      self.command_buffers = Vulkan::create_command_buffers(vk, device, &self.command_pool, self.framebuffers.len() as u32);
      
      unsafe {
        vk.DeviceWaitIdle(*device);
      }
    }
    
    println!("Finished resize");
  }*/
  
  pub fn window_resized(&mut self) {
    self.recreate_swapchain = true;
  }
  
  pub fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn begin_single_time_command(device: &Device, command_pool: &CommandPool) -> CommandBuffer {
    let mut command_buffer = CommandBuffer::primary(device, command_pool);
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
  
  fn create_uniform_buffer(instance: &Instance, device: &Device, swapchain_extent: &vk::Extent2D, descriptor_set: &DescriptorSet) -> Buffer<f32> {
    
    let usage = BufferUsage::uniform_buffer();
    let real_data: Vec<f32> = vec!(0.4, 0.4);
    
    let uniform_buffer = Buffer::cpu_buffer(instance, device, usage, real_data);
    
    descriptor_set.update_sets(device, &uniform_buffer);
    
    uniform_buffer
     /*
    let perspective = perspective(Deg(60.0), (swapchain_extent.width as f32 / swapchain_extent.height as f32), 0.1, 256.0);
    let view_matrix = Matrix4::identity();
    let model_matrix = Matrix4::identity();
    
    struct uniform {
      projectionMatrix: [[f32; 4]; 4],
      modelMatrix: [[f32; 4]; 4],
      viewMatrix: [[f32; 4]; 4],
    };
    
    let ubo = uniform {
      projectionMatrix: perspective.into(),
      modelMatrix: model_matrix.into(),
      viewMatrix: view_matrix.into(),
    };
    
    let mut real_data: [f32; 48] = unsafe { mem::uninitialized() };
    
    for i in 0..4 {
      for j in 0..4 {
        real_data[i] = ubo.projectionMatrix[i][j];
      }
    }
    for i in 0..4 {
      for j in 0..4 {
        real_data[16+i] = ubo.modelMatrix[i][j];
      }
    }
    for i in 0..4 {
      for j in 0..4 {
        real_data[32+i] = ubo.viewMatrix[i][j];
      }
    }
    
    let real_data: [f32; 2] = Vector2::new(0.4, 0.4).into();
    
    let mut data = unsafe { mem::uninitialized() };
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      check_errors(vk.MapMemory(*device, uniform_buffer_memory, 0, buffer_size, 0, &mut data));
      memcpy(data, real_data.as_ptr() as *const _, (mem::size_of::<f32>() * 48));
      vk.UnmapMemory(*device, uniform_buffer_memory);
    }
    
    (uniform_buffer, uniform_buffer_memory)*/
  }
  
  fn create_index_buffer(instance: &Instance, device: &Device, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<u32> {
    let indices = vec!(0, 1, 2);
    
    let usage_src = BufferUsage::index_transfer_src_buffer();
    let usage_dst = BufferUsage::index_transfer_dst_buffer();
    
    let staging_buffer: Buffer<u32> = Buffer::cpu_buffer(instance, device, usage_src, indices.clone());
    let buffer: Buffer<u32> = Buffer::device_local_buffer(instance, device, usage_dst, indices);
    
    let mut command_buffer = Vulkan::begin_single_time_command(device, command_pool);
    command_buffer.copy_buffer(device, &staging_buffer, &buffer);
    Vulkan::end_single_time_command(device, command_buffer, command_pool, graphics_queue);
    
    staging_buffer.destroy(device);
    
    buffer
  }
  
  fn create_vertex_buffer(instance: &Instance, device: &Device, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<Vertex> {
    let triangle = vec!(
      Vertex { pos: Vector2::new(0.0, -0.5), colour: Vector3::new(1.0, 0.0, 0.0) },
      Vertex { pos: Vector2::new(0.5, 0.5), colour: Vector3::new(0.0, 1.0, 0.0) },
      Vertex { pos: Vector2::new(-0.5, 0.5), colour: Vector3::new(0.0, 0.0, 1.0) },
    );
    
    let usage_src = BufferUsage::vertex_transfer_src_buffer();
    let usage_dst = BufferUsage::vertex_transfer_dst_buffer();
    
    let staging_buffer: Buffer<Vertex> = Buffer::cpu_buffer(instance, device, usage_src, triangle.clone());
    let buffer: Buffer<Vertex> = Buffer::device_local_buffer(instance, device, usage_dst, triangle);
    
    let mut command_buffer = Vulkan::begin_single_time_command(device, command_pool);
    command_buffer.copy_buffer(device, &staging_buffer, &buffer);
    Vulkan::end_single_time_command(device, command_buffer, command_pool, graphics_queue);
    
    staging_buffer.destroy(device);
    
    buffer
  }
  
  fn create_frame_buffers(device: &Device, render_pass: &RenderPass, swapchain_extent: &vk::Extent2D, image_views: &Vec<vk::ImageView>) -> Vec<Framebuffer> {
    let mut framebuffers: Vec<Framebuffer> = Vec::with_capacity(image_views.len());
    
    for i in 0..image_views.len() {
      let mut framebuffer: Framebuffer = Framebuffer::new(device, render_pass, swapchain_extent, &image_views[i]);
      
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  fn create_fences(device: &Device, num_fences: u32) -> Vec<Fence> {
    let mut fences: Vec<Fence> = Vec::with_capacity(num_fences as usize);
    
    for i in 0..num_fences {
      let mut fence: Fence = Fence::new(device);
      fences.push(fence);
    }
    
    fences
  }
}

impl Drop for Vulkan {
  fn drop(&mut self) {
    let device = self.window.device();
    unsafe {
      device.wait();
      
      println!("Destroying Fences");
      for fence in &self.fences {
        fence.wait(device);
        fence.destroy(device);
      }
      
      for uniform in &self.uniform_buffer {
        uniform.destroy(device);
      }
      
      self.index_buffer.destroy(device);
      self.vertex_buffer.destroy(device);
      
      /*
      vk.DestroySampler(*device, self.texture_sampler, ptr::null());
      vk.DestroyImageView(*device, self.texture_image_view, ptr::null());
      vk.FreeMemory(*device, self.texture_image_memory, ptr::null());
      vk.DestroyImage(*device, self.texture_image, ptr::null());*/
      
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
}
