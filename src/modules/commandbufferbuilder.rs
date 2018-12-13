use vk;

use crate::modules::Device;
use crate::modules::RenderPass;
use crate::modules::Pipeline;
use crate::modules::DescriptorSet;
use crate::modules::CommandPool;
use crate::modules::CommandBuffer;
use crate::ownage::check_errors;

use std::mem;
use std::ptr;
use std::sync::Arc;

pub struct CommandBufferBuilder {
  flags: u32,
  command_buffer: Arc<CommandBuffer>,
}

impl CommandBufferBuilder {
  pub fn primary_one_time_submit(device: &Device, command_buffer: Arc<CommandBuffer>) -> CommandBufferBuilder {
    CommandBufferBuilder {
      flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
      command_buffer,
    }
  }
  
  pub fn begin_command_buffer(mut self, device: &Device) -> CommandBufferBuilder {
    self.command_buffer.begin_command_buffer(device, self.flags);
    
    self
  }
  
  pub fn begin_render_pass(mut self, device: &Device, clear_values: &Vec<vk::ClearValue>, render_pass: &RenderPass, framebuffer: &vk::Framebuffer, render_area: &vk::Extent2D) -> CommandBufferBuilder {
    self.command_buffer.begin_render_pass(device, render_pass, framebuffer, clear_values, render_area.width, render_area.height);
    
    self
  }
  
  pub fn draw_indexed(mut self, device: &Device, vertex_buffer: &vk::Buffer, index_buffer: &vk::Buffer, index_count: u32, pipeline: &Pipeline, descriptor_set: &DescriptorSet) -> CommandBufferBuilder {
    self.command_buffer.bind_pipeline(device, pipeline);
    self.command_buffer.bind_descriptor_set(device, pipeline, descriptor_set);
    self.command_buffer.bind_vertex_buffer(device, vertex_buffer);
    self.command_buffer.bind_index_buffer(device, index_buffer);
    self.command_buffer.draw_indexed(device, index_count, 1);
    
    self
  }
  
  pub fn end_render_pass(mut self, device: &Device) -> CommandBufferBuilder {
    self.command_buffer.end_render_pass(device);
    
    self
  }
  
  pub fn end_command_buffer(mut self, device: &Device) -> CommandBufferBuilder {
    let vk = device.pointers();
    
    self.command_buffer.end_command_buffer(device);
    
    self
  }
  
  pub fn destroy(&self) {
    
  }
}
