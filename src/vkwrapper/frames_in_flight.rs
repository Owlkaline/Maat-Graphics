use ash::vk;

use crate::vkwrapper::{Fence, Semaphore, VkCommandPool, VkDevice};

pub struct Frame {
  present_semaphore: Semaphore,
  render_semaphore: Semaphore,
  render_fence: Fence,

  pool: VkCommandPool,
  command_buffer: vk::CommandBuffer,
}

impl Frame {
  pub fn new(device: &VkDevice) -> Frame {
    let pool = VkCommandPool::new(&device);
    let command_buffer = pool.allocate_primary_command_buffers(&device, 1)[0];

    Frame {
      present_semaphore: Semaphore::new(device),
      render_semaphore: Semaphore::new(device),
      render_fence: Fence::new_signaled(device),

      pool,
      command_buffer,
    }
  }

  pub fn present_semaphore(&mut self) -> &mut Semaphore {
    &mut self.present_semaphore
  }

  pub fn render_semaphore(&mut self) -> &mut Semaphore {
    &mut self.render_semaphore
  }

  pub fn render_fence(&mut self) -> &mut Fence {
    &mut self.render_fence
  }

  pub fn command_buffer(&self) -> vk::CommandBuffer {
    self.command_buffer
  }
}
