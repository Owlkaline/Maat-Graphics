use ash::vk;
use ash::version::DeviceV1_0;

use crate::modules::VkDevice;

pub struct PassDescription {
  format: vk::Format,
  samples: vk::SampleCountFlags,
  load_op: vk::AttachmentLoadOp,
  store_op: vk::AttachmentStoreOp,
  attachment_layout: vk::ImageLayout,
  initial_layout: vk::ImageLayout,
  final_layout: vk::ImageLayout,
  is_depth: bool,
  is_colour: bool,
}

impl PassDescription {
  pub fn new(format: vk::Format) -> PassDescription {
    
    let samples: vk::SampleCountFlags = Default::default();
    let load_op: vk::AttachmentLoadOp = Default::default();
    let store_op: vk::AttachmentStoreOp = Default::default();
    let initial_layout: vk::ImageLayout = Default::default();
    let final_layout: vk::ImageLayout = Default::default();
    let attachment_layout: vk::ImageLayout = Default::default();
    
    PassDescription {
      format,
      samples,
      load_op,
      store_op,
      attachment_layout,
      initial_layout,
      final_layout,
      is_depth: false,
      is_colour: false,
    }
  }
  
  pub fn attachment_layout(&self) -> vk::ImageLayout {
    self.attachment_layout
  }
  
  pub fn final_layout(&self) -> vk::ImageLayout {
    self.final_layout
  }
  
  pub fn is_depth(&self) -> bool {
    self.is_depth
    /*self.final_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL ||
    self.final_layout == vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL*/
  }
  
  pub fn is_colour(&self) -> bool {
    self.is_colour
  }
  
  pub fn attachment_load_op_load(mut self) -> PassDescription {
    self.load_op = vk::AttachmentLoadOp::LOAD;
    self
  }
  
  pub fn attachment_load_op_clear(mut self) -> PassDescription {
    self.load_op = vk::AttachmentLoadOp::CLEAR;
    self
  }
  
  pub fn attachment_store_op_store(mut self) -> PassDescription {
    self.store_op = vk::AttachmentStoreOp::STORE;
    self
  }
  
  pub fn attachment_layout_colour(mut self) -> PassDescription {
    self.attachment_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
    self.is_colour = true;
    self
  }
  
  pub fn attachment_layout_depth_stencil(mut self) -> PassDescription {
    self.attachment_layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
    self.is_depth = true;
    self
  }
  
  pub fn initial_layout_undefined(mut self) -> PassDescription {
    self.initial_layout = vk::ImageLayout::UNDEFINED;
    self
  }
  
  pub fn final_layout_present_src(mut self) -> PassDescription {
    self.final_layout = vk::ImageLayout::PRESENT_SRC_KHR;
    self
  }
  
  pub fn final_layout_general(mut self) -> PassDescription {
    self.final_layout = vk::ImageLayout::GENERAL;
    self
  }
  
  pub fn final_layout_depth_stencil(mut self) -> PassDescription {
    self.final_layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
    self
  }
  
  pub fn final_layout_depth_stencil_read_only(mut self) -> PassDescription {
    self.final_layout = vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL;
    self
  }
  
  pub fn final_layout_shader_read_only(mut self) -> PassDescription {
    self.final_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    self
  }
  
  pub fn final_layout_transfer_src(mut self) -> PassDescription {
    self.final_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
    self
  }
  
  pub fn final_layout_transfer_dst(mut self) -> PassDescription {
    self.final_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    self
  }
  
  pub fn final_layout_preinitialised(mut self) -> PassDescription {
    self.final_layout = vk::ImageLayout::PREINITIALIZED;
    self
  }
  
  pub fn samples_1(mut self) -> PassDescription {
    self.samples = vk::SampleCountFlags::TYPE_1;
    self
  }
  
  pub fn samples_2(mut self) -> PassDescription {
    self.samples = vk::SampleCountFlags::TYPE_2;
    self
  }
  
  pub fn samples_4(mut self) -> PassDescription {
    self.samples = vk::SampleCountFlags::TYPE_4;
    self
  }
  
  pub fn samples_8(mut self) -> PassDescription {
    self.samples = vk::SampleCountFlags::TYPE_8;
    self
  }
  
  pub fn samples_16(mut self) -> PassDescription {
    self.samples = vk::SampleCountFlags::TYPE_16;
    self
  }
  
  pub fn samples_32(mut self) -> PassDescription {
    self.samples = vk::SampleCountFlags::TYPE_32;
    self
  }
  
  pub fn samples_64(mut self) -> PassDescription {
    self.samples = vk::SampleCountFlags::TYPE_64;
    self
  }
  
  pub fn build(&self) ->  vk::AttachmentDescription {
    vk::AttachmentDescription {
      format: self.format,
      samples: self.samples,
      load_op: self.load_op,
      store_op: self.store_op,
      final_layout: self.final_layout,
      ..Default::default()
    }
  }
}

pub struct Renderpass {
  renderpass: vk::RenderPass,
}

impl Renderpass {
  pub fn new(device: &VkDevice, passes: Vec<PassDescription>) -> Renderpass {
    
    let mut pass_descriptions = Vec::new();
    let mut dependancies = Vec::new();
    //let mut input_attachments = Vec::new();
    let mut color_attachments = Vec::new();
    //let mut resolve_attachments = Vec::new();
    let mut depth_stencil_attachment = None;
    //let mut preserve_attachments = Vec::new();
    
    for i in 0..passes.len() {
      pass_descriptions.push(passes[i].build());
      
      if passes[i].final_layout() != vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL &&
         passes[i].final_layout() != vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL &&
         passes[i].final_layout() != vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
        dependancies.push(
          vk::SubpassDependency {
            // srcSubpass is the subpass index of the first subpass in the dependency, or VK_SUBPASS_EXTERNAL
            src_subpass: vk::SUBPASS_EXTERNAL,
            // srcStageMask is a bitmask of VkPipelineStageFlagBits specifying the source stage mask.
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            // srcAccessMask is a bitmask of VkAccessFlagBits specifying a source access mask.
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            // dstStageMask is a bitmask of VkPipelineStageFlagBits specifying the destination stage mask
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
          }
        );
      }
      
      if passes[i].is_depth() {
        depth_stencil_attachment = Some(
          vk::AttachmentReference {
            attachment: i as u32,
            layout: passes[i].attachment_layout(),
          }
        );
      } else if passes[i].is_colour() {
        color_attachments.push(
          vk::AttachmentReference {
            attachment: i as u32,
            layout: passes[i].attachment_layout(),
          }
        );
      }
    }
    
    let mut subpasses = vk::SubpassDescription::builder()
                                              .color_attachments(&color_attachments);
    
    let depth_stencil;
    if depth_stencil_attachment.is_some() {
      depth_stencil = depth_stencil_attachment.unwrap();
      subpasses = subpasses.depth_stencil_attachment(&depth_stencil);
    }
    
    let subpasses = [subpasses.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS).build()];
    
    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
                                     .attachments(&pass_descriptions)
                                     .subpasses(&subpasses)
                                     .dependencies(&dependancies);
    
    let renderpass: vk::RenderPass = unsafe {
      device.internal().create_render_pass(&renderpass_create_info, None).unwrap()
    };
    
    Renderpass {
      renderpass,
    }
  }
  
  pub fn internal(&self) -> vk::RenderPass {
    self.renderpass
  }
}








fn create_renderpass(device: &VkDevice, surface_format: vk::SurfaceFormatKHR) -> vk::RenderPass {
    let renderpass_attachments = [
        vk::AttachmentDescription {
            format: surface_format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        },
        vk::AttachmentDescription {
            format: vk::Format::D16_UNORM,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        },
    ];
    
    let color_attachment_refs = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];
    let depth_attachment_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    
    let dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
            | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        ..Default::default()
    }];

    let subpasses = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_refs)
        .depth_stencil_attachment(&depth_attachment_ref)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .build()];

    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&renderpass_attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    let renderpass = unsafe {
        device.internal()
            .create_render_pass(&renderpass_create_info, None)
            .unwrap()
    };

    renderpass
}



















