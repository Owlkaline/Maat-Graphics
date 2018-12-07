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
      inputRate: vk::VERTEX_INPUT_RATE_VERTEX+600,
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
  fences: Vec<vk::Fence>,
  semaphore_image_available: vk::Semaphore,
  semaphore_render_finished: vk::Semaphore,
  command_pool: vk::CommandPool,
  command_buffers: Vec<vk::CommandBuffer>,
  render_pass: vk::RenderPass,
  framebuffers: Vec<vk::Framebuffer>,
  vertex_shader: vk::ShaderModule,
  fragment_shader: vk::ShaderModule,
  descriptor_set_layout: vk::DescriptorSetLayout,
  descriptor_set_pool: vk::DescriptorPool,
  descriptor_sets: Vec<vk::DescriptorSet>,
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
    let command_pool: vk::CommandPool;
    let command_buffers: Vec<vk::CommandBuffer>;
    let render_pass: vk::RenderPass;
    let framebuffers: Vec<vk::Framebuffer>;
    let vertex_shader: vk::ShaderModule;
    let fragment_shader: vk::ShaderModule;
    let descriptor_set_layout: vk::DescriptorSetLayout;
    let descriptor_set_pool: vk::DescriptorPool;
    let descriptor_sets: Vec<vk::DescriptorSet>;
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
    
    {
      let vk = window.device_pointers();
      let vk_instance = window.instance_pointers();
      let device = window.device();
      let format = window.swapchain_format();
      let graphics_family = window.get_graphics_family();
      let graphics_queue = window.get_graphics_queue();
      let current_extent = window.get_current_extent();
      let image_views = window.swapchain_image_views();
      let phys_device = window.physical_device();
      
      let (semaphore1, semaphore2) = Vulkan::create_semaphores(vk, device);
      semaphore_image_available = semaphore1;
      semaphore_render_finished = semaphore2;
      render_pass = Vulkan::create_render_pass(vk, device, &format);
      framebuffers = Vulkan::create_frame_buffers(vk, device, &render_pass, &current_extent, image_views);
      fences = Vulkan::create_fences(vk, device, framebuffers.len() as u32);
      command_pool = Vulkan::create_command_pool(vk, device, graphics_family);
      command_buffers = Vulkan::create_command_buffers(vk, device, &command_pool, framebuffers.len() as u32);
      
      let (vshader, fshader) = Vulkan::create_shaders(vk, device);
      vertex_shader = vshader;
      fragment_shader = fshader;
      
      descriptor_set_layout = Vulkan::create_descriptor_set_layout(vk, device);
      descriptor_set_pool = Vulkan::create_descriptor_pool(vk, device);
      descriptor_sets = Vulkan::create_descriptor_sets(vk, device, &descriptor_set_layout, &descriptor_set_pool);
      
      let (pipeline, cache, layout) = Vulkan::create_pipelines(vk, device, &vertex_shader, &fragment_shader, &render_pass, &current_extent, &format, &descriptor_set_layout);
      pipelines = pipeline;
      pipeline_cache = cache;
      pipeline_layout = layout;
      /*
      let (texture, texture_memory, texture_view) = Vulkan::create_texture_image(vk, vk_instance, device, phys_device, &format, "./src/shaders/statue.jpg".to_string());
      texture_image = texture;
      texture_image_memory = texture_memory;
      texture_image_view = texture_view;
      
      texture_sampler = Vulkan::create_texture_sampler(vk, device);*/
      
      let (vertex, vertex_memory) = Vulkan::create_vertex_buffer(vk, vk_instance, device, phys_device, &command_pool, graphics_queue);
      vertex_buffer = vertex;
      vertex_buffer_memory = vertex_memory;
      
      let (index, index_memory) = Vulkan::create_index_buffer(vk, vk_instance, device, phys_device, &command_pool, graphics_queue);
      index_buffer = index;
      index_buffer_memory = index_memory;
      
      let (uniform, uniform_memory) = Vulkan::create_uniform_buffer(vk, vk_instance, device, phys_device, current_extent, &descriptor_sets[0]);
      uniform_buffer = uniform;
      uniform_buffer_memory = uniform_memory;
    }
    
    Vulkan {
      window: window,
      fences: fences,
      semaphore_image_available: semaphore_image_available,
      semaphore_render_finished: semaphore_render_finished,
      command_pool: command_pool,
      command_buffers: command_buffers,
      render_pass: render_pass,
      framebuffers: framebuffers,
      vertex_shader: vertex_shader,
      fragment_shader: fragment_shader,
      descriptor_set_layout: descriptor_set_layout,
      descriptor_set_pool: descriptor_set_pool,
      descriptor_sets: descriptor_sets,
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
        /*
        let viewport = {
          vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: window_size.width as f32,
            height: window_size.height as f32,
            minDepth: 0.0,
            maxDepth: 1.0,
          }
        };
        vk.CmdSetViewport(self.command_buffers[i], 0, 1, &viewport);
        
        let scissor = {
          vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: window_size.width, height: window_size.height },
          }
        };
        vk.CmdSetScissor(self.command_buffers[i], 0, 1, &scissor);
        */
      //  vk.CmdBindDescriptorSets(self.command_buffers[i], vk::PIPELINE_BIND_POINT_GRAPHICS, self.pipeline_layout, 0, 1, &self.descriptor_sets[0], 0, ptr::null());
        vk.CmdBindPipeline(self.command_buffers[i], vk::PIPELINE_BIND_POINT_GRAPHICS, self.pipelines[0]);
        vk.CmdBindVertexBuffers(self.command_buffers[i], 0, 1, &self.vertex_buffer, &0);
       // vk.CmdBindIndexBuffer(self.command_buffers[i], self.index_buffer, 0, vk::INDEX_TYPE_UINT32);
        let indices_count = 3;
        vk.CmdDraw(self.command_buffers[i], 3, 1, 0, 1);
        vk.CmdEndRenderPass(self.command_buffers[i]);
        
        check_errors(vk.EndCommandBuffer(self.command_buffers[i]));
      }
    }
  }
  
  pub fn draw(&mut self) {
    let vk = self.window.device_pointers();
    let device = self.window.device();
    let swapchain = self.window.get_swapchain();
    let graphics_queue = self.window.get_graphics_queue();
    let present_queue = self.window.get_present_queue();
    
    let mut current_buffer = 0;
    unsafe {
      check_errors(vk.AcquireNextImageKHR(*device, *swapchain, 0, self.semaphore_image_available, 0, &mut current_buffer));
      println!("wait for fences");
      check_errors(vk.WaitForFences(*device, 1, &self.fences[current_buffer as usize], vk::TRUE, u64::max_value()));
      check_errors(vk.ResetFences(*device, 1, &self.fences[current_buffer as usize]));
      println!("reset fences");
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
    println!("There");
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
    println!("There1");
    unsafe {
      check_errors(vk.QueuePresentKHR(*graphics_queue, &present_info_khr));
    //  vk.DeviceWaitIdle(*device);
      vk.QueueWaitIdle(*graphics_queue);
    }
    
    println!("here");
  }
  
  pub fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn begin_single_time_command(vk: &vk::DevicePointers, device: &vk::Device, command_pool: &vk::CommandPool) -> vk::CommandBuffer {
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
      check_errors(vk.AllocateCommandBuffers(*device, &command_buffer_allocate_info, &mut command_buffer));
      check_errors(vk.BeginCommandBuffer(command_buffer, &command_buffer_begin_info));
    }
    
    command_buffer
  }
  
  fn end_single_time_command(vk: &vk::DevicePointers, device: &vk::Device, command_buffer: vk::CommandBuffer, command_pool: &vk::CommandPool, graphics_queue: &vk::Queue) {
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
      vk.EndCommandBuffer(command_buffer);
      vk.QueueSubmit(*graphics_queue, 1, &submit_info, 0);
      vk.QueueWaitIdle(*graphics_queue);
      vk.FreeCommandBuffers(*device, *command_pool, 1, &command_buffer);
    }
  }
  
  fn create_uniform_buffer(vk: &vk::DevicePointers, vk_instance: &vk::InstancePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, swapchain_extent: vk::Extent2D, descriptor_set: &vk::DescriptorSet) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_size: vk::DeviceSize = (mem::size_of::<f32>()*48) as u64;
    
    let (uniform_buffer, uniform_buffer_memory) = Vulkan::create_buffer(vk, vk_instance, device, phys_device, buffer_size, vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);
    
    let descriptor_buffer_info = {
      vk::DescriptorBufferInfo {
        buffer: uniform_buffer,
        offset: 0,
        range: (mem::size_of::<f32>()*48) as u64,
      }
    };
    
    let write_descriptor_set = {
      vk::WriteDescriptorSet {
        sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
        pNext: ptr::null(),
        dstSet: *descriptor_set,
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
      vk.UpdateDescriptorSets(*device, 1, &write_descriptor_set, 0, ptr::null());
    }
     
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
    
    let mut data = unsafe { mem::uninitialized() };
    unsafe {
      check_errors(vk.MapMemory(*device, uniform_buffer_memory, 0, (mem::size_of::<f32>()*48) as u64, 0, &mut data));
      memcpy(data, real_data.as_ptr() as *const _, (mem::size_of::<f32>() * 48));
      vk.UnmapMemory(*device, uniform_buffer_memory);
    }
    
    (uniform_buffer, uniform_buffer_memory)
  }
  
  fn create_index_buffer(vk: &vk::DevicePointers, vk_instance: &vk::InstancePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, command_pool: &vk::CommandPool, graphics_queue: &vk::Queue) -> (vk::Buffer, vk::DeviceMemory) {
    let indices = [
      0, 1, 2
    ];
    
    let mut buffer_size: vk::DeviceSize = (mem::size_of::<[f32; 3]>()) as u64;
    
    let (staging_index_buffer, staging_index_buffer_memory) = Vulkan::create_buffer(vk, vk_instance, device, phys_device, buffer_size, vk::BUFFER_USAGE_TRANSFER_SRC_BIT, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);
    
    let mut host_visible_data = unsafe { mem::uninitialized() };
    
    unsafe {
      check_errors(vk.MapMemory(*device, staging_index_buffer_memory, 0, buffer_size, 0, &mut host_visible_data));
      memcpy(host_visible_data, indices.as_ptr() as *const _, buffer_size as usize);
      vk.UnmapMemory(*device, staging_index_buffer_memory);
    }
    
    let (index_buffer, index_buffer_memory) = Vulkan::create_buffer(vk, vk_instance, device, phys_device, buffer_size, vk::BUFFER_USAGE_INDEX_BUFFER_BIT | vk::BUFFER_USAGE_TRANSFER_DST_BIT, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    
    let command_buffer = Vulkan::begin_single_time_command(vk, device, command_pool);
    
    let buffer_copy = {
      vk::BufferCopy {
        srcOffset: 0,
        dstOffset: 0,
        size: buffer_size,
      }
    };
    
    unsafe {
      vk.CmdCopyBuffer(command_buffer, staging_index_buffer, index_buffer, 1, &buffer_copy);
    }
    
    Vulkan::end_single_time_command(vk, device, command_buffer, command_pool, graphics_queue);
    
    unsafe {
      vk.FreeMemory(*device, staging_index_buffer_memory, ptr::null());
      vk.DestroyBuffer(*device, staging_index_buffer, ptr::null());
    }
    
    (index_buffer, index_buffer_memory)
  }
  
  fn create_vertex_buffer(vk: &vk::DevicePointers, vk_instance: &vk::InstancePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, command_pool: &vk::CommandPool, graphics_queue: &vk::Queue) -> (vk::Buffer, vk::DeviceMemory) {
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
    
    let (staging_vertex_buffer, staging_vertex_buffer_memory) = Vulkan::create_buffer(vk, vk_instance, device, phys_device, buffer_size, vk::BUFFER_USAGE_TRANSFER_SRC_BIT, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);
    
    let mut host_visible_data = unsafe { mem::uninitialized() };
    
    unsafe {
      check_errors(vk.MapMemory(*device, staging_vertex_buffer_memory, 0, buffer_size, 0, &mut host_visible_data));
      memcpy(host_visible_data, triangle.as_ptr() as *const _, buffer_size as usize);
      vk.UnmapMemory(*device, staging_vertex_buffer_memory);
    }
    
    let (vertex_buffer, vertex_buffer_memory) = Vulkan::create_buffer(vk, vk_instance, device, phys_device, buffer_size, vk::BUFFER_USAGE_VERTEX_BUFFER_BIT | vk::BUFFER_USAGE_TRANSFER_DST_BIT, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    
    let command_buffer = Vulkan::begin_single_time_command(vk, device, command_pool);
    
    let buffer_copy = {
      vk::BufferCopy {
        srcOffset: 0,
        dstOffset: 0,
        size: buffer_size,
      }
    };
    
    unsafe {
      vk.CmdCopyBuffer(command_buffer, staging_vertex_buffer, vertex_buffer, 1, &buffer_copy);
    }
    
    Vulkan::end_single_time_command(vk, device, command_buffer, command_pool, graphics_queue);
    
    unsafe {
      vk.FreeMemory(*device, staging_vertex_buffer_memory, ptr::null());
      vk.DestroyBuffer(*device, staging_vertex_buffer, ptr::null());
    }
    
    (vertex_buffer, vertex_buffer_memory)
  }
  
  fn create_buffer(vk: &vk::DevicePointers, vk_instance: &vk::InstancePointers,  device: &vk::Device, phys_device: &vk::PhysicalDevice, buffer_size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory) {
    
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
      check_errors(vk.CreateBuffer(*device, &buffer_create_info, ptr::null(), &mut buffer));
      vk.GetBufferMemoryRequirements(*device, buffer, &mut memory_requirements);
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
  
  fn create_pipelines(vk: &vk::DevicePointers, device: &vk::Device, vertex_shader: &vk::ShaderModule, fragment_shader: &vk::ShaderModule, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D, swapchain_format: &vk::Format, descriptor_set_layout: &vk::DescriptorSetLayout) -> (Vec<vk::Pipeline>, vk::PipelineCache, vk::PipelineLayout) {
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
      vk::VertexInputBindingDescription {
        binding: 0,
        stride: (mem::size_of::<f32>()*6) as u32,
        inputRate: vk::VERTEX_INPUT_RATE_VERTEX,
      }
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
      vk::VertexInputBindingDescription {
        binding: 0,
        stride: (mem::size_of::<Vertex>()) as u32,
        inputRate: vk::VERTEX_INPUT_RATE_VERTEX,
      }
    );
    
    let pipeline_vertex_input_state_create_info = {
      vk::PipelineVertexInputStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        vertexBindingDescriptionCount: vertex_binding.len() as u32,//vertex_input_binding_descriptions.len() as u32,
        pVertexBindingDescriptions: vertex_binding.as_ptr(),//vertex_input_binding_descriptions.as_ptr(),
        vertexAttributeDescriptionCount: 2,//vertex_input_attribute_descriptions.len() as u32,
        pVertexAttributeDescriptions: &Vertex::vertex_input_attributes()[0],//vertex_input_attribute_descriptions.as_ptr(),
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
        pSetLayouts: descriptor_set_layout,
        pushConstantRangeCount: 0,
        pPushConstantRanges: ptr::null(),//&push_constant_range,
      }
    };
    
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
  
  fn create_descriptor_sets(vk: &vk::DevicePointers, device: &vk::Device, descriptor_set_layout: &vk::DescriptorSetLayout, descriptor_set_pool: &vk::DescriptorPool) -> Vec<vk::DescriptorSet> {
    let mut descriptor_sets: Vec<vk::DescriptorSet> = Vec::with_capacity(1);
    
    let descriptor_set_allocate_info = {
      vk::DescriptorSetAllocateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
        pNext: ptr::null(),
        descriptorPool: *descriptor_set_pool,
        descriptorSetCount: 1,
        pSetLayouts: descriptor_set_layout,
      }
    };
    
    unsafe {
      check_errors(vk.AllocateDescriptorSets(*device, &descriptor_set_allocate_info, descriptor_sets.as_mut_ptr()));
      descriptor_sets.set_len(1);
    }
    
    descriptor_sets
  }
  
  fn create_descriptor_pool(vk: &vk::DevicePointers, device: &vk::Device) -> vk::DescriptorPool {
    let mut descriptor_pool: vk::DescriptorPool = unsafe { mem::uninitialized() };
    let mut descriptor_pool_size: Vec<vk::DescriptorPoolSize> = Vec::with_capacity(1);
    
    descriptor_pool_size.push(
      vk::DescriptorPoolSize {
        ty: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        descriptorCount: 1,
      }
    );
    /*
    descriptor_pool_size.push(
      vk::DescriptorPoolSize {
        ty: vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
        descriptorCount: 1,
      }
    );*/
    
    let descriptor_pool_create_info = {
      vk::DescriptorPoolCreateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        maxSets: 1,
        poolSizeCount: descriptor_pool_size.len() as u32,
        pPoolSizes: descriptor_pool_size.as_ptr(),
      }
    };
    
    unsafe {
      check_errors(vk.CreateDescriptorPool(*device, &descriptor_pool_create_info, ptr::null(), &mut descriptor_pool));
    }
    
    descriptor_pool
  }
  
  fn create_descriptor_set_layout(vk: &vk::DevicePointers, device: &vk::Device) -> vk::DescriptorSetLayout {
    let mut descriptor_set_layout: vk::DescriptorSetLayout = unsafe { mem::uninitialized() };
    let mut descriptor_bindings: Vec<vk::DescriptorSetLayoutBinding> = Vec::with_capacity(1);
    
    descriptor_bindings.push(
      vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptorType: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        descriptorCount: 1,
        stageFlags: vk::SHADER_STAGE_VERTEX_BIT,
        pImmutableSamplers: ptr::null(),
      }
    );
    /*
    descriptor_bindings.push(
      vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptorType: vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
        descriptorCount: 1,
        stageFlags: vk::SHADER_STAGE_FRAGMENT_BIT,
        pImmutableSamplers: ptr::null(),
      }
    );
    */
    
    let descriptor_set_layout_create_info = {
      vk::DescriptorSetLayoutCreateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        bindingCount: 1,
        pBindings: descriptor_bindings.as_ptr(),
      }
    };
    
    unsafe {
      vk.CreateDescriptorSetLayout(*device, &descriptor_set_layout_create_info, ptr::null(), &mut descriptor_set_layout);
    }
    
    descriptor_set_layout
  }
  
  fn create_shaders(vk: &vk::DevicePointers, device: &vk::Device) -> (vk::ShaderModule, vk::ShaderModule) {
    let vertex_shader_data = include_bytes!("../shaders/test_vert.spv");
    let fragment_shader_data = include_bytes!("../shaders/test_frag.spv");
    
    let mut shader_module_vertex: vk::ShaderModule = unsafe { mem::uninitialized() };
    let mut shader_module_fragment: vk::ShaderModule = unsafe { mem::uninitialized() };
    
    let mut vertex_code_size = mem::size_of::<u8>() * vertex_shader_data.len();
    let mut fragment_code_size = mem::size_of::<u8>() * fragment_shader_data.len();
    /*
    let mut multiple_of_4 = false;
    while !multiple_of_4 {
      if vertex_code_size % 4 == 0 {
        break;
      }
      vertex_code_size += 1;
    }
    
    multiple_of_4 = false;
    while !multiple_of_4 {
      if fragment_code_size % 4 == 0 {
        break;
      }
      fragment_code_size += 1;
    }*/
    
    let mut vertex_shader_module_create_info = vk::ShaderModuleCreateInfo {
      sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      codeSize: vertex_code_size,
      pCode: vertex_shader_data.as_ptr() as *const _,
    };
    
    let mut fragment_shader_module_create_info = vk::ShaderModuleCreateInfo {
      sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      codeSize: fragment_code_size,
      pCode: fragment_shader_data.as_ptr() as *const _,
    };
    
    unsafe {
      vk.CreateShaderModule(*device, &vertex_shader_module_create_info, ptr::null(), &mut shader_module_vertex);
      vk.CreateShaderModule(*device, &fragment_shader_module_create_info, ptr::null(), &mut shader_module_fragment);
    }
    
    (shader_module_vertex, shader_module_fragment)
  }
  
  fn create_frame_buffers(vk: &vk::DevicePointers, device: &vk::Device, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D, image_views: &Vec<vk::ImageView>) -> Vec<vk::Framebuffer> {
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
      
      unsafe {
        check_errors(vk.CreateFramebuffer(*device, &framebuffer_create_info, ptr::null(), &mut framebuffer));
      }
      
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  fn create_render_pass(vk: &vk::DevicePointers, device: &vk::Device, format: &vk::Format) -> vk::RenderPass {
    
    let mut render_pass: vk::RenderPass = unsafe { mem::uninitialized() };
    
    let attachment_description = vk::AttachmentDescription {
      flags: 0,
      format: *format,
      samples: vk::SAMPLE_COUNT_1_BIT,
      loadOp: vk::ATTACHMENT_LOAD_OP_CLEAR,
      storeOp: vk::ATTACHMENT_STORE_OP_STORE,
      stencilLoadOp: vk::ATTACHMENT_LOAD_OP_DONT_CARE,
      stencilStoreOp: vk::ATTACHMENT_STORE_OP_DONT_CARE,
      initialLayout: vk::IMAGE_LAYOUT_UNDEFINED,
      finalLayout: vk::IMAGE_LAYOUT_PRESENT_SRC_KHR,
    };
    
   // let mut input_attachments: Vec<vk::AttachmentReference>;
    let mut colour_attachments: Vec<vk::AttachmentReference> = Vec::new();
    //let mut resolve_attachmets: Vec<vk::AttachmentReference>;
    
    colour_attachments.push(
      vk::AttachmentReference {
        attachment: 0,
        layout: vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
      }
    );
    
    let subpass_description = vk::SubpassDescription {
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
    };
    
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
      attachmentCount: 1,
      pAttachments: &attachment_description,
      subpassCount: 1,
      pSubpasses: &subpass_description,
      dependencyCount: subpass_dependency.len() as u32,
      pDependencies: subpass_dependency.as_ptr(),
    };
    
    unsafe {
      vk.CreateRenderPass(*device, &render_pass_create_info, ptr::null(), &mut render_pass);
    }
    
    render_pass
  }
  
  fn create_command_buffers(vk: &vk::DevicePointers, device: &vk::Device, command_pool: &vk::CommandPool, num_command_command_buffers: u32) -> Vec<vk::CommandBuffer> {
    let mut command_buffers: Vec<vk::CommandBuffer> = Vec::with_capacity(num_command_command_buffers as usize);
    
    let allocate_command_buffer_info = vk::CommandBufferAllocateInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
      pNext: ptr::null(),
      commandPool: *command_pool,
      level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
      commandBufferCount: num_command_command_buffers,
    };
    
    unsafe {
      check_errors(vk.AllocateCommandBuffers(*device, &allocate_command_buffer_info, command_buffers.as_mut_ptr()));
      command_buffers.set_len(num_command_command_buffers as usize);
    }
    
    command_buffers
  }
  
  fn create_command_pool(vk: &vk::DevicePointers, device: &vk::Device, graphics_family: u32) -> vk::CommandPool {
    let mut command_pool: vk::CommandPool = unsafe { mem::uninitialized() };
    
    let command_pool_info = vk::CommandPoolCreateInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
      pNext: ptr::null(),
      flags: vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,//vk::COMMAND_POOL_CREATE_TRANSIENT_BIT, //to use vkResetCommandBuffer change to vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT
      queueFamilyIndex: graphics_family,
    };
    
    unsafe {
      check_errors(vk.CreateCommandPool(*device, &command_pool_info, ptr::null(), &mut command_pool));
    }
    
    command_pool
  }
  
  fn create_semaphores(vk: &vk::DevicePointers, device: &vk::Device) -> (vk::Semaphore, vk::Semaphore) {
    let mut semaphore_image_available: vk::Semaphore = unsafe { mem::uninitialized() };
    let mut semaphore_render_finished: vk::Semaphore = unsafe { mem::uninitialized() };
    
    let semaphore_info = vk::SemaphoreCreateInfo {
      sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
    };
    
    unsafe {
      check_errors(vk.CreateSemaphore(*device, &semaphore_info, ptr::null(), &mut semaphore_image_available));
      check_errors(vk.CreateSemaphore(*device, &semaphore_info, ptr::null(), &mut semaphore_render_finished));
    }
    
    (semaphore_image_available, semaphore_render_finished)
  }
  
  fn create_fences(vk: &vk::DevicePointers, device: &vk::Device, num_fences: u32) -> Vec<vk::Fence> {
    let mut fences: Vec<vk::Fence> = Vec::with_capacity(num_fences as usize);
    
    let fence_info = vk::FenceCreateInfo {
      sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
      pNext: ptr::null(),
      flags: vk::FENCE_CREATE_SIGNALED_BIT,
    };
    
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
    let device = self.window.device();
    let vk = self.window.device_pointers();
    println!("Destroying Command pool, semaphores and fences");
    unsafe {
      vk.DeviceWaitIdle(*device);
      
      for fence in &self.fences {
        check_errors(vk.WaitForFences(*device, 1, fence, vk::TRUE, u64::max_value()));
        vk.DestroyFence(*device, *fence, ptr::null());
      }
      
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
      vk.DestroyPipelineLayout(*device, self.pipeline_layout, ptr::null());
      vk.DestroyPipelineCache(*device, self.pipeline_cache, ptr::null());
      
      for pipeline in &self.pipelines {
        vk.DestroyPipeline(*device, *pipeline, ptr::null());
      }
      
      vk.DestroyDescriptorPool(*device, self.descriptor_set_pool, ptr::null());
      vk.DestroyDescriptorSetLayout(*device, self.descriptor_set_layout, ptr::null());
      
      vk.DestroyShaderModule(*device, self.vertex_shader, ptr::null());
      vk.DestroyShaderModule(*device, self.fragment_shader, ptr::null());
      
      for framebuffer in &self.framebuffers {
        vk.DestroyFramebuffer(*device, *framebuffer, ptr::null());
      }
      vk.DestroyRenderPass(*device, self.render_pass, ptr::null());
      vk.FreeCommandBuffers(*device, self.command_pool, self.command_buffers.len() as u32, self.command_buffers.as_mut_ptr());
      vk.DestroyCommandPool(*device, self.command_pool, ptr::null());
      vk.DestroySemaphore(*device, self.semaphore_image_available, ptr::null());
      vk.DestroySemaphore(*device, self.semaphore_render_finished, ptr::null());
    }
  }
}
