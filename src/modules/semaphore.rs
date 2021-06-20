use ash::vk;
use ash::version::DeviceV1_0;

use crate::modules::VkDevice;

pub struct Semaphore {
  semaphore: vk::Semaphore,
}

impl Semaphore {
  pub fn new(device: &VkDevice) -> Semaphore {
    
    let semaphore_create_info = vk::SemaphoreCreateInfo::default();
    
    let semaphore: vk::Semaphore;
    
    unsafe {
      semaphore = device.internal().create_semaphore(&semaphore_create_info, None).expect("Create Semaphore failed.");
    }
    
    Semaphore {
      semaphore,
    }
  }
  
  pub fn internal(&self) -> vk::Semaphore {
    self.semaphore
  }
}
