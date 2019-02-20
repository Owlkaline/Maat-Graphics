use vk;

use crate::vulkan::{Device, RenderPass, Pipeline, ClearValues, ImageAttachment};
use crate::vulkan::buffer::{CommandBuffer, UniformData, Buffer};
use crate::vulkan::vkenums::{ShaderStage, CommandBufferUsage, Access, ImageLayout, ImageAspect, PipelineStage};

use std::sync::Arc;

pub struct CommandBufferBuilder {
  flags: u32,
  command_buffer: Arc<CommandBuffer>,
}

impl CommandBufferBuilder {
  pub fn primary_one_time_submit(command_buffer: Arc<CommandBuffer>) -> CommandBufferBuilder {
    CommandBufferBuilder {
      flags: CommandBufferUsage::OneTimeSubmit.to_bits(),
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
  
  pub fn copy_buffer_to_buffer<T: Clone, U: Clone>(self, device: Arc<Device>, src_buffer: &Buffer<T>, dst_buffer: &Buffer<U>, current_buffer: usize) -> CommandBufferBuilder {
    self.command_buffer.copy_buffer(Arc::clone(&device), src_buffer, dst_buffer, current_buffer);
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
  
  pub fn push_constants(self, device: Arc<Device>, pipeline: &Pipeline, shader_stage: ShaderStage, push_constant_data: UniformData) -> CommandBufferBuilder {
    self.command_buffer.push_constants(Arc::clone(&device), pipeline, shader_stage, push_constant_data);
    
    self
  }
  
  pub fn draw(self, device: Arc<Device>, vertex_buffer: &vk::Buffer, vertex_count: u32, pipeline: &Pipeline, descriptor_set: Vec<vk::DescriptorSet>) -> CommandBufferBuilder {
    self.command_buffer.bind_graphics_pipeline(Arc::clone(&device), pipeline);
    
    self.command_buffer.bind_graphics_descriptor_set(Arc::clone(&device), pipeline, descriptor_set);
    
    self.command_buffer.bind_vertex_buffer(Arc::clone(&device), 0, vertex_buffer);
    self.command_buffer.draw(Arc::clone(&device), vertex_count, 1);
    
    self
  }
  
  pub fn draw_indexed(self, device: Arc<Device>, vertex_buffer: &vk::Buffer, index_buffer: &vk::Buffer, index_count: u32, pipeline: &Pipeline, descriptor_set: Vec<vk::DescriptorSet>) -> CommandBufferBuilder {
    self.command_buffer.bind_graphics_pipeline(Arc::clone(&device), pipeline);
    
    self.command_buffer.bind_graphics_descriptor_set(Arc::clone(&device), pipeline, descriptor_set);
    
    self.command_buffer.bind_vertex_buffer(Arc::clone(&device), 0, vertex_buffer);
    self.command_buffer.bind_index_buffer(Arc::clone(&device), index_buffer);
    self.command_buffer.draw_indexed(Arc::clone(&device), index_count, 1);
    
    self
  }
  
    pub fn draw_instanced_indexed(self, device: Arc<Device>, vertex_buffer: &vk::Buffer, index_buffer: &vk::Buffer, instance_buffer: &vk::Buffer, index_count: u32, instance_count: u32, pipeline: &Pipeline, descriptor_set: Vec<vk::DescriptorSet>) -> CommandBufferBuilder {
    self.command_buffer.bind_graphics_pipeline(Arc::clone(&device), pipeline);
    
    self.command_buffer.bind_graphics_descriptor_set(Arc::clone(&device), pipeline, descriptor_set);
    
    self.command_buffer.bind_vertex_buffer(Arc::clone(&device), 0, vertex_buffer);
    self.command_buffer.bind_vertex_buffer(Arc::clone(&device), 1, instance_buffer);
    self.command_buffer.bind_index_buffer(Arc::clone(&device), index_buffer);
    
    self.command_buffer.draw_indexed(Arc::clone(&device), index_count, instance_count);
    
    self
  }
  
  pub fn compute_dispatch(self, device: Arc<Device>, pipeline: &Pipeline, descriptor_set: Vec<vk::DescriptorSet>, x: u32, y: u32, z: u32) -> CommandBufferBuilder {
    self.command_buffer.bind_compute_pipeline(Arc::clone(&device), pipeline);
    self.command_buffer.bind_compute_descriptor_set(Arc::clone(&device), pipeline, descriptor_set);
    
    self.command_buffer.dispatch(Arc::clone(&device), x, y, z);
    
    self
  }
  
  pub fn image_barrier(self, device: Arc<Device>, src_mask: &Access, dst_mask: &Access, old_layout: &ImageLayout, new_layout: &ImageLayout, aspect: &ImageAspect, src_stage: PipelineStage, dst_stage: PipelineStage, src_queue_family: u32, dst_queue_family: u32, image: &ImageAttachment) -> CommandBufferBuilder {
    self.command_buffer.image_barrier(Arc::clone(&device), src_mask, dst_mask, old_layout, new_layout, aspect, src_stage, dst_stage, src_queue_family, dst_queue_family, image);
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
