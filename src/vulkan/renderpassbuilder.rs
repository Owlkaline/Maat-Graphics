use crate::vulkan::Device;
use crate::vulkan::RenderPass;
use crate::vulkan::vkenums::{SampleCount, AttachmentLoadOp, AttachmentStoreOp, ImageLayout, PipelineStage, 
                             Access, Dependency};

use std::mem;
use std::ptr;
use std::sync::Arc;

pub struct AttachmentInfo {
  format: vk::Format,
  samples: SampleCount,
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
  num_resolve_attachments: usize,
  resolve_attachment_indexs: Vec<usize>,
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
      samples: SampleCount::OneBit,
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
  
  pub fn multisample(mut self, sample: &SampleCount) -> AttachmentInfo {
    self.samples = *sample;
    self
  }
  
  /*
  pub fn multisample(mut self, sample: usize) -> AttachmentInfo {
    if sample >= 64 {
      self.samples = SampleCount::SixtyFourBit;
    } else if sample >= 32 {
      self.samples = SampleCount::ThirtyTwoBit;
    } else if sample >= 16 {
      self.samples = SampleCount::SixteenBit;
    } else if sample >= 8 {
      self.samples = SampleCount::EightBit;
    } else if sample >= 4 {
      self.samples = SampleCount::FourBit;
    } else if sample >= 2 {
      self.samples = SampleCount::TwoBit;
    } else {
      self.samples = SampleCount::OneBit;
    }
    self
  }*/
  
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
  
  pub fn get_final_layout(&self) -> ImageLayout {
    self.final_layout.clone()
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
      num_resolve_attachments: 0,
      resolve_attachment_indexs: Vec::new(),
      depth_stencil_index: None,
    }
  }
  
  pub fn num_input_attachments(&self) -> u32 {
    self.num_input_attachments as u32
  }
  
  pub fn num_colour_attachments(&self) -> u32 {
    self.num_colour_attachments as u32
  }
  
  pub fn get_colour_attachment_index(&self, i: usize) -> u32 {
    self.colour_attachment_indexs[i] as u32
  }
  
  pub fn num_resolve_attachments(&self) -> u32 {
    self.num_resolve_attachments as u32
  }
  
  pub fn num_depth_attachments(&self) -> u32 {
    if self.depth_stencil_index.is_some() { 1 } else { 0 }
  }
  
  pub fn get_resolve_attachment_index(&self, i: usize) -> u32 {
    self.resolve_attachment_indexs[i] as u32
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
    self.num_resolve_attachments += 1;
    self.resolve_attachment_indexs.push(index);
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
      attachments.push(ImageLayout::ShaderReadOnlyOptimal.to_attachment_reference(self.input_attachment_indexs[i] as u32));
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
  
  pub fn build(self, device: Arc<Device>) -> RenderPass {
    let mut render_pass: vk::RenderPass = unsafe { mem::uninitialized() };
    
    let mut attachment_descriptions = Vec::with_capacity(self.attachments.len());
    let mut subpass_descriptions = Vec::with_capacity(self.subpasses.len());
    //let mut subpass_dependency: Vec<vk::SubpassDependency> = Vec::with_capacity(2);
    let mut input_attachments: Vec<Vec<vk::AttachmentReference>> = Vec::with_capacity(self.subpasses.len());
    let mut colour_attachments: Vec<Vec<vk::AttachmentReference>> = Vec::with_capacity(self.subpasses.len());
    let mut resolve_attachments: Vec<Vec<vk::AttachmentReference>> = Vec::with_capacity(self.subpasses.len());
    let mut depth_stencil_attachments: Vec<Vec<vk::AttachmentReference>> = Vec::with_capacity(self.subpasses.len());
    
    for i in 0..self.attachments.len() {
      let attachment = self.attachments[i].build();
      attachment_descriptions.push(
        attachment
      );
    }
    
    for i in 0..self.subpasses.len() {
      input_attachments.push(Vec::new());
      colour_attachments.push(Vec::new());
      resolve_attachments.push(Vec::new());
      depth_stencil_attachments.push(Vec::new());
      
      if let Some(input) = self.subpasses[i].get_input_attachment_references() {
        input_attachments[i] = input;
      }
      
      let num_attchments = self.subpasses[i].num_colour_attachments();
      
      for j in 0..num_attchments as usize {
        let attachment_index = self.subpasses[i].get_colour_attachment_index(j);
        let attachment_layout = if i == 0 { self.attachments[attachment_index as usize].get_image_usage() } else {
          self.attachments[attachment_index as usize].get_final_layout()
        };
        let reference = vk::AttachmentReference {
          attachment: attachment_index,
          layout: attachment_layout.to_bits(),
        };
        
        colour_attachments[i].push(reference);
      }
      
      let num_resolve_attachments = self.subpasses[i].num_resolve_attachments();
      for j in 0..num_resolve_attachments as usize {
        let attachment_index = self.subpasses[i].get_resolve_attachment_index(j);
        let attachment_layout = if i == 0 { self.attachments[attachment_index as usize].get_image_usage() } else {
          self.attachments[attachment_index as usize].get_final_layout() };
        let reference = vk::AttachmentReference {
          attachment: attachment_index,
          layout: attachment_layout.to_bits(),
        };
        
        resolve_attachments[i].push(reference);
      }
      
      if let Some(depth_stencil) = self.subpasses[i].get_depth_stencil_attachment_references() {
        depth_stencil_attachments[i] = depth_stencil;
      }
      
      subpass_descriptions.push(
        vk::SubpassDescription {
          flags: 0,
          pipelineBindPoint: vk::PIPELINE_BIND_POINT_GRAPHICS,
          inputAttachmentCount: input_attachments[i].len() as u32,
          pInputAttachments: if input_attachments[i].len() == 0 { ptr::null() } else { input_attachments[i].as_ptr() },
          colorAttachmentCount: colour_attachments[i].len() as u32/* + resolve_attachments.len() as u32*/,
          pColorAttachments: if colour_attachments[i].len() == 0 { ptr::null() } else { colour_attachments[i].as_ptr() },
          pResolveAttachments: if resolve_attachments[i].len() == 0 { ptr::null() } else { resolve_attachments[i].as_ptr() },
          pDepthStencilAttachment: if depth_stencil_attachments[i].len() == 0 { ptr::null() } else { depth_stencil_attachments[i].as_ptr() },
          preserveAttachmentCount: 0,
          pPreserveAttachments: ptr::null(),
        }
      );
    }
    
    let mut subpass_dependency: Vec<vk::SubpassDependency> = Vec::with_capacity(self.subpasses.len());
    
    // Texture in
    subpass_dependency.push(vk::SubpassDependency {
            srcSubpass: vk::SUBPASS_EXTERNAL,
            dstSubpass: 0,
            srcStageMask: PipelineStage::BottomOfPipe.to_bits(),
            dstStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
            srcAccessMask: Access::MemoryRead.to_bits(),
            dstAccessMask: Access::ColourAttachmentReadAndWrite.to_bits(),
            dependencyFlags: Dependency::ByRegion.to_bits(),
    });
    /*
    subpass_dependency.push(vk::SubpassDependency {
            srcSubpass: 0,
            dstSubpass: 1,
            srcStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
            dstStageMask: PipelineStage::FragmentShader.to_bits(),
            srcAccessMask: Access::ColourAttachmentWrite.to_bits(),
            dstAccessMask: Access::ShaderRead.to_bits(),
            dependencyFlags: Dependency::ByRegion.to_bits(),
      });*/
    
    subpass_dependency.push(vk::SubpassDependency {
            srcSubpass: 1,
            dstSubpass: vk::SUBPASS_EXTERNAL,
            srcStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
            dstStageMask: PipelineStage::BottomOfPipe.to_bits(),
            srcAccessMask: Access::ColourAttachmentReadAndWrite.to_bits(),
            dstAccessMask: Access::MemoryRead.to_bits(),
            dependencyFlags: Dependency::ByRegion.to_bits(),
      });
    
    /*
    for i in 0..self.subpasses.len() {
      for j in 0..self.subpasses[i].num_colour_attachments() {
        let num_resolve = self.subpasses[i].num_resolve_attachments();
        if num_resolve > j {
          // Colour Resolve
          subpass_dependency.push(vk::SubpassDependency {
            srcSubpass: i as u32,
            dstSubpass: if i == self.subpasses.len()-1 { vk::SUBPASS_EXTERNAL } else { (i+1) as u32 },
            srcStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
            dstStageMask: PipelineStage::BottomOfPipe.to_bits(),
            srcAccessMask: Access::ColourAttachmentReadAndWrite.to_bits(),
            dstAccessMask: Access::MemoryRead.to_bits(),
            dependencyFlags: Dependency::ByRegion.to_bits(),
          });
        } else {
          // Colour Attachemnt 
          subpass_dependency.push(vk::SubpassDependency {
            srcSubpass: if i == self.subpasses.len()-1 { i as u32 } else { vk::SUBPASS_EXTERNAL },
            dstSubpass:  if i == self.subpasses.len()-1 { vk::SUBPASS_EXTERNAL } else { i as u32 },
            srcStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
            dstStageMask: if i == self.subpasses.len()-1 { PipelineStage::BottomOfPipe.to_bits() } else { PipelineStage::ColorAttachmentOutput.to_bits() },
            srcAccessMask: Access::ColourAttachmentRead.to_bits(),
            dstAccessMask: if i == self.subpasses.len()-1 { Access::MemoryRead.to_bits() } else { 0 },//Access::ColourAttachmentReadAndWrite.to_bits(),
            dependencyFlags: Dependency::ByRegion.to_bits(),
          });
        }
      }
      
      for j in 0..self.subpasses[i].num_resolve_attachments() {
        subpass_dependency.push(vk::SubpassDependency {
          srcSubpass: vk::SUBPASS_EXTERNAL,
          dstSubpass: i as u32,
          srcStageMask: PipelineStage::BottomOfPipe.to_bits(),
          dstStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
          srcAccessMask: Access::MemoryRead.to_bits(),
          dstAccessMask: Access::ColourAttachmentReadAndWrite.to_bits(),
          dependencyFlags: Dependency::ByRegion.to_bits(),
        });
      }
      
      for j in 0..self.subpasses[i].num_input_attachments() {
        subpass_dependency.push(vk::SubpassDependency {
          srcSubpass: (i-1) as u32,
          dstSubpass: i as u32,
          srcStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
          dstStageMask: PipelineStage::FragmentShader.to_bits(),
          srcAccessMask: Access::ColourAttachmentWrite.to_bits(),
          dstAccessMask: Access::ColourAttachmentRead.to_bits(),
          dependencyFlags: Dependency::ByRegion.to_bits(),
        });
      }
    }*/
    
    /*
    let mut subpass_dependency: Vec<vk::SubpassDependency> = Vec::with_capacity(2);
  
      if resolve_attachments.len() == 0 {
        subpass_dependency.push(vk::SubpassDependency {
          srcSubpass: vk::SUBPASS_EXTERNAL,
          dstSubpass: 0,
          srcStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
          dstStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
          srcAccessMask: 0,
          dstAccessMask: Access::ColourAttachmentRead.to_bits() | Access::ColourAttachmentWrite.to_bits(),
          dependencyFlags: Dependency::ByRegion.to_bits(),
        });
      } else {
        // MSAA
        subpass_dependency.push(vk::SubpassDependency {
          srcSubpass: vk::SUBPASS_EXTERNAL,
          dstSubpass: 0,
          srcStageMask: PipelineStage::BottomOfPipe.to_bits(),
          dstStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
          srcAccessMask: Access::MemoryRead.to_bits(),
          dstAccessMask: Access::ColourAttachmentReadAndWrite.to_bits(),
          dependencyFlags: Dependency::ByRegion.to_bits(),
        });
        
        // Colour
        subpass_dependency.push(vk::SubpassDependency {
          srcSubpass: 0,
          dstSubpass: vk::SUBPASS_EXTERNAL,
          srcStageMask: PipelineStage::ColorAttachmentOutput.to_bits(),
          dstStageMask: PipelineStage::BottomOfPipe.to_bits(),
          srcAccessMask: Access::ColourAttachmentReadAndWrite.to_bits(),
          dstAccessMask: Access::MemoryRead.to_bits(),
          dependencyFlags: Dependency::ByRegion.to_bits(),
        });
      }
      */
    
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
    
    let mut num_colour_attachments: Vec<u32> = self.subpasses.iter().map(|s| { s.num_colour_attachments() as u32 }).collect::<Vec<u32>>();
    
    /*
    let mut num_colour_attachments: u32 = self.subpasses.iter().map(|s| { s.num_colour_attachments() as u32 }).sum();
    num_colour_attachments -= self.subpasses.iter().map(|s| { s.num_depth_attachments() as u32 }).sum::<u32>();*/
    let num_attachments = attachment_descriptions.len() as u32;
    
    RenderPass::new_from_renderpass(render_pass, num_attachments, num_colour_attachments)
  }
}
