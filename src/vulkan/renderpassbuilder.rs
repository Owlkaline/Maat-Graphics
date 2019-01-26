use crate::vulkan::Device;
use crate::vulkan::RenderPass;
use crate::vulkan::vkenums::{Sample, AttachmentLoadOp, AttachmentStoreOp, ImageLayout};

use std::mem;
use std::ptr;
use std::sync::Arc;

pub struct AttachmentInfo {
  format: vk::Format,
  samples: Sample,
  load: AttachmentLoadOp,
  store: AttachmentStoreOp,
  stencil_load: AttachmentLoadOp,
  stencil_store: AttachmentStoreOp,
  initial_layout: ImageLayout,
  final_layout: ImageLayout,
  image_usage: ImageLayout,
}

pub struct SubpassInfo {
  num_input_attachments: usize,
  input_attachment_indexs: Vec<usize>,
  num_colour_attachments: usize,
  colour_attachment_indexs: Vec<usize>,
  resolve_attachment_indexs: Option<Vec<usize>>,
  depth_stencil_index: Option<usize>,
  //num_preserve_attachments: usize,
 // preserve_attachment_indexs: Vec<usize>,
}

pub struct RenderPassBuilder {
  attachments: Vec<AttachmentInfo>,
  subpasses: Vec<SubpassInfo>,
}

impl AttachmentInfo {
  pub fn new() -> AttachmentInfo {
    AttachmentInfo {
      format: 0,
      samples: Sample::Count1Bit,
      load: AttachmentLoadOp::Clear,
      store: AttachmentStoreOp::Store,
      stencil_load: AttachmentLoadOp::DontCare,
      stencil_store: AttachmentStoreOp::DontCare,
      initial_layout: ImageLayout::Undefined,
      final_layout: ImageLayout::PresentSrcKHR,
      image_usage: ImageLayout::Undefined,
    }
  }
  
  pub fn format(mut self, format: vk::Format) -> AttachmentInfo {
    self.format = format;
    self
  }
  
  pub fn multisample(mut self, sample: usize) -> AttachmentInfo {
    if sample >= 16 {
      self.samples = Sample::Count16Bit;
    } else if sample >= 8 {
      self.samples = Sample::Count8Bit;
    } else if sample >= 4 {
      self.samples = Sample::Count4Bit;
    } else if sample >= 2 {
      self.samples = Sample::Count2Bit;
    } else {
      self.samples = Sample::Count1Bit;
    }
    self
  }
  
  pub fn load(mut self, load_op: AttachmentLoadOp) -> AttachmentInfo {
    self.load = load_op;
    self
  }
  
  pub fn store(mut self, store_op: AttachmentStoreOp) -> AttachmentInfo {
    self.store = store_op;
    self
  }
  
  pub fn stencil_load(mut self, load_op: AttachmentLoadOp) -> AttachmentInfo {
    self.stencil_load = load_op;
    self
  }
  
  pub fn stencil_store(mut self, store_op: AttachmentStoreOp) -> AttachmentInfo {
    self.stencil_store = store_op;
    self
  }
  
  pub fn initial_layout(mut self, layout: ImageLayout) -> AttachmentInfo {
    self.initial_layout = layout;
    self
  }
  
  pub fn final_layout(mut self, layout: ImageLayout) -> AttachmentInfo {
    self.final_layout = layout;
    self
  }
  
  pub fn image_usage(mut self, layout: ImageLayout) -> AttachmentInfo {
    self.image_usage = layout;
    self
  }
  
  pub fn get_image_usage(&self) -> ImageLayout {
    self.image_usage.clone()
  }
  
  pub fn build(&self) -> vk::AttachmentDescription {
    let description = vk::AttachmentDescription {
      flags: 0,
      format: self.format,
      samples: self.samples.to_bits(),
      loadOp: self.load.to_bits(),
      storeOp: self.store.to_bits(),
      stencilLoadOp: self.stencil_load.to_bits(),
      stencilStoreOp: self.stencil_store.to_bits(),
      initialLayout: self.initial_layout.to_bits(),
      finalLayout: self.final_layout.to_bits(),
    };
    
    description
  }
}

impl SubpassInfo {
  pub fn new() -> SubpassInfo {
    SubpassInfo {
      num_input_attachments: 0,
      input_attachment_indexs: Vec::new(),
      num_colour_attachments: 0,
      colour_attachment_indexs: Vec::new(),
      resolve_attachment_indexs: None,
      depth_stencil_index: None,
    }
  }
  
  pub fn get_num_input_count(&self) -> u32 {
    self.num_input_attachments as u32
  }
  
  pub fn num_colour_attachments(&self) -> u32 {
    self.num_colour_attachments as u32
  }
  
  pub fn get_colour_attachment_index(&self, i: usize) -> u32 {
    self.colour_attachment_indexs[i] as u32
  }
  
  pub fn add_input_attachment(mut self, index: usize) -> SubpassInfo {
    self.num_input_attachments += 1;
    self.input_attachment_indexs.push(index);
    self
  }
  
  pub fn add_colour_attachment(mut self, index: usize) -> SubpassInfo {
    self.num_colour_attachments += 1;
    self.colour_attachment_indexs.push(index);
    self
  }
  
  pub fn add_resolve_attachment(mut self, index: usize) -> SubpassInfo {
    self.resolve_attachment_indexs = Some(vec!(index));
    self
  }
  
  pub fn add_depth_stencil(mut self, index: usize) -> SubpassInfo {
    self.depth_stencil_index = Some(index);
    self
  }
  
  pub fn get_input_attachment_references(&self) -> Option<Vec<vk::AttachmentReference>> {
    if self.num_input_attachments == 0 {
      return None;
    }
    
    let mut attachments = Vec::with_capacity(self.input_attachment_indexs.len());
    for i in 0..self.input_attachment_indexs.len() {
      attachments.push(ImageLayout::TransferSrcOptimal.to_attachment_reference(self.input_attachment_indexs[i] as u32));
    }
    
    Some(attachments)
  }
  
  pub fn get_colour_attachment_references(&self) -> Vec<vk::AttachmentReference> {
    if self.num_colour_attachments == 0 {
      return Vec::new();
    }
    
    let mut attachments = Vec::with_capacity(self.colour_attachment_indexs.len());
    for i in 0..self.colour_attachment_indexs.len() {
      let index = self.colour_attachment_indexs[i] as u32;
      let layout = ImageLayout::ColourAttachmentOptimal;
      let references = layout.to_attachment_reference(index);
      attachments.push(references);
    }
    
    attachments
  }
  
  pub fn get_resolve_attachment_references(&mut self) -> Option<Vec<vk::AttachmentReference>> {
    if let Some(attachment_indexs) = &self.resolve_attachment_indexs {
      let mut attachments = Vec::with_capacity(attachment_indexs.len());
      for i in 0..attachment_indexs.len() {
        attachments.push(ImageLayout::TransferDstOptimal.to_attachment_reference(attachment_indexs[i] as u32));
      }
      
      Some(attachments)
    } else { 
      None 
    }
  }
  
  pub fn get_depth_stencil_attachment_references(&self) -> Option<Vec<vk::AttachmentReference>> {
    if self.depth_stencil_index.is_none() {
      None
    } else {
      let depth_stencil_index = self.depth_stencil_index.unwrap();
      let mut attachments = Vec::with_capacity(1);
      attachments.push(ImageLayout::DepthStencilAttachmentOptimal.to_attachment_reference(depth_stencil_index as u32));
      
      Some(attachments)
    }
  }
}



impl RenderPassBuilder {
  pub fn new() -> RenderPassBuilder {
    RenderPassBuilder {
      attachments: Vec::new(),
      subpasses: Vec::new(),
    }
  }
  
  pub fn add_attachment(mut self, attachment: AttachmentInfo) -> RenderPassBuilder {
    self.attachments.push(attachment);
    self
  }
  
  pub fn add_subpass(mut self, subpass: SubpassInfo) -> RenderPassBuilder {
    self.subpasses.push(subpass);
    self
  }
  
  pub fn build(mut self, device: Arc<Device>) -> RenderPass {
    let mut render_pass: vk::RenderPass = unsafe { mem::uninitialized() };
    
    let mut attachment_descriptions = Vec::with_capacity(self.attachments.len());
    let mut subpass_descriptions = Vec::with_capacity(self.subpasses.len());
    //let mut subpass_dependency: Vec<vk::SubpassDependency> = Vec::with_capacity(2);
    let mut input_attachments: Vec<vk::AttachmentReference> = Vec::new();
    let mut colour_attachments: Vec<vk::AttachmentReference> = Vec::new();
    let mut resolve_attachments: Vec<vk::AttachmentReference> = Vec::new();
    let mut depth_stencil_attachments: Vec<vk::AttachmentReference> = Vec::new();
    
    for i in 0..self.attachments.len() {
      let attachment = self.attachments[i].build();
      attachment_descriptions.push(
        attachment
      );
    }
    
    for i in 0..self.subpasses.len() {
      if let Some(input) = self.subpasses[i].get_input_attachment_references() {
        input_attachments = input;
      }
      
      let num_attchments = self.subpasses[i].num_colour_attachments();
      
      for j in 0..num_attchments as usize {
        let attachment_index = self.subpasses[i].get_colour_attachment_index(j);
        let attachment_layout = self.attachments[attachment_index as usize].get_image_usage();
        let reference = vk::AttachmentReference {
          attachment: attachment_index,
          layout: attachment_layout.to_bits(),
        };
        
        colour_attachments.push(reference);
      }
      
      if let Some(resolve) = self.subpasses[i].get_resolve_attachment_references() {
        resolve_attachments = resolve;
      }
      
      if let Some(depth_stencil) = self.subpasses[i].get_depth_stencil_attachment_references() {
        depth_stencil_attachments = depth_stencil;
      }
      
      subpass_descriptions.push(
        vk::SubpassDescription {
          flags: 0,
          pipelineBindPoint: vk::PIPELINE_BIND_POINT_GRAPHICS,
          inputAttachmentCount: input_attachments.len() as u32,
          pInputAttachments: if input_attachments.len() == 0 { ptr::null() } else { input_attachments.as_ptr() },
          colorAttachmentCount: colour_attachments.len() as u32,
          pColorAttachments: if colour_attachments.len() == 0 { ptr::null() } else { colour_attachments.as_ptr() },
          pResolveAttachments: if resolve_attachments.len() == 0 { ptr::null() } else { resolve_attachments.as_ptr() },
          pDepthStencilAttachment: if depth_stencil_attachments.len() == 0 { ptr::null() } else { depth_stencil_attachments.as_ptr() },
          preserveAttachmentCount: 0,
          pPreserveAttachments: ptr::null(),
        }
      );
    }
    
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
    
    let render_pass_create_info = vk::RenderPassCreateInfo {
      sType: vk::STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      attachmentCount: attachment_descriptions.len() as u32,
      pAttachments: attachment_descriptions.as_ptr(),
      subpassCount: subpass_descriptions.len() as u32,
      pSubpasses: subpass_descriptions.as_ptr(),
      dependencyCount: subpass_dependency.len() as u32,
      pDependencies: subpass_dependency.as_ptr(),
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      vk.CreateRenderPass(*device, &render_pass_create_info, ptr::null(), &mut render_pass);
    }
    
    let num_attachments = input_attachments.len() + colour_attachments.len() + resolve_attachments.len() + depth_stencil_attachments.len();
    
    RenderPass::new_from_renderpass(render_pass, num_attachments as u32)
  }
}
