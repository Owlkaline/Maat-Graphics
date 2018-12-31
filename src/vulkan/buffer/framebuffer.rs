use vk;

use crate::vulkan::Device;
use crate::vulkan::RenderPass;
use crate::vulkan::ownage::check_errors;

use std::mem;
use std::ptr;

pub struct Framebuffer {
  framebuffer: vk::Framebuffer,
}

impl Framebuffer {
  pub fn new(device: &Device, render_pass: &RenderPass, extent: &vk::Extent2D, image_view: &vk::ImageView) -> Framebuffer {
    let mut framebuffer: vk::Framebuffer = unsafe { mem::uninitialized() };
    
    let framebuffer_create_info = vk::FramebufferCreateInfo {
      sType: vk::STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      renderPass: *render_pass.internal_object(),
      attachmentCount: 1,
      pAttachments: image_view,
      width: extent.width,
      height: extent.height,
      layers: 1,
    };
    
    let vk = device.pointers();
    let device = device.internal_object();
    
    unsafe {
      check_errors(vk.CreateFramebuffer(*device, &framebuffer_create_info, ptr::null(), &mut framebuffer));
    }
    
    Framebuffer {
      framebuffer,
    }
  }
  
  pub fn internal_object(&self) -> &vk::Framebuffer {
    &self.framebuffer
  }
  
  pub fn destroy(&self, device: &Device) {
    println!("Destroying Framebuffer");
    
    let vk = device.pointers();
    let device = device.internal_object();
    unsafe {
      vk.DestroyFramebuffer(*device, self.framebuffer, ptr::null());
    }
  }
}
