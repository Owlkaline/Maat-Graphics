trait FinalShader {
  fn create(device: Arc<Device>, swapchain_format: format::Format);
  fn recreate_framebuffer(&mut self, window: Arc<Window>);
  
  fn begin_renderpass(&mut self, cb: AutoCommandBuffer, image_num: usize);
  
  fn end_renderpass(&mut self, cb: AutoCommandBufferBuilder);
}
