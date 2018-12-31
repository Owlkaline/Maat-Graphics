use vulkano::image as vkimage;
use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::pipeline::GraphicsPipelineAbstract;

use std::sync::Arc;
use std::marker::Sync;
use std::marker::Send;

pub struct CustomRenderpass {
  renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
  pipeline: Vec<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  framebuffer: Option<Arc<framebuffer::FramebufferAbstract + Send + Sync>>,
  attachments: Vec<Arc<vkimage::AttachmentImage>>,
}

impl CustomRenderpass {
  pub fn new(attachment: Vec<Arc<vkimage::AttachmentImage>>) -> CustomRenderpass {
    CustomRenderpass {
      renderpass: None,
      pipeline: Vec::new(),
      framebuffer: None,
      attachments: attachment,
    }
  }
  
  pub fn new_empty() -> CustomRenderpass {
    CustomRenderpass {
      renderpass: None,
      pipeline: Vec::new(),
      framebuffer: None,
      attachments: Vec::new(),
    }
  }
  
  pub fn replace(renderpass: Arc<RenderPassAbstract + Send + Sync>, pipeline: Vec<Arc<GraphicsPipelineAbstract + Send + Sync>>, attachments: Vec<Arc<vkimage::AttachmentImage>>) -> CustomRenderpass {
    CustomRenderpass {
      renderpass: Some(renderpass),
      pipeline: pipeline,
      framebuffer: None,
      attachments: attachments,
    }
  }
  
  pub fn renderpass(&self) -> Arc<RenderPassAbstract + Send + Sync> {
    self.renderpass.clone().unwrap()
  }
  
  pub fn pipeline_subpass(&self, index: usize) -> Arc<GraphicsPipelineAbstract + Send + Sync> {
    self.pipeline[index].clone()
  }
  
  pub fn framebuffer(&self) -> Arc<framebuffer::FramebufferAbstract + Send + Sync> {
    self.framebuffer.clone().unwrap()
  }
  
  pub fn framebuffer_ref(&self) -> Arc<framebuffer::FramebufferAbstract + Send + Sync> {
    self.framebuffer.as_ref().unwrap().clone()
  }
  
  pub fn attachment(&self, index: usize) -> Arc<vkimage::AttachmentImage> {
    debug_assert!(index < self.attachments.len(), "AttachmentImage Index out of bounds, Limit: {}, Actual: {}", self.attachments.len(), index);
    self.attachments[index].clone()
  }
  
  pub fn set_renderpass(&mut self, renderpass: Arc<RenderPassAbstract + Send + Sync>) {
    self.renderpass = Some(renderpass);
  }
  
  pub fn set_pipelines(&mut self, pipeline: Vec<Arc<GraphicsPipelineAbstract + Send + Sync>>) {
    self.pipeline = pipeline;
  }
  
  pub fn set_framebuffer(&mut self, framebuffer: Arc<framebuffer::FramebufferAbstract + Send + Sync>) {
    self.framebuffer = Some(framebuffer);
  }
  
  pub fn update_attachments(&mut self, attachments: Vec<Arc<vkimage::AttachmentImage>>) {
    debug_assert!(self.renderpass.is_some(), "Renderpass was not set before updating attachments");
    debug_assert!(self.attachments.len() == attachments.len(), "Attachments Expected: {}, Actual Attachments: {}", self.attachments.len(), attachments.len());
    
    self.attachments = attachments.clone();
  }
}
