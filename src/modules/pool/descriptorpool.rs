use vk;

use crate::modules::Device;
use crate::ownage::check_errors; 

use std::mem;
use std::ptr;

pub struct DescriptorPool {
  pool: vk::DescriptorPool
}

impl DescriptorPool {
  pub fn new(device: &Device, max_sets: u32, num_uniforms: u32, num_images: u32) -> DescriptorPool {
    let mut descriptor_pool: vk::DescriptorPool = unsafe { mem::uninitialized() };
    let mut descriptor_pool_size: Vec<vk::DescriptorPoolSize> = Vec::with_capacity((num_uniforms + num_images) as usize);
    
    if num_uniforms > 0 {
      descriptor_pool_size.push(
        vk::DescriptorPoolSize {
          ty: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
          descriptorCount: max_sets,
        }
      );
    }
    
    if num_images > 0 {
      descriptor_pool_size.push(
        vk::DescriptorPoolSize {
          ty: vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
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
    
    DescriptorPool {
      pool: descriptor_pool,
    }
  }
  
  pub fn local_pool(&self) -> &vk::DescriptorPool {
    &self.pool
  }
  
  pub fn destroy(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    println!("Destroying Descriptor Pool");
    
    unsafe {
      vk.DestroyDescriptorPool(*device, self.pool, ptr::null());
    }
  }
}
