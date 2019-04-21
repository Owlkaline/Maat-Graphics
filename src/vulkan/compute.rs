use vk;

use crate::vulkan::vkenums::{ImageLayout, Access, ImageAspect, PipelineStage};

use crate::vulkan::buffer::{CommandBuffer, CommandBufferBuilder};
use crate::vulkan::pool::{CommandPool, DescriptorPool};
use crate::vulkan::sync::{Fence};
use crate::vulkan::{Instance, Device, DescriptorSet, DescriptorSetBuilder, UpdateDescriptorSets, 
                    Pipeline, PipelineBuilder, ImageAttachment, Shader};

use std::sync::Arc;

pub struct Compute {
  queue: vk::Queue,
  _family: u32,
  shader: Shader,
  command_pool: CommandPool,
  command_buffers: Vec<Arc<CommandBuffer>>,
  fences: Vec<Fence>,
  descriptor_sets: Vec<DescriptorSet>,
  pipeline: Pipeline,
}

impl Compute {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, _dummy_image: &ImageAttachment, descriptor_pool: &DescriptorPool, num_sets: u32) -> Compute {
    
    let (compute_queue, compute_family) = device.get_compute_queue(Arc::clone(&instance));
    
    let compute_shader = Shader::new(Arc::clone(&device), include_bytes!("../shaders/sprv/ComputeSharpen.spv"));
    
    let mut descriptor_sets = Vec::with_capacity(num_sets as usize);
    
    for _ in 0..num_sets {
      descriptor_sets.push(DescriptorSetBuilder::new()
                           .compute_storage_image(0)//input
                           .compute_storage_image(1)//output
                           .build(Arc::clone(&device), descriptor_pool, 1));
    }
  /*  UpdateDescriptorSets::new()
      .add_storage_image(0, &dummy_image, ImageLayout::StorageImage)
      .add_storage_image(1, &dummy_image, ImageLayout::General)
      .finish_update(Arc::clone(&device), &descriptor_set);*/
    
    let pipeline = PipelineBuilder::new()
                     .compute_shader(*compute_shader.get_shader())
                     .descriptor_set_layout(descriptor_sets[0].layouts_clone())
                     .build_compute(Arc::clone(&device));
    
    let command_pool = CommandPool::new(Arc::clone(&device), compute_family);
    let command_buffers = command_pool.create_command_buffers(Arc::clone(&device), num_sets);
    
    let mut fences = Vec::with_capacity(num_sets as usize);
    for _ in 0..num_sets as usize {
      fences.push(Fence::new(Arc::clone(&device)));
    }
    
    Compute {
      queue: compute_queue,
      _family: compute_family,
      shader: compute_shader,
      command_pool,
      command_buffers,
      fences,
      descriptor_sets,
      pipeline,
    }
  }
  
  pub fn build(&mut self, device: Arc<Device>, graphics_queue: u32, image: Vec<&ImageAttachment>) {
    for i in 0..self.command_buffers.len() {
      UpdateDescriptorSets::new()
      .add_storage_image(0, &image[i], ImageLayout::General)
      .add_storage_image(1, &image[i], ImageLayout::ColourAttachmentOptimal)
      .finish_update(Arc::clone(&device), &self.descriptor_sets[i]);
      
      let mut cmd = CommandBufferBuilder::primary_one_time_submit(Arc::clone(&self.command_buffers[i]));
      cmd = cmd.begin_command_buffer(Arc::clone(&device));
      
      cmd = cmd.image_barrier(Arc::clone(&device), &Access::ColourAttachmentRead, &Access::ColourAttachmentWrite, &ImageLayout::ColourAttachmentOptimal, &ImageLayout::General, &ImageAspect::Colour, PipelineStage::FragmentShader, PipelineStage::ComputeShader, graphics_queue, self.queue as u32, image[i]);
      
      let (width, height) = image[i].get_size();
      cmd = cmd.compute_dispatch(Arc::clone(&device), &self.pipeline, vec!(*self.descriptor_sets[i].set(0)), width / 16, height / 16, 1);
      
      cmd = cmd.image_barrier(Arc::clone(&device), &Access::ColourAttachmentWrite, &Access::ColourAttachmentRead, &ImageLayout::General, &ImageLayout::ColourAttachmentOptimal, &ImageAspect::Colour, PipelineStage::ComputeShader, PipelineStage::FragmentShader, self.queue as u32, graphics_queue, image[i]);
      
      cmd.end_command_buffer(Arc::clone(&device));
    }
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    for fence in &self.fences {
      fence.wait(Arc::clone(&device));
      fence.destroy(Arc::clone(&device));
    }
    
    self.shader.destroy(Arc::clone(&device));
    
    self.command_pool.destroy(Arc::clone(&device));
    
    self.pipeline.destroy(Arc::clone(&device));
    
    for descriptor in &self.descriptor_sets {
      descriptor.destroy(Arc::clone(&device));
    }
  }
}



