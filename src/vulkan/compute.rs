use vk;

use crate::vulkan::vkenums::{ImageLayout};

use crate::vulkan::buffer::{CommandBuffer};
use crate::vulkan::pool::{CommandPool, DescriptorPool};
use crate::vulkan::sync::{Fence};
use crate::vulkan::{Instance, Device, DescriptorSet, DescriptorSetBuilder, UpdateDescriptorSets, 
                    Pipeline, PipelineBuilder, ImageAttachment};

use std::sync::Arc;

pub struct Compute {
  queue: vk::Queue,
  command_pool: CommandPool,
  command_buffer: Vec<CommandBuffer>,
  fences: Fence,
  descriptor_set: DescriptorSet,
  pipeline: Pipeline,
  graphics_queue: vk::Queue,
}

impl Compute {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, dummy_image: &ImageAttachment, descriptor_pool: &DescriptorPool, num_sets: u32) {
    let compute_queue = device.get_compute_queue(Arc::clone(&instance));
    
    let descriptor_set = DescriptorSetBuilder::new()
                           .compute_storage_image(0)//input
                           .compute_storage_image(1)//output
                           .build(Arc::clone(&device), descriptor_pool, num_sets);
    
    UpdateDescriptorSets::new()
      .add_storage_image(0, &dummy_image, ImageLayout::General)
      .add_storage_image(1, &dummy_image, ImageLayout::General)
      .finish_update(Arc::clone(&device), &descriptor_set);
    
    
  }
}
