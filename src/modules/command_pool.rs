use ash::vk;
use ash::version::DeviceV1_0;

use crate::modules::VkDevice;

pub struct VkCommandPool {
  pool: vk::CommandPool,
}

impl VkCommandPool {
  pub fn new(device: &VkDevice) -> VkCommandPool {
    let pool_create_info = vk::CommandPoolCreateInfo::builder()
                              .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                              .queue_family_index(device.queue_family_index());
    
    let pool = unsafe { device.internal().create_command_pool(&pool_create_info, None).unwrap() };
    
    VkCommandPool {
      pool,
    }
  }
  
  pub fn allocate_primary_command_buffers(&self, device: &VkDevice, count: u32) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                                          .command_buffer_count(count)
                                          .command_pool(self.pool)
                                          .level(vk::CommandBufferLevel::PRIMARY);

    let command_buffers = unsafe {
        device.internal()
            .allocate_command_buffers(&command_buffer_allocate_info)
            .unwrap()
    };
    
    command_buffers
  }
}
