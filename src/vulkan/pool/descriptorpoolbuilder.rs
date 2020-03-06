use vk;

use crate::vulkan::vkenums::{DescriptorType};

use crate::vulkan::pool::{DescriptorPool};

use crate::vulkan::{check_errors, Device};

use std::mem;
use std::ptr;
use std::sync::Arc;

pub struct DescriptorPoolBuilder {
  num_combined_image_samplers: u32,
  num_sampled_images: u32,
  num_storage_images: u32,
  num_uniform_buffers: u32,
  num_storage_buffers: u32,
  num_input_attachments: u32,
}

impl DescriptorPoolBuilder {
  pub fn new() -> DescriptorPoolBuilder {
    DescriptorPoolBuilder {
      num_combined_image_samplers: 0,
      num_sampled_images: 0,
      num_storage_images: 0,
      num_uniform_buffers: 0,
      num_storage_buffers: 0,
      num_input_attachments: 0,
    }
  }
  
  pub fn add_combined_image_samplers(mut self, num: u32) -> DescriptorPoolBuilder {
    self.num_combined_image_samplers += num;
    self
  }
  
  pub fn add_sampled_images(mut self, num: u32) -> DescriptorPoolBuilder {
    self.num_sampled_images += num;
    self
  }
  
  pub fn add_storage_images(mut self, num: u32) -> DescriptorPoolBuilder {
    self.num_storage_images += num;
    self
  }
  
  pub fn add_uniform_buffers(mut self, num: u32) -> DescriptorPoolBuilder {
    self.num_uniform_buffers += num;
    self
  }
  
  pub fn add_storage_buffers(mut self, num: u32) -> DescriptorPoolBuilder {
    self.num_storage_buffers += num;
    self
  }
  
  pub fn add_input_attachments(mut self, num: u32) -> DescriptorPoolBuilder {
    self.num_input_attachments += num;
    self
  }
  
  pub fn build(&self, device: Arc<Device>, num_sets: u32) -> DescriptorPool {
    let max_sets = num_sets * self.num_uniform_buffers
                                .max(self.num_sampled_images)
                                .max(self.num_combined_image_samplers)
                                .max(self.num_storage_images)
                                .max(self.num_storage_buffers)
                                .max(self.num_input_attachments);
    
    let mut descriptor_pool: vk::DescriptorPool = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let mut descriptor_pool_size: Vec<vk::DescriptorPoolSize> = Vec::with_capacity((self.num_uniform_buffers + self.num_sampled_images + self.num_combined_image_samplers + self.num_storage_images + self.num_storage_buffers + self.num_input_attachments) as usize);
    
    for _ in 0..self.num_combined_image_samplers {
      descriptor_pool_size.push(
        vk::DescriptorPoolSize {
          ty: DescriptorType::CombinedImageSampler.to_bits(),
          descriptorCount: max_sets,
        }
      );
    }
    
    for _ in 0..self.num_sampled_images {
      descriptor_pool_size.push(
        vk::DescriptorPoolSize {
          ty: DescriptorType::SampledImage.to_bits(),
          descriptorCount: max_sets,
        }
      );
    }
    
    for _ in 0..self.num_storage_images {
      descriptor_pool_size.push(
        vk::DescriptorPoolSize {
          ty: DescriptorType::StorageImage.to_bits(),
          descriptorCount: max_sets,
        }
      );
    }
    
    for _ in 0..self.num_uniform_buffers {
      descriptor_pool_size.push(
        vk::DescriptorPoolSize {
          ty: DescriptorType::UniformBuffer.to_bits(),
          descriptorCount: max_sets,
        }
      );
    }
    
    for _ in 0..self.num_storage_buffers {
      descriptor_pool_size.push(
        vk::DescriptorPoolSize {
          ty: DescriptorType::StorageBuffer.to_bits(),
          descriptorCount: max_sets,
        }
      );
    }
    
    for _ in 0..self.num_input_attachments {
      descriptor_pool_size.push(
        vk::DescriptorPoolSize {
          ty: DescriptorType::InputAttachment.to_bits(),
          descriptorCount: max_sets,
        }
      );
    }
    
    let descriptor_pool_create_info = {
      vk::DescriptorPoolCreateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        maxSets: descriptor_pool_size.len() as u32*max_sets,
        poolSizeCount: descriptor_pool_size.len() as u32,
        pPoolSizes: descriptor_pool_size.as_ptr(),
      }
    };
    
    let vk = device.pointers();
    let device = device.internal_object();
    
    unsafe {
      check_errors(vk.CreateDescriptorPool(*device, &descriptor_pool_create_info, ptr::null(), &mut descriptor_pool));
    }
    
    DescriptorPool::with_internals(descriptor_pool)
  }
}
