use vk;

use crate::vulkan::Device;

use std::ptr;
use std::sync::Arc;

pub struct DescriptorPool {
  pool: vk::DescriptorPool
}

impl DescriptorPool {
  pub fn with_internals(pool: vk::DescriptorPool) -> DescriptorPool {
    DescriptorPool {
      pool,
    }
  }
  
  pub fn local_pool(&self) -> &vk::DescriptorPool {
    &self.pool
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    println!("Destroying Descriptor Pool");
    
    unsafe {
      vk.DestroyDescriptorPool(*device, self.pool, ptr::null());
    }
  }
}
