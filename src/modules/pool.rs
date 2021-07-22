pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;

use crate::modules::VkDevice;

pub struct DescriptorPoolBuilder {
  uniform_buffers: u32,
  combined_image_samplers: u32,
  storages: u32,
}

impl DescriptorPoolBuilder {
  pub fn new() -> DescriptorPoolBuilder {
    DescriptorPoolBuilder {
      uniform_buffers: 0,
      combined_image_samplers: 0,
      storages: 0,
    }
  }

  pub fn num_uniform_buffers(mut self, num: u32) -> DescriptorPoolBuilder {
    self.uniform_buffers = num;
    self
  }

  pub fn num_combined_image_samplers(mut self, num: u32) -> DescriptorPoolBuilder {
    self.combined_image_samplers = num;
    self
  }

  pub fn num_storage(mut self, num: u32) -> DescriptorPoolBuilder {
    self.storages = num;
    self
  }

  pub fn build(&self, device: &VkDevice) -> vk::DescriptorPool {
    let mut descriptor_sizes = Vec::new();

    if self.uniform_buffers != 0 {
      descriptor_sizes.push(vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: self.uniform_buffers,
      });
    }

    if self.combined_image_samplers != 0 {
      descriptor_sizes.push(vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: self.combined_image_samplers,
      });
    }

    if self.storages != 0 {
      descriptor_sizes.push(vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: self.storages,
      });
    }

    let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
      .pool_sizes(&descriptor_sizes)
      .max_sets(self.storages + self.combined_image_samplers + self.uniform_buffers);

    unsafe {
      device
        .internal()
        .create_descriptor_pool(&descriptor_pool_info, None)
        .unwrap()
    }
  }
}
