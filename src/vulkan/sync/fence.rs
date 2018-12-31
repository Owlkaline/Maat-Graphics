use vk;

use crate::vulkan::Device;
use crate::vulkan::ownage::check_errors;

use std::mem;
use std::ptr;

pub struct Fence {
  fence: vk::Fence
}

impl Fence {
  pub fn new(device: &Device) -> Fence {
    let fence_info = vk::FenceCreateInfo {
      sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
      pNext: ptr::null(),
      flags: vk::FENCE_CREATE_SIGNALED_BIT,
    };
    
    let vk = device.pointers();
    let device = device.internal_object();
    
    let mut fence: vk::Fence = unsafe { mem::uninitialized() };
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
  
  pub fn reset(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.internal_object();
    unsafe {
      check_errors(vk.ResetFences(*device, 1, &self.fence));
    }
  }
  
  pub fn wait(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.internal_object();
    unsafe {
      check_errors(vk.WaitForFences(*device, 1, &self.fence, vk::TRUE, u64::max_value()));
    }
  }
  
  pub fn destroy(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.internal_object();
    unsafe {
      vk.DestroyFence(*device, self.fence, ptr::null());
    }
  }
}
