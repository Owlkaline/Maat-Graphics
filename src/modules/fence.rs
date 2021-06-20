use ash::vk;
use ash::version::DeviceV1_0;

use crate::modules::VkDevice;

pub struct Fence {
  fence: vk::Fence,
}

impl Fence {
  pub fn new_signaled(device: &VkDevice) -> Fence {
    
    let fence_create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    
    let fence: vk::Fence;
    
    unsafe {
      fence = device.internal().create_fence(&fence_create_info, None).expect("Create fence failed.");
    }
    
    Fence {
      fence,
    }
  }
  
  pub fn new_many_signaled(device: &VkDevice, count: u32) -> Vec<Fence> {
    let mut fences = Vec::new();
    
    for i in 0..count {
      fences.push(Fence::new_signaled(device));
    }
    
    fences
  }
  
  pub fn wait(&self, device: &VkDevice) {
    unsafe {
      device.wait_for_fences(&[self.fence], true, std::u64::MAX).expect("Wait for fence failed.");
    }
  }
  
  pub fn reset(&self, device: &VkDevice) {
    unsafe {
      device.reset_fences(&[self.fence]).expect("Reset fences failed.");
    }
  }
  
  pub fn internal(&self) -> vk::Fence {
    self.fence
  }
}
