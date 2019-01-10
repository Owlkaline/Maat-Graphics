use vk;

use crate::vulkan::Device;
use crate::vulkan::RenderPass;
use crate::vulkan::Pipeline;
use crate::vulkan::buffer::{CommandBuffer, UniformData};
use crate::vulkan::vkenums::{ShaderStageFlagBits};

use std::sync::Arc;

pub struct CommandBufferBuilder {
  flags: u32,
  command_buffer: Arc<CommandBuffer>,
}

impl CommandBufferBuilder {
  pub fn primary_one_time_submit(command_buffer: Arc<CommandBuffer>) -> CommandBufferBuilder {
    CommandBufferBuilder {
      flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
      command_buffer,
    }
  }
  
  pub fn begin_command_buffer(self, device: Arc<Device>) -> CommandBufferBuilder {
    self.command_buffer.begin_command_buffer(Arc::clone(&device), self.flags);
    self
  }
  
  pub fn begin_render_pass(self, device: Arc<Device>, clear_values: &Vec<vk::ClearValue>, render_pass: &RenderPass, framebuffer: &vk::Framebuffer, render_area: &vk::Extent2D) -> CommandBufferBuilder {
    self.command_buffer.begin_render_pass(Arc::clone(&device), render_pass, framebuffer, clear_values, render_area.width, render_area.height);
    
    self
  }
  
  pub fn set_viewport(self, device: Arc<Device>, x: f32, y: f32, width: f32, height: f32) -> CommandBufferBuilder {
    self.command_buffer.set_viewport(Arc::clone(&device), x, y, width, height);
    self
  }
  
  pub fn set_scissor(self, device: Arc<Device>, x: i32, y: i32, width: u32, height: u32) -> CommandBufferBuilder {
    self.command_buffer.set_scissor(Arc::clone(&device), x, y, width, height);
    self
  }
  
  pub fn push_constants(mut self, device: Arc<Device>, pipeline: &Pipeline, shader_stage: ShaderStageFlagBits, push_constant_data: UniformData) -> CommandBufferBuilder {
    self.command_buffer.push_constants(Arc::clone(&device), pipeline, shader_stage, push_constant_data);
    
    self
  }
  
  pub fn draw_indexed(self, device: Arc<Device>, vertex_buffer: &vk::Buffer, index_buffer: &vk::Buffer, index_count: u32, pipeline: &Pipeline, descriptor_set: Vec<&vk::DescriptorSet>) -> CommandBufferBuilder {
    self.command_buffer.bind_pipeline(Arc::clone(&device), pipeline);
    for i in 0..descriptor_set.len() {
      self.command_buffer.bind_descriptor_set(Arc::clone(&device), pipeline, descriptor_set[i]);
    }
    self.command_buffer.bind_vertex_buffer(Arc::clone(&device), vertex_buffer);
    self.command_buffer.bind_index_buffer(Arc::clone(&device), index_buffer);
    self.command_buffer.draw_indexed(Arc::clone(&device), index_count, 1);
    
    self
  }
  
  pub fn end_render_pass(self, device: Arc<Device>) -> CommandBufferBuilder {
    self.command_buffer.end_render_pass(Arc::clone(&device));
    self
  }
  
  pub fn end_command_buffer(self, device: Arc<Device>) -> CommandBufferBuilder {
    self.command_buffer.end_command_buffer(Arc::clone(&device));
    self
  }
  
  pub fn destroy(&self) {
    
  }
}