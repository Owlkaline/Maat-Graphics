use vk;

use crate::modules::Device;

use std::mem;
use std::ptr;

pub struct RenderPass {
  render_pass: vk::RenderPass,
}

impl RenderPass {
  pub fn new(device: &Device, format: &vk::Format) -> RenderPass {
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
    
    RenderPass {
      render_pass,
    }
  }
  
  pub fn internal_object(&self) -> &vk::RenderPass {
    &self.render_pass
  }
  
  pub fn destroy(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.local_device();
    
    println!("Destroying RenderPass");
    
    unsafe {
      vk.DestroyRenderPass(*device, self.render_pass, ptr::null());
    }
  }
}
