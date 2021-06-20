use crate::modules::{VkDevice, VkSwapchain, Renderpass, Image};

use ash::{vk, Device};
use ash::version::DeviceV1_0;

pub struct VkFrameBuffer {
  framebuffers: Vec<vk::Framebuffer>,
}

impl VkFrameBuffer {
  pub fn new(device: &VkDevice, swapchain: &mut VkSwapchain, depth_image: &Image,
             renderpass: &Renderpass) -> VkFrameBuffer {
    
    let extent = swapchain.extent();
    let framebuffers: Vec<vk::Framebuffer> = swapchain.present_images()
        .iter()
        .map(|present_image| {
            let present_image_view = present_image.view();
            let framebuffer_attachments = [present_image_view, depth_image.view()];
            let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass.internal())
                .attachments(&framebuffer_attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            unsafe {
                device.internal()
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .unwrap()
            }
        })
        .collect();
    
    VkFrameBuffer {
      framebuffers,
    }
  }
  
  pub fn framebuffers(&self) -> &Vec<vk::Framebuffer> {
    &self.framebuffers
  }
  
  pub fn destroy(&self, device: &Device) {
    unsafe {
      for &framebuffer in self.framebuffers.iter() {
        device.destroy_framebuffer(framebuffer, None);
      }
    }
  }
}
