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

use modules::VkWindow;
use modules::Shader;
use modules::CommandPool;
use modules::Instance;
use modules::Device;
use modules::DescriptorPool;
use modules::DescriptorSet;
use ownage::check_errors;

use std::ptr;
use std::mem;
use std::ffi::c_void;
use std::ffi::CString;

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
        format: vk::FORMAT_R32G32_SFLOAT,//*swapchain_format,
        offset: 0,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::FORMAT_R32G32B32_SFLOAT,//*swapchain_format,
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
  fences: Vec<vk::Fence>,
  semaphore_image_available: vk::Semaphore,
  semaphore_render_finished: vk::Semaphore,
  command_pool: CommandPool,
  command_buffers: Vec<vk::CommandBuffer>,
  render_pass: vk::RenderPass,
  framebuffers: Vec<vk::Framebuffer>,
  vertex_shader: Shader,
  fragment_shader: Shader,
 // descriptor_set_layout: vk::DescriptorSetLayout,
  descriptor_set_pool: DescriptorPool,
  descriptor_set: DescriptorSet,
  pipelines: Vec<vk::Pipeline>,
  pipeline_cache: vk::PipelineCache,
  pipeline_layout: vk::PipelineLayout,
  /*texture_image: vk::Image,
  texture_image_memory: vk::DeviceMemory,
  texture_image_view: vk::ImageView,
  texture_sampler: vk::Sampler,*/
  vertex_buffer: vk::Buffer,
  vertex_buffer_memory: vk::DeviceMemory,
  index_buffer: vk::Buffer,
  index_buffer_memory: vk::DeviceMemory,
  uniform_buffer: vk::Buffer,
  uniform_buffer_memory: vk::DeviceMemory,
}

impl Vulkan {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> Vulkan {
    let window = VkWindow::new(app_name, app_version, width, height, should_debug);
    
    let fences: Vec<vk::Fence>;
    let semaphore_image_available: vk::Semaphore;
    let semaphore_render_finished: vk::Semaphore;
    let command_pool: CommandPool;
    let command_buffers: Vec<vk::CommandBuffer>;
    let render_pass: vk::RenderPass;
    let framebuffers: Vec<vk::Framebuffer>;
    let vertex_shader: Shader;
    let fragment_shader: Shader;
    let descriptor_set_pool: DescriptorPool;
    let descriptor_set: DescriptorSet;
    let pipelines: Vec<vk::Pipeline>;
    let pipeline_cache: vk::PipelineCache;
    let pipeline_layout: vk::PipelineLayout;
   /* let texture_image: vk::Image;
    let texture_image_memory: vk::DeviceMemory;
    let texture_image_view: vk::ImageView;
    let texture_sampler: vk::Sampler;*/
    let vertex_buffer: vk::Buffer;
    let vertex_buffer_memory: vk::DeviceMemory;
    let index_buffer: vk::Buffer;
    let index_buffer_memory: vk::DeviceMemory;
    let uniform_buffer: vk::Buffer;
    let uniform_buffer_memory: vk::DeviceMemory;
    
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
      
      let (semaphore1, semaphore2) = Vulkan::create_semaphores(device);
      semaphore_image_available = semaphore1;
      semaphore_render_finished = semaphore2;
      render_pass = Vulkan::create_render_pass(device, &format);
      framebuffers = Vulkan::create_frame_buffers(device, &render_pass, &current_extent, image_views);
      fences = Vulkan::create_fences(device, framebuffers.len() as u32);
      command_pool = CommandPool::new(device, graphics_family);
      command_buffers = command_pool.create_command_buffers(device, framebuffers.len() as u32);
      
      //descriptor_set_layout = Vulkan::create_descriptor_set_layout(device);
      descriptor_set_pool = DescriptorPool::new(device, 1, 0);
      descriptor_set = DescriptorSet::new(device, &descriptor_set_pool);//Vulkan::create_descriptor_sets(device, &descriptor_set_layout, &descriptor_set_pool);
      
      let (pipeline, cache, layout) = Vulkan::create_pipelines(device, vertex_shader.get_shader(), &fragment_shader.get_shader(), &render_pass, &current_extent, &format, &descriptor_set);
      pipelines = pipeline;
      pipeline_cache = cache;
      pipeline_layout = layout;
      /*
      let (texture, texture_memory, texture_view) = Vulkan::create_texture_image(vk, vk_instance, device, phys_device, &format, "./src/shaders/statue.jpg".to_string());
      texture_image = texture;
      texture_image_memory = texture_memory;
      texture_image_view = texture_view;
      
      texture_sampler = Vulkan::create_texture_sampler(vk, device);*/
      
      let (vertex, vertex_memory) = Vulkan::create_vertex_buffer(instance, device, &command_pool, graphics_queue);
      vertex_buffer = vertex;
      vertex_buffer_memory = vertex_memory;
      
      let (index, index_memory) = Vulkan::create_index_buffer(instance, device, &command_pool, graphics_queue);
      index_buffer = index;
      index_buffer_memory = index_memory;
      
      let (uniform, uniform_memory) = Vulkan::create_uniform_buffer(instance, device, &current_extent, &descriptor_set);
      uniform_buffer = uniform;
      uniform_buffer_memory = uniform_memory;
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
      pipelines: pipelines,
      pipeline_cache: pipeline_cache,
      pipeline_layout: pipeline_layout,
     /* texture_image: texture_image,
      texture_image_memory: texture_image_memory,
      texture_image_view: texture_image_view,
      texture_sampler: texture_sampler*/
      vertex_buffer: vertex_buffer,
      vertex_buffer_memory: vertex_buffer_memory,
      index_buffer: index_buffer,
      index_buffer_memory: index_buffer_memory,
      uniform_buffer: uniform_buffer,
      uniform_buffer_memory: uniform_buffer_memory,
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
    
    let vk = self.window.device_pointers();
    let window_size = self.window.get_current_extent();
    
    let command_buffer_begin_info = vk::CommandBufferBeginInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      pNext: ptr::null(),
      flags: vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
      pInheritanceInfo: ptr::null(),
    };
    
    let clear_values: vk::ClearValue = {
      vk::ClearValue { 
        color: vk::ClearColorValue { float32: [0.0, 0.0, 0.2, 1.0] }
      }
    };
    
    let mut render_pass_begin_info = {
      vk::RenderPassBeginInfo {
        sType: vk::STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
        pNext: ptr::null(),
        renderPass: self.render_pass,
        framebuffer: self.framebuffers[0],
        renderArea: vk::Rect2D { offset: vk::Offset2D {x: 0, y: 0 }, extent: vk::Extent2D { width: window_size.width, height: window_size.height, } },
        clearValueCount: 1,
        pClearValues: &clear_values,
      }
    };
    
    for i in 0..self.command_buffers.len() {
      render_pass_begin_info.framebuffer = self.framebuffers[i];
      
      unsafe {
        check_errors(vk.BeginCommandBuffer(self.command_buffers[i], &command_buffer_begin_info));
        
        vk.CmdBeginRenderPass(self.command_buffers[i], &render_pass_begin_info, vk::SUBPASS_CONTENTS_INLINE);
        
        vk.CmdBindDescriptorSets(self.command_buffers[i], vk::PIPELINE_BIND_POINT_GRAPHICS, self.pipeline_layout, 0, 1, self.descriptor_set.set(), 0, ptr::null());
        vk.CmdBindPipeline(self.command_buffers[i], vk::PIPELINE_BIND_POINT_GRAPHICS, self.pipelines[0]);
        vk.CmdBindVertexBuffers(self.command_buffers[i], 0, 1, &self.vertex_buffer, &0);
        vk.CmdBindIndexBuffer(self.command_buffers[i], self.index_buffer, 0, vk::INDEX_TYPE_UINT32);
        
        let indices_count = 3;
        vk.CmdDrawIndexed(self.command_buffers[i], indices_count, 1, 0, 0, 0);
        vk.CmdEndRenderPass(self.command_buffers[i]);
        
        check_errors(vk.EndCommandBuffer(self.command_buffers[i]));
      }
    }
  }
  
  pub fn draw(&mut self) {
    if self.recreate_swapchain {
    //  return;
    }
    
    let vk = self.window.device_pointers();
    let device = self.window.device();
    let device = device.local_device();
    let swapchain = self.window.get_swapchain();
    let graphics_queue = self.window.get_graphics_queue();
    let present_queue = self.window.get_present_queue();
    
    let mut current_buffer = 0;
    unsafe {
      check_errors(vk.AcquireNextImageKHR(*device, *swapchain, 0, self.semaphore_image_available, 0, &mut current_buffer));
      check_errors(vk.WaitForFences(*device, 1, &self.fences[current_buffer as usize], vk::TRUE, u64::max_value()));
      check_errors(vk.ResetFences(*device, 1, &self.fences[current_buffer as usize]));
    }
    
    let current_buffer = current_buffer as usize;
    
    let pipeline_stage_flags = vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    
    let mut submit_info: vk::SubmitInfo = {
      vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 1,
        pWaitSemaphores: &self.semaphore_image_available,
        pWaitDstStageMask: &pipeline_stage_flags,
        commandBufferCount: 1,
        pCommandBuffers: &self.command_buffers[current_buffer],
        signalSemaphoreCount: 1,
        pSignalSemaphores: &self.semaphore_render_finished,
      }
    };
    unsafe {
      check_errors(vk.QueueSubmit(*graphics_queue, 1, &submit_info, self.fences[current_buffer]));
    }
    
    let present_info_khr = {
      vk::PresentInfoKHR {
        sType: vk::STRUCTURE_TYPE_PRESENT_INFO_KHR,
        pNext: ptr::null(),
        waitSemaphoreCount: 1,
        pWaitSemaphores: &self.semaphore_render_finished,
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
  
  fn begin_single_time_command(device: &Device, command_pool: &CommandPool) -> vk::CommandBuffer {
    let command_pool = command_pool.local_command_pool();
    
    let command_buffer_allocate_info = {
      vk::CommandBufferAllocateInfo {
        sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: ptr::null(),
        commandPool: *command_pool,
        level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
        commandBufferCount: 1,
      }
    };
    
    let command_buffer_begin_info = {
      vk::CommandBufferBeginInfo {
        sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
        pNext: ptr::null(),
        flags: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
        pInheritanceInfo: ptr::null(),
      }
    };
    
    let mut command_buffer: vk::CommandBuffer = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.AllocateCommandBuffers(*device, &command_buffer_allocate_info, &mut command_buffer));
      check_errors(vk.BeginCommandBuffer(command_buffer, &command_buffer_begin_info));
    }
    
    command_buffer
  }
  
  fn end_single_time_command(device: &Device, command_buffer: vk::CommandBuffer, command_pool: &CommandPool, graphics_queue: &vk::Queue) {
    let submit_info = {
      vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 0,
        pWaitSemaphores: ptr::null(),
        pWaitDstStageMask: ptr::null(),
        commandBufferCount: 1,
        pCommandBuffers: &command_buffer,
        signalSemaphoreCount: 0,
        pSignalSemaphores: ptr::null(),
      }
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      let command_pool = command_pool.local_command_pool();
      vk.EndCommandBuffer(command_buffer);
      vk.QueueSubmit(*graphics_queue, 1, &submit_info, 0);
      vk.QueueWaitIdle(*graphics_queue);
      vk.FreeCommandBuffers(*device, *command_pool, 1, &command_buffer);
    }
  }
  
  fn create_uniform_buffer(instance: &Instance, device: &Device, swapchain_extent: &vk::Extent2D, descriptor_set: &DescriptorSet) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size: vk::DeviceSize = (mem::size_of::<f32>()*2) as u64;
    
    let (uniform_buffer, uniform_buffer_memory) = Vulkan::create_buffer(instance, device, buffer_size, vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);
    
    let descriptor_buffer_info = {
      vk::DescriptorBufferInfo {
        buffer: uniform_buffer,
        offset: 0,
        range: buffer_size,
      }
    };
    
    let write_descriptor_set = {
      vk::WriteDescriptorSet {
        sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
        pNext: ptr::null(),
        dstSet: *descriptor_set.set(),
        dstBinding: 0,
        dstArrayElement: 0,
        descriptorCount: 1,
        descriptorType: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        pImageInfo: ptr::null(),
        pBufferInfo: &descriptor_buffer_info,
        pTexelBufferView: ptr::null(),
      }
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      vk.UpdateDescriptorSets(*device, 1, &write_descriptor_set, 0, ptr::null());
    }
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
    }*/
    
    let real_data: [f32; 2] = Vector2::new(0.4, 0.4).into();
    
    let mut data = unsafe { mem::uninitialized() };
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.MapMemory(*device, uniform_buffer_memory, 0, buffer_size, 0, &mut data));
      memcpy(data, real_data.as_ptr() as *const _, (mem::size_of::<f32>() * 48));
      vk.UnmapMemory(*device, uniform_buffer_memory);
    }
    
    (uniform_buffer, uniform_buffer_memory)
  }
  
  fn create_index_buffer(instance: &Instance, device: &Device, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> (vk::Buffer, vk::DeviceMemory) {
    let indices = [
      0, 1, 2
    ];
    
    let mut buffer_size: vk::DeviceSize = (mem::size_of::<[f32; 3]>()) as u64;
    
    let (staging_index_buffer, staging_index_buffer_memory) = Vulkan::create_buffer(instance, device, buffer_size, vk::BUFFER_USAGE_TRANSFER_SRC_BIT, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);
    
    let mut host_visible_data = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.MapMemory(*device, staging_index_buffer_memory, 0, buffer_size, 0, &mut host_visible_data));
      memcpy(host_visible_data, indices.as_ptr() as *const _, buffer_size as usize);
      vk.UnmapMemory(*device, staging_index_buffer_memory);
    }
    
    let (index_buffer, index_buffer_memory) = Vulkan::create_buffer(instance, device, buffer_size, vk::BUFFER_USAGE_INDEX_BUFFER_BIT | vk::BUFFER_USAGE_TRANSFER_DST_BIT, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    
    let command_buffer = Vulkan::begin_single_time_command(device, command_pool);
    
    let buffer_copy = {
      vk::BufferCopy {
        srcOffset: 0,
        dstOffset: 0,
        size: buffer_size,
      }
    };
    
    unsafe {
      let vk = device.pointers();
      vk.CmdCopyBuffer(command_buffer, staging_index_buffer, index_buffer, 1, &buffer_copy);
    }
    
    Vulkan::end_single_time_command(device, command_buffer, command_pool, graphics_queue);
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      vk.FreeMemory(*device, staging_index_buffer_memory, ptr::null());
      vk.DestroyBuffer(*device, staging_index_buffer, ptr::null());
    }
    
    (index_buffer, index_buffer_memory)
  }
  
  fn create_vertex_buffer(instance: &Instance, device: &Device, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> (vk::Buffer, vk::DeviceMemory) {
   /* let square = [
      [[1.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
      [[-1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
      [[0.0, -1.0, 0.0], [0.0, 0.0, 1.0]],
    ];*/
    /*
    let triangle = vec!(
      [0.0, -0.5], 
      [0.5, 0.5],
      [-0.5, 0.5],
    );*/
    
    let triangle = vec!(
      Vertex { pos: Vector2::new(0.0, -0.5), colour: Vector3::new(1.0, 0.0, 0.0) },
      Vertex { pos: Vector2::new(0.5, 0.5), colour: Vector3::new(0.0, 1.0, 0.0) },
      Vertex { pos: Vector2::new(-0.5, 0.5), colour: Vector3::new(0.0, 0.0, 1.0) },
    );
    
    let mut buffer_size: vk::DeviceSize = (mem::size_of::<Vertex>()*triangle.len()) as u64;
    
    let (staging_vertex_buffer, staging_vertex_buffer_memory) = Vulkan::create_buffer(instance, device, buffer_size, vk::BUFFER_USAGE_TRANSFER_SRC_BIT, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);
    
    let mut host_visible_data = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.MapMemory(*device, staging_vertex_buffer_memory, 0, buffer_size, 0, &mut host_visible_data));
      memcpy(host_visible_data, triangle.as_ptr() as *const _, buffer_size as usize);
      vk.UnmapMemory(*device, staging_vertex_buffer_memory);
    }
    
    let (vertex_buffer, vertex_buffer_memory) = Vulkan::create_buffer(instance, device, buffer_size, vk::BUFFER_USAGE_VERTEX_BUFFER_BIT | vk::BUFFER_USAGE_TRANSFER_DST_BIT, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    
    let command_buffer = Vulkan::begin_single_time_command(device, command_pool);
    
    let buffer_copy = {
      vk::BufferCopy {
        srcOffset: 0,
        dstOffset: 0,
        size: buffer_size,
      }
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      vk.CmdCopyBuffer(command_buffer, staging_vertex_buffer, vertex_buffer, 1, &buffer_copy);
    }
    
    Vulkan::end_single_time_command(device, command_buffer, command_pool, graphics_queue);
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      vk.FreeMemory(*device, staging_vertex_buffer_memory, ptr::null());
      vk.DestroyBuffer(*device, staging_vertex_buffer, ptr::null());
    }
    
    (vertex_buffer, vertex_buffer_memory)
  }
  
  fn create_buffer(instance: &Instance, device: &Device, buffer_size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory) {
    
    let mut buffer: vk::Buffer = unsafe { mem::uninitialized() };
    let mut buffer_memory: vk::DeviceMemory = unsafe { mem::uninitialized() };
    
    let mut buffer_create_info = {
      vk::BufferCreateInfo {
        sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        size: buffer_size,
        usage: usage,
        sharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: ptr::null(),
      }
    };
    
    let mut memory_requirements: vk::MemoryRequirements = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.CreateBuffer(*device, &buffer_create_info, ptr::null(), &mut buffer));
      vk.GetBufferMemoryRequirements(*device, buffer, &mut memory_requirements);
    }
    
    let memory_type_bits_index = {
      let mut memory_properties: vk::PhysicalDeviceMemoryProperties = unsafe { mem::uninitialized() };
      
      unsafe {
        let vk = instance.pointers();
        let phys_device = device.physical_device();
        vk.GetPhysicalDeviceMemoryProperties(*phys_device, &mut memory_properties);
      }
      
      let mut index: i32 = -1;
      for i in 0..memory_properties.memoryTypeCount as usize {
        if memory_requirements.memoryTypeBits & (1 << i) != 0 && memory_properties.memoryTypes[i].propertyFlags & properties == properties {
          index = i as i32;
        }
      }
      
      if index == -1 {
        panic!("Failed to find suitable memory type");
      }
      
      index
    };
    
    let memory_allocate_info = {
      vk::MemoryAllocateInfo {
        sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
        pNext: ptr::null(),
        allocationSize: memory_requirements.size,
        memoryTypeIndex: memory_type_bits_index as u32,
      }
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.AllocateMemory(*device, &memory_allocate_info, ptr::null(), &mut buffer_memory));
      vk.BindBufferMemory(*device, buffer, buffer_memory, 0);
    }
    
    (buffer, buffer_memory)
  }
  
  fn create_texture_sampler(vk: &vk::DevicePointers, device: &vk::Device) -> vk::Sampler {
    let mut sampler: vk::Sampler = unsafe { mem::uninitialized() };
    
    let mag_filter = vk::FILTER_NEAREST;
    let min_filter = vk::FILTER_NEAREST;
    let mipmap_mode = vk::SAMPLER_MIPMAP_MODE_LINEAR;
    let address_mode = vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    
    let sampler_create_info = {
      vk::SamplerCreateInfo {
        sType: vk::STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        magFilter: mag_filter,
        minFilter: min_filter,
        mipmapMode: mipmap_mode,
        addressModeU: address_mode,
        addressModeV: address_mode,
        addressModeW: address_mode,
        mipLodBias: 0.0,
        anisotropyEnable: vk::TRUE,
        maxAnisotropy: 16.0,
        compareEnable: vk::FALSE,
        compareOp: vk::COMPARE_OP_ALWAYS,
        minLod: 0.0,
        maxLod: 0.0,
        borderColor: vk::BORDER_COLOR_INT_OPAQUE_BLACK,
        unnormalizedCoordinates: vk::FALSE,
      }
    };
    
    unsafe {
      check_errors(vk.CreateSampler(*device, &sampler_create_info, ptr::null(), &mut sampler));
    }
    
    sampler
  }
  
  fn create_image_view(vk: &vk::DevicePointers, device: &vk::Device, image: &vk::Image, format: &vk::Format) -> vk::ImageView {
    let mut image_view: vk::ImageView = unsafe { mem::uninitialized() };
    
    let component = vk::ComponentMapping {
      r: vk::COMPONENT_SWIZZLE_IDENTITY,
      g: vk::COMPONENT_SWIZZLE_IDENTITY,
      b: vk::COMPONENT_SWIZZLE_IDENTITY,
      a: vk::COMPONENT_SWIZZLE_IDENTITY,
    };
    
    let subresource = vk::ImageSubresourceRange {
      aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
      baseMipLevel: 0,
      levelCount: 1,
      baseArrayLayer: 0,
      layerCount: 1,
    };
    
    let image_view_create_info = vk::ImageViewCreateInfo {
      sType: vk::STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      image: *image,
      viewType: vk::IMAGE_VIEW_TYPE_2D,
      format: *format,
      components: component,
      subresourceRange: subresource,
    };
    
    unsafe {
      vk.CreateImageView(*device, &image_view_create_info, ptr::null(), &mut image_view);
    }
    
    image_view
  }
  
  fn create_texture_image(vk: &vk::DevicePointers, vk_instance: &vk::InstancePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, swapchain_format: &vk::Format, location: String) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
    let image = image::open(&location.clone()).expect(&("No file or Directory at: ".to_string() + &location)).to_rgba(); 
    let (width, height) = image.dimensions();
    let image_data = image.into_raw().clone();
    
    let image_size: vk::DeviceSize = (width * height * 4).into();
    
    let mut texture_image: vk::Image = unsafe { mem::uninitialized() };
    let mut texture_memory: vk::DeviceMemory = unsafe { mem::uninitialized() };
    let mut texture_image_view: vk::ImageView;
    
    Vulkan::create_image(vk, vk_instance, device, phys_device, vk::Extent2D { width: width, height: height }, swapchain_format, vk::IMAGE_TILING_OPTIMAL, vk::IMAGE_USAGE_TRANSFER_DST_BIT | vk::IMAGE_USAGE_SAMPLED_BIT, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT, &mut texture_image, &mut texture_memory);
    
    texture_image_view = Vulkan::create_image_view(vk, device, &texture_image, swapchain_format);
    
    (texture_image, texture_memory, texture_image_view)
  }
  
  fn create_image(vk: &vk::DevicePointers, vk_instance: &vk::InstancePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, image_extent: vk::Extent2D, format: &vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, properties: vk::MemoryPropertyFlags, image: &mut vk::Image, image_memory: &mut vk::DeviceMemory) {
    //
    // Start Create image
    //
    let image_create_info = {
      vk::ImageCreateInfo {
        sType: vk::STRUCTURE_TYPE_IMAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        imageType: vk::IMAGE_TYPE_2D,
        format: *format,
        extent: vk::Extent3D { width: image_extent.width, height: image_extent.height, depth: 1 },
        mipLevels: 1,
        arrayLayers: 1,
        samples: vk::SAMPLE_COUNT_1_BIT,
        tiling: tiling,
        usage: usage,
        sharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: ptr::null(),
        initialLayout: vk::IMAGE_LAYOUT_PREINITIALIZED,
      }
    };
    
   let mut memory_requirements: vk::MemoryRequirements = unsafe { mem::uninitialized() };
    
    unsafe {
      check_errors(vk.CreateImage(*device, &image_create_info, ptr::null(), image));
      vk.GetImageMemoryRequirements(*device, *image, &mut memory_requirements);
    }
    
    let memory_type_bits_index = {
      
      let mut memory_properties: vk::PhysicalDeviceMemoryProperties = unsafe { mem::uninitialized() };
      
      unsafe {
        vk_instance.GetPhysicalDeviceMemoryProperties(*phys_device, &mut memory_properties);
      }
      
      let mut index: i32 = -1;
      for i in 0..memory_properties.memoryTypeCount as usize {
        if memory_requirements.memoryTypeBits & (1 << i) != 0 && memory_properties.memoryTypes[i].propertyFlags & properties == properties {
          index = i as i32;
        }
      }
      
      if index == -1 {
        panic!("Failed to find suitable memory type");
      }
      
      index
    };
    
    let memory_allocate_info = {
      vk::MemoryAllocateInfo {
        sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
        pNext: ptr::null(),
        allocationSize: memory_requirements.size,
        memoryTypeIndex: memory_type_bits_index as u32,
      }
    };
    
    unsafe {
      check_errors(vk.AllocateMemory(*device, &memory_allocate_info, ptr::null(), image_memory));
      check_errors(vk.BindImageMemory(*device, *image, *image_memory, 0));
    }
  }
  
  fn create_pipelines(device: &Device, vertex_shader: &vk::ShaderModule, fragment_shader: &vk::ShaderModule, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D, swapchain_format: &vk::Format, descriptor_set: &DescriptorSet) -> (Vec<vk::Pipeline>, vk::PipelineCache, vk::PipelineLayout) {
    let mut pipelines: Vec<vk::Pipeline> = Vec::with_capacity(1);
    let mut pipeline_layout: vk::PipelineLayout = unsafe { mem::uninitialized() };
    let mut pipeline_cache: vk::PipelineCache = unsafe { mem::uninitialized() };
    
    let mut graphics_pipeline_create_infos: Vec<vk::GraphicsPipelineCreateInfo> = Vec::with_capacity(2);
    let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::with_capacity(2);
    let mut vertex_input_binding_descriptions: Vec<vk::VertexInputBindingDescription> = Vec::with_capacity(1);
    
    let topology = vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;
    let polygon_mode = vk::POLYGON_MODE_FILL;
    let enable_depth_clamp = vk::FALSE;
    let cull_mode =  vk::CULL_MODE_BACK_BIT;
    let front_face = vk::FRONT_FACE_CLOCKWISE;
    let depth_test = vk::FALSE;
    let depth_write = vk::FALSE;
    
    let blend_constants: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
    
    shader_stages.push(
      vk::PipelineShaderStageCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        stage: vk::SHADER_STAGE_VERTEX_BIT,
        module: *vertex_shader,
        pName: CString::new("main").unwrap().into_raw(),
        pSpecializationInfo: ptr::null(),
      }
    );
    
    shader_stages.push(
      vk::PipelineShaderStageCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        stage: vk::SHADER_STAGE_FRAGMENT_BIT,
        module: *fragment_shader,
        pName: CString::new("main").unwrap().into_raw(),
        pSpecializationInfo: ptr::null(),
      }
    );
    
    vertex_input_binding_descriptions.push(
      Vertex::vertex_input_binding()
    );
    
    /*
    float: VK_FORMAT_R32_SFLOAT
    vec2: VK_FORMAT_R32G32_SFLOAT
    vec3: VK_FORMAT_R32G32B32_SFLOAT
    vec4: VK_FORMAT_R32G32B32A32_SFLOAT
    ivec2: VK_FORMAT_R32G32_SINT
    uvec4: VK_FORMAT_R32G32B32A32_UINT
    double: VK_FORMAT_R64_SFLOAT
    */
    
    let mut vertex_binding: Vec<vk::VertexInputBindingDescription> = Vec::with_capacity(1);
    
    vertex_binding.push(
      Vertex::vertex_input_binding()
    );
    
    let mut vertex_input_attribute_descriptions = Vertex::vertex_input_attributes();
    
    let pipeline_vertex_input_state_create_info = {
      vk::PipelineVertexInputStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        vertexBindingDescriptionCount: vertex_binding.len() as u32,
        pVertexBindingDescriptions: vertex_binding.as_ptr(),
        vertexAttributeDescriptionCount: vertex_input_attribute_descriptions.len() as u32,
        pVertexAttributeDescriptions: vertex_input_attribute_descriptions.as_ptr(),
      }
    };
    
    let pipeline_input_assembly_state_create_info = {
      vk::PipelineInputAssemblyStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        topology: topology,
        primitiveRestartEnable: vk::FALSE,
      }
    };
    
    let pipeline_tessellation_state_create_info = {
      vk::PipelineTessellationStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        patchControlPoints: 0,
      }
    };
    
    let viewport = {
      vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swapchain_extent.width as f32,
        height: swapchain_extent.height as f32,
        minDepth: 0.0,
        maxDepth: 1.0,
      }
    };
    
    let scissor = {
      vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0,},
        extent: vk::Extent2D { width: swapchain_extent.width, height: swapchain_extent.height },
      }
    };
    
    let pipeline_viewport_state_create_info = {
      vk::PipelineViewportStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        viewportCount: 1,
        pViewports: &viewport,
        scissorCount: 1,
        pScissors: &scissor,
      }
    };
    
    let pipeline_rasterization_state_create_info = {
      vk::PipelineRasterizationStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        depthClampEnable: enable_depth_clamp,
        rasterizerDiscardEnable: vk::FALSE,
        polygonMode: polygon_mode,
        cullMode: cull_mode,
        frontFace: front_face,
        depthBiasEnable: vk::FALSE,
        depthBiasConstantFactor: 0.0,
        depthBiasClamp: 0.0,
        depthBiasSlopeFactor: 0.0,
        lineWidth: 1.0,
      }
    };
    
    let pipeline_multisample_state_create_info = {
      vk::PipelineMultisampleStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        rasterizationSamples: vk::SAMPLE_COUNT_1_BIT,
        sampleShadingEnable: vk::FALSE,
        minSampleShading: 1.0,
        pSampleMask: ptr::null(),
        alphaToCoverageEnable: vk::FALSE,
        alphaToOneEnable: vk::FALSE,
      }
    };
    
    let front_stencil_op_state = {
      vk::StencilOpState {
        failOp: vk::STENCIL_OP_KEEP,
        passOp: vk::STENCIL_OP_KEEP,
        depthFailOp: vk::STENCIL_OP_KEEP,
        compareOp: vk::COMPARE_OP_NEVER,
        compareMask: 0,
        writeMask: 0,
        reference: 0,
      }
    };
    
    let back_stencil_op_state = {
      vk::StencilOpState {
        failOp: vk::STENCIL_OP_KEEP,
        passOp: vk::STENCIL_OP_KEEP,
        depthFailOp: vk::STENCIL_OP_KEEP,
        compareOp: vk::COMPARE_OP_NEVER,
        compareMask: 0,
        writeMask: 0,
        reference: 0,
      }
    };
    
    let pipeline_depth_stencil_state_create_info = {
      vk::PipelineDepthStencilStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        depthTestEnable: depth_test,
        depthWriteEnable: depth_write,
        depthCompareOp: vk::COMPARE_OP_LESS_OR_EQUAL,
        depthBoundsTestEnable: vk::FALSE,
        stencilTestEnable: vk::FALSE,
        front: front_stencil_op_state,
        back: back_stencil_op_state,
        minDepthBounds: 0.0,
        maxDepthBounds: 1.0,
      }
    };
    
    let pipeline_color_blend_attachments = {
      vk::PipelineColorBlendAttachmentState {
        blendEnable: vk::FALSE,
        srcColorBlendFactor: vk::BLEND_FACTOR_ONE,
        dstColorBlendFactor: vk::BLEND_FACTOR_ZERO,
        colorBlendOp: vk::BLEND_OP_ADD,
        srcAlphaBlendFactor: vk::BLEND_FACTOR_ONE,
        dstAlphaBlendFactor: vk::BLEND_FACTOR_ZERO,
        alphaBlendOp: vk::BLEND_OP_ADD,
        colorWriteMask: vk::COLOR_COMPONENT_R_BIT | vk::COLOR_COMPONENT_G_BIT | vk::COLOR_COMPONENT_B_BIT | vk::COLOR_COMPONENT_A_BIT,
      }
    };
    
    let pipeline_colour_blend_state_create_info = {
      vk::PipelineColorBlendStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        logicOpEnable: vk::FALSE,
        logicOp: vk::LOGIC_OP_COPY,
        attachmentCount: 1,
        pAttachments: &pipeline_color_blend_attachments,
        blendConstants: blend_constants,
      }
    };
    
    let dynamic_state_create_info = {
      vk::PipelineDynamicStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        dynamicStateCount: 2,
        pDynamicStates: vec!(vk::DYNAMIC_STATE_VIEWPORT, vk::DYNAMIC_STATE_LINE_WIDTH).as_ptr(),
      }
    };
    
    let push_constant_range = {
      vk::PushConstantRange {
        stageFlags: vk::SHADER_STAGE_VERTEX_BIT,
        offset: 0,
        size: 0,
      }
    };
    
    let pipeline_layout_create_info = {
      vk::PipelineLayoutCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        setLayoutCount: 1,
        pSetLayouts: descriptor_set.layout(),
        pushConstantRangeCount: 0,
        pPushConstantRanges: ptr::null(),//&push_constant_range,
      }
    };
    
    let vk = device.pointers();
    let device = device.local_device();
    
    unsafe {
      vk.CreatePipelineLayout(*device, &pipeline_layout_create_info, ptr::null(), &mut pipeline_layout);
    }
    
    graphics_pipeline_create_infos.push(
      vk::GraphicsPipelineCreateInfo {
        sType: vk::STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        stageCount: shader_stages.len() as u32,
        pStages: shader_stages.as_ptr(),
        pVertexInputState: &pipeline_vertex_input_state_create_info,
        pInputAssemblyState: &pipeline_input_assembly_state_create_info,
        pTessellationState: &pipeline_tessellation_state_create_info,
        pViewportState: &pipeline_viewport_state_create_info,
        pRasterizationState: &pipeline_rasterization_state_create_info,
        pMultisampleState: &pipeline_multisample_state_create_info,
        pDepthStencilState: ptr::null(),//&pipeline_depth_stencil_state_create_info,
        pColorBlendState: &pipeline_colour_blend_state_create_info,
        pDynamicState: ptr::null(),//&dynamic_state_create_info,
        layout: pipeline_layout,
        renderPass: *render_pass,
        subpass: 0,
        basePipelineHandle: 0,
        basePipelineIndex: -1,
      }
    );
    
    let pipeline_cache_create_info = {
      vk::PipelineCacheCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        initialDataSize: 0,
        pInitialData: ptr::null(),
      }
    };
    
    unsafe {
      check_errors(vk.CreatePipelineCache(*device, &pipeline_cache_create_info, ptr::null(), &mut pipeline_cache));
      check_errors(vk.CreateGraphicsPipelines(*device, pipeline_cache, graphics_pipeline_create_infos.len() as u32, graphics_pipeline_create_infos.as_ptr(), ptr::null(), pipelines.as_mut_ptr()));
      pipelines.set_len(graphics_pipeline_create_infos.len());
    }
    
    (pipelines, pipeline_cache, pipeline_layout)
  }
  
  fn create_frame_buffers(device: &Device, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D, image_views: &Vec<vk::ImageView>) -> Vec<vk::Framebuffer> {
    let mut framebuffers: Vec<vk::Framebuffer> = Vec::with_capacity(image_views.len());
    
    for i in 0..image_views.len() {
      let mut framebuffer: vk::Framebuffer = unsafe { mem::uninitialized() };
      
      let framebuffer_create_info = vk::FramebufferCreateInfo {
        sType: vk::STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        renderPass: *render_pass,
        attachmentCount: 1,
        pAttachments: &image_views[i],
        width: swapchain_extent.width,
        height: swapchain_extent.height,
        layers: 1,
      };
      
      let vk = device.pointers();
      let device = device.local_device();
      
      unsafe {
        check_errors(vk.CreateFramebuffer(*device, &framebuffer_create_info, ptr::null(), &mut framebuffer));
      }
      
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  fn create_render_pass(device: &Device, format: &vk::Format) -> vk::RenderPass {
    
    let mut render_pass: vk::RenderPass = unsafe { mem::uninitialized() };
    
    let mut attachment_description = Vec::with_capacity(1);
    attachment_description.push(
      vk::AttachmentDescription {
        flags: 0,
        format: *format,
        samples: vk::SAMPLE_COUNT_1_BIT,
        loadOp: vk::ATTACHMENT_LOAD_OP_CLEAR,
        storeOp: vk::ATTACHMENT_STORE_OP_STORE,
        stencilLoadOp: vk::ATTACHMENT_LOAD_OP_DONT_CARE,
        stencilStoreOp: vk::ATTACHMENT_STORE_OP_DONT_CARE,
        initialLayout: vk::IMAGE_LAYOUT_UNDEFINED,
        finalLayout: vk::IMAGE_LAYOUT_PRESENT_SRC_KHR,
      }
    );
    
   // let mut input_attachments: Vec<vk::AttachmentReference>;
    let mut colour_attachments: Vec<vk::AttachmentReference> = Vec::new();
    //let mut resolve_attachmets: Vec<vk::AttachmentReference>;
    
    colour_attachments.push(
      vk::AttachmentReference {
        attachment: 0,
        layout: vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
      }
    );
    
    let mut subpass_description = Vec::with_capacity(1);
    subpass_description.push(
      vk::SubpassDescription {
        flags: 0,
        pipelineBindPoint: vk::PIPELINE_BIND_POINT_GRAPHICS,
        inputAttachmentCount: 0,//input_attachments.len() as u32,
        pInputAttachments: ptr::null(),//input_attachments,
        colorAttachmentCount: colour_attachments.len() as u32,
        pColorAttachments: colour_attachments.as_ptr(),
        pResolveAttachments: ptr::null(),//resolve_attachmets.len() as u32,
        pDepthStencilAttachment: ptr::null(),//resolve_attachmets,
        preserveAttachmentCount: 0,
        pPreserveAttachments: ptr::null(),
      }
    );
    
    let mut subpass_dependency: Vec<vk::SubpassDependency> = Vec::with_capacity(2);
    
    subpass_dependency.push(vk::SubpassDependency {
      srcSubpass: vk::SUBPASS_EXTERNAL,
      dstSubpass: 0,
      srcStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
      dstStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
      srcAccessMask: 0,
      dstAccessMask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
      dependencyFlags: vk::DEPENDENCY_BY_REGION_BIT,
    });
    /*
    subpass_dependency.push(vk::SubpassDependency {
      srcSubpass: 0,
      dstSubpass: vk::SUBPASS_EXTERNAL,
      srcStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
      dstStageMask: vk::PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
      srcAccessMask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
      dstAccessMask: 0,//vk::ACCESS_MEMORY_READ_BIT,
      dependencyFlags: vk::DEPENDENCY_BY_REGION_BIT,
    });*/
    
    let render_pass_create_info = vk::RenderPassCreateInfo {
      sType: vk::STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      attachmentCount: attachment_description.len() as u32,
      pAttachments: attachment_description.as_ptr(),
      subpassCount: subpass_description.len() as u32,
      pSubpasses: subpass_description.as_ptr(),
      dependencyCount: subpass_dependency.len() as u32,
      pDependencies: subpass_dependency.as_ptr(),
    };
    
    let vk = device.pointers();
    let device = device.local_device();
    
    unsafe {
      vk.CreateRenderPass(*device, &render_pass_create_info, ptr::null(), &mut render_pass);
    }
    
    render_pass
  }
  
  fn create_semaphores(device: &Device) -> (vk::Semaphore, vk::Semaphore) {
    let mut semaphore_image_available: vk::Semaphore = unsafe { mem::uninitialized() };
    let mut semaphore_render_finished: vk::Semaphore = unsafe { mem::uninitialized() };
    
    let semaphore_info = vk::SemaphoreCreateInfo {
      sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.CreateSemaphore(*device, &semaphore_info, ptr::null(), &mut semaphore_image_available));
      check_errors(vk.CreateSemaphore(*device, &semaphore_info, ptr::null(), &mut semaphore_render_finished));
    }
    
    (semaphore_image_available, semaphore_render_finished)
  }
  
  fn create_fences(device: &Device, num_fences: u32) -> Vec<vk::Fence> {
    let mut fences: Vec<vk::Fence> = Vec::with_capacity(num_fences as usize);
    
    let fence_info = vk::FenceCreateInfo {
      sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
      pNext: ptr::null(),
      flags: vk::FENCE_CREATE_SIGNALED_BIT,
    };
    
    let vk = device.pointers();
    let device = device.local_device();
    
    for i in 0..num_fences {
      let mut fence: vk::Fence = unsafe { mem::uninitialized() };
      unsafe {
        check_errors(vk.CreateFence(*device, &fence_info, ptr::null(), &mut fence));
      }
      fences.push(fence);
    }
    
    fences
  }
}

impl Drop for Vulkan {
  fn drop(&mut self) {
    let the_device = self.window.device();
    let device = the_device.local_device();
    let vk = self.window.device_pointers();
    unsafe {
      the_device.wait();
      
      println!("Destroying Fences");
      for fence in &self.fences {
        check_errors(vk.WaitForFences(*device, 1, fence, vk::TRUE, u64::max_value()));
        vk.DestroyFence(*device, *fence, ptr::null());
      }
      
      println!("Destroying buffers");
      vk.FreeMemory(*device, self.uniform_buffer_memory, ptr::null());
      vk.DestroyBuffer(*device, self.uniform_buffer, ptr::null());
      
      vk.FreeMemory(*device, self.index_buffer_memory, ptr::null());
      vk.DestroyBuffer(*device, self.index_buffer, ptr::null());
      
      vk.FreeMemory(*device, self.vertex_buffer_memory, ptr::null());
      vk.DestroyBuffer(*device, self.vertex_buffer, ptr::null());
      /*
      vk.DestroySampler(*device, self.texture_sampler, ptr::null());
      vk.DestroyImageView(*device, self.texture_image_view, ptr::null());
      vk.FreeMemory(*device, self.texture_image_memory, ptr::null());
      vk.DestroyImage(*device, self.texture_image, ptr::null());*/
      
      println!("Destroying Pipeline");
      vk.DestroyPipelineLayout(*device, self.pipeline_layout, ptr::null());
      vk.DestroyPipelineCache(*device, self.pipeline_cache, ptr::null());
      
      for pipeline in &self.pipelines {
        vk.DestroyPipeline(*device, *pipeline, ptr::null());
      }
      
      self.descriptor_set.destroy(the_device);
      self.descriptor_set_pool.destroy(the_device);
      
      self.vertex_shader.destroy(the_device);
      self.fragment_shader.destroy(the_device);
      
      for framebuffer in &self.framebuffers {
        vk.DestroyFramebuffer(*device, *framebuffer, ptr::null());
      }
      vk.DestroyRenderPass(*device, self.render_pass, ptr::null());
      vk.FreeCommandBuffers(*device, *self.command_pool.local_command_pool(), self.command_buffers.len() as u32, self.command_buffers.as_mut_ptr());
      self.command_pool.destroy(self.window.device());
      vk.DestroySemaphore(*device, self.semaphore_image_available, ptr::null());
      vk.DestroySemaphore(*device, self.semaphore_render_finished, ptr::null());
    }
  }
}
