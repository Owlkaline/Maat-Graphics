use vk;

use crate::vulkan::vkenums::{SampleCount, AttachmentLoadOp, AttachmentStoreOp, ImageLayout, PipelineBindPoint, PipelineStage, Access, Dependency};

use crate::vulkan::Device;

use std::mem;
use std::ptr;
use std::sync::Arc;

#[derive(Clone)]
pub struct RenderPass {
  render_pass: vk::RenderPass,
  num_attachments: u32,
  num_colour_attachments: Vec<u32>,
}

impl RenderPass {
  pub fn new_from_renderpass(render_pass: vk::RenderPass, num_attachments: u32, num_colour_attachments: Vec<u32>) -> RenderPass {
    RenderPass {
      render_pass,
      num_attachments,
      num_colour_attachments,
    }
  }
  
  pub fn internal_object(&self) -> &vk::RenderPass {
    &self.render_pass
  }
  
  pub fn get_num_attachments(&self) -> u32 {
    self.num_attachments
  }
  
  pub fn get_num_colour_attachments_in_subpass(&self, subpass: u32) -> u32 {
    self.num_colour_attachments[subpass as usize]
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    println!("Destroying RenderPass");
    
    unsafe {
      vk.DestroyRenderPass(*device, self.render_pass, ptr::null());
    }
  }
}
