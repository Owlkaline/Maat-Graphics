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

use modules::VkWindow;
use ownage::check_errors;


use std::ptr;
use std::mem;

pub struct Vulkan {
  window: VkWindow,
  fence: vk::Fence,
  semaphore: vk::Semaphore,
  command_pool: vk::CommandPool,
  command_buffers: Vec<vk::CommandBuffer>,
}

impl Vulkan {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> Vulkan {
    let window = VkWindow::new(app_name, app_version, width, height, should_debug);
    
    let fence: vk::Fence;
    let semaphore: vk::Semaphore;
    let command_pool: vk::CommandPool;
    let command_buffers: Vec<vk::CommandBuffer>;
    
    {
      let vk = window.device_pointers();
      let device = window.device();
      let graphics_family = window.get_graphics_family();
      
      fence = Vulkan::create_fence(vk, device);
      semaphore = Vulkan::create_semaphore(vk, device);
      command_pool = Vulkan::create_command_pool(vk, device, graphics_family);
      command_buffers = Vulkan::create_command_buffers(vk, device, &command_pool, 1);
    }
    
    Vulkan {
      window: window,
      fence: fence,
      semaphore: semaphore,
      command_pool: command_pool,
      command_buffers: command_buffers,
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
  
  pub fn draw(&mut self) {
    let vk = self.window.device_pointers();
    
    let command_buffer_begin_info = vk::CommandBufferBeginInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      pNext: ptr::null(),
      flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
      pInheritanceInfo: ptr::null(),
    };
    
    unsafe {
      check_errors(vk.BeginCommandBuffer(self.command_buffers[0], &command_buffer_begin_info));
      
      // Add special handles or error codes for endcommandbuffer issues
      check_errors(vk.EndCommandBuffer(self.command_buffers[0])); 
      /*
      let mut submit_info: Vec<vk::SubmitInfo> = Vec::new();
      submit_info.push(vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 1,
        pWaitSemaphores: &self.semaphore,
        pWaitDstStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        commandBufferCount: 1,
        pCommandBuffers: &self.command_buffers[0],
        signalSemaphoreCount: 1,
        pSignalSemaphores: &self.semaphore,
      });
      
      check_errors(vk::QueueSubmit( , 1, &submit_info, self.fence));*/
      
      check_errors(vk.ResetCommandBuffer(self.command_buffers[0], vk::COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT));
    }
  }
  
  pub fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
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
  
  fn create_semaphore(vk: &vk::DevicePointers, device: &vk::Device) -> vk::Semaphore {
    let mut semaphore: vk::Semaphore = unsafe { mem::uninitialized() };
    
    let semaphore_info = vk::SemaphoreCreateInfo {
      sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
    };
    
    unsafe {
      check_errors(vk.CreateSemaphore(*device, &semaphore_info, ptr::null(), &mut semaphore));
    }
    
    semaphore
  }
  
  fn create_fence(vk: &vk::DevicePointers, device: &vk::Device) -> vk::Fence {
    let mut fence: vk::Fence = unsafe { mem::uninitialized() };
    
    let fence_info = vk::FenceCreateInfo {
      sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
    };
    
    unsafe {
      check_errors(vk.CreateFence(*device, &fence_info, ptr::null(), &mut fence));
    }
    
    fence
  }
}

impl Drop for Vulkan {
  fn drop(&mut self) {
    unsafe {
      let device = self.window.device();
      let vk = self.window.device_pointers();
      println!("Destroying Command pool, semaphores and fences");
      unsafe {
        vk.FreeCommandBuffers(*device, self.command_pool, self.command_buffers.len() as u32, self.command_buffers.as_mut_ptr());
        vk.DestroyCommandPool(*device, self.command_pool, ptr::null());
        vk.DestroySemaphore(*device, self.semaphore, ptr::null());
        vk.DestroyFence(*device, self.fence, ptr::null());
      }
    }
  }
}
