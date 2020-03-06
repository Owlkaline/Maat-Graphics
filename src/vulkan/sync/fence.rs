use vk;

use crate::vulkan::Device;
use crate::vulkan::check_errors;

use std::mem;
use std::ptr;
use std::sync::Arc;

pub struct Fence {
  fence: vk::Fence
}

impl Fence {
  pub fn new(device: Arc<Device>) -> Fence {
    let fence_info = vk::FenceCreateInfo {
      sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
      pNext: ptr::null(),
      flags: vk::FENCE_CREATE_SIGNALED_BIT,
    };
    
    let vk = device.pointers();
    let device = device.internal_object();
    
    let mut fence: vk::Fence = unsafe { mem::MaybeUninit::uninit().assume_init() };
    unsafe {
      check_errors(vk.CreateFence(*device, &fence_info, ptr::null(), &mut fence));
    }
    
    Fence {
      fence
    }
  }
  
  pub fn internal_object(&self) -> &vk::Fence {
    &self.fence
  }
  
  pub fn ready(&self, device: Arc<Device>) -> bool {
    let vk = device.pointers();
    let device = device.internal_object();
    
    let result;
    
    unsafe {
      result = vk.GetFenceStatus(*device, self.fence);
    }
    
    match result {
      vk::SUCCESS => true,
      vk::NOT_READY => false,
      _ => {true}
    }
  }
  
  pub fn reset(&self, device: Arc<Device>) {
    let vk = device.pointers();
    let device = device.internal_object();
    unsafe {
      check_errors(vk.ResetFences(*device, 1, &self.fence));
    }
  }
  
  pub fn wait(&self, device: Arc<Device>) {
    let vk = device.pointers();
    let device = device.internal_object();
    unsafe {
      check_errors(vk.WaitForFences(*device, 1, &self.fence, vk::TRUE, u64::max_value()));
    }
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    let vk = device.pointers();
    let device = device.internal_object();
    unsafe {
      vk.DestroyFence(*device, self.fence, ptr::null());
    }
  }
}
