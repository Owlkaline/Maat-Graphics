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
  semaphore_image_available: vk::Semaphore,
  semaphore_render_finished: vk::Semaphore,
  command_pool: vk::CommandPool,
  command_buffers: Vec<vk::CommandBuffer>,
  render_pass: vk::RenderPass,
  framebuffers: Vec<vk::Framebuffer>,
}

impl Vulkan {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> Vulkan {
    let window = VkWindow::new(app_name, app_version, width, height, should_debug);
    
    let fence: vk::Fence;
    let semaphore_image_available: vk::Semaphore;
    let semaphore_render_finished: vk::Semaphore;
    let command_pool: vk::CommandPool;
    let command_buffers: Vec<vk::CommandBuffer>;
    let render_pass: vk::RenderPass;
    let framebuffers: Vec<vk::Framebuffer>;
    
    {
      let vk = window.device_pointers();
      let device = window.device();
      let format = window.swapchain_format();
      let graphics_family = window.get_graphics_family();
      let current_extent = window.get_current_extent();
      let image_views = window.swapchain_image_views();
      
      fence = Vulkan::create_fence(vk, device);
      let (semaphore1, semaphore2) = Vulkan::create_semaphores(vk, device);
      semaphore_image_available = semaphore1;
      semaphore_render_finished = semaphore2;
      command_pool = Vulkan::create_command_pool(vk, device, graphics_family);
      command_buffers = Vulkan::create_command_buffers(vk, device, &command_pool, 1);
      render_pass = Vulkan::create_render_pass(vk, device, &format);
      framebuffers = Vulkan::create_frame_buffers(vk, device, &render_pass, &current_extent, image_views);
    }
    
    Vulkan {
      window: window,
      fence: fence,
      semaphore_image_available: semaphore_image_available,
      semaphore_render_finished: semaphore_render_finished,
      command_pool: command_pool,
      command_buffers: command_buffers,
      render_pass: render_pass,
      framebuffers: framebuffers,
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
        pAttachments: image_views.as_ptr(),
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
    
    let subpass_dependency = vk::SubpassDependency {
      srcSubpass: vk::SUBPASS_EXTERNAL,
      dstSubpass: 0,
      srcStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
      dstStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
      srcAccessMask: 0,
      dstAccessMask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
      dependencyFlags: vk::DEPENDENCY_BY_REGION_BIT,
    };
    
    let render_pass_create_info = vk::RenderPassCreateInfo {
      sType: vk::STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      attachmentCount: 1,
      pAttachments: &attachment_description,
      subpassCount: 1,
      pSubpasses: &subpass_description,
      dependencyCount: 1,
      pDependencies: &subpass_dependency,
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
    let device = self.window.device();
    let vk = self.window.device_pointers();
    println!("Destroying Command pool, semaphores and fences");
    unsafe {
      for framebuffer in &self.framebuffers {
        vk.DestroyFramebuffer(*device, *framebuffer, ptr::null());
      }
      vk.DestroyRenderPass(*device, self.render_pass, ptr::null());
      vk.FreeCommandBuffers(*device, self.command_pool, self.command_buffers.len() as u32, self.command_buffers.as_mut_ptr());
      vk.DestroyCommandPool(*device, self.command_pool, ptr::null());
      vk.DestroySemaphore(*device, self.semaphore_image_available, ptr::null());
      vk.DestroySemaphore(*device, self.semaphore_render_finished, ptr::null());
      vk.DestroyFence(*device, self.fence, ptr::null());
    }
  }
}
