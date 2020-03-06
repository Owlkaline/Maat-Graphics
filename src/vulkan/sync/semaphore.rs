use vk;

use crate::vulkan::Device;
use crate::vulkan::ownage::check_errors;

use std::mem;
use std::ptr;
use std::sync::Arc;

pub struct Semaphore {
  semaphore: vk::Semaphore,
}

impl Semaphore {
  pub fn new(device: Arc<Device>) -> Semaphore {
    let mut semaphore: vk::Semaphore = unsafe { mem::MaybeUninit::uninit().assume_init() };
    
    let semaphore_info = vk::SemaphoreCreateInfo {
      sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      check_errors(vk.CreateSemaphore(*device, &semaphore_info, ptr::null(), &mut semaphore));
    }
    
    Semaphore {
      semaphore,
    }
  }
  
  pub fn internal_object(&self) -> &vk::Semaphore {
    &self.semaphore
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    let vk = device.pointers();
    let device = device.internal_object();
    unsafe {
      vk.DestroySemaphore(*device, self.semaphore, ptr::null());
    }
  }
}
