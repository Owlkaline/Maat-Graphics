use vulkano::image as vkimage;
use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::pipeline::GraphicsPipelineAbstract;


use std::sync::Arc;
use std::marker::Sync;
use std::marker::Send;

pub struct CustomRenderpass {
  renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
  pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  framebuffer: Option<Arc<framebuffer::FramebufferAbstract + Send + Sync>>,
  attachment: Arc<vkimage::AttachmentImage>,
}

impl CustomRenderpass {
  pub fn new(attachment: Arc<vkimage::AttachmentImage>) -> CustomRenderpass {
    CustomRenderpass {
      renderpass: None,
      pipeline: None,
      framebuffer: None,
      attachment: attachment,
    }
  }
  
  pub fn replace(renderpass: Arc<RenderPassAbstract + Send + Sync>, pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>, attachment: Arc<vkimage::AttachmentImage>) -> CustomRenderpass {
    CustomRenderpass {
      renderpass: Some(renderpass),
      pipeline: Some(pipeline),
      framebuffer: None,
      attachment: attachment,
    }
  }
  
  pub fn renderpass(&self) -> Arc<RenderPassAbstract + Send + Sync> {
    self.renderpass.clone().unwrap()
  }
  
  pub fn pipeline(&self) -> Arc<GraphicsPipelineAbstract + Send + Sync> {
    self.pipeline.clone().unwrap()
  }
  
  pub fn framebuffer(&self) -> Arc<framebuffer::FramebufferAbstract + Send + Sync> {
    self.framebuffer.clone().unwrap()
  }
  
  pub fn framebuffer_ref(&self) -> Arc<framebuffer::FramebufferAbstract + Send + Sync> {
    self.framebuffer.as_ref().unwrap().clone()
  }
  
  pub fn attachment(&self) -> Arc<vkimage::AttachmentImage> {
    self.attachment.clone()
  }
  
  pub fn set_renderpass(&mut self, renderpass: Arc<RenderPassAbstract + Send + Sync>) {
    self.renderpass = Some(renderpass);
  }
  
  pub fn set_pipeline(&mut self, pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>) {
    self.pipeline = Some(pipeline);
  }
  
  pub fn set_framebuffer(&mut self, framebuffer: Arc<framebuffer::FramebufferAbstract + Send + Sync>) {
    self.framebuffer = Some(framebuffer);
  }
  
  pub fn set_attachment(&mut self, attachment: Arc<vkimage::AttachmentImage>) {
    self.attachment = attachment;
  }
}
