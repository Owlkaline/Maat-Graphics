use crate::vkwrapper::{CommandBuffer, Semaphore, VkCommandPool, VkDevice};

pub struct Frame {
  present_semaphore: Semaphore,
  render_semaphore: Semaphore,
  pool: VkCommandPool,
  command_buffer: CommandBuffer,
}

impl Frame {
  pub fn new(device: &VkDevice) -> Frame {
    let pool = VkCommandPool::new(&device);
    let command_buffer = CommandBuffer::new_one_time_submit(device, &pool);

    Frame {
      present_semaphore: Semaphore::new(device),
      render_semaphore: Semaphore::new(device),
      pool,
      command_buffer,
    }
  }

  pub fn borrow_all(&mut self) -> (&mut CommandBuffer, &Semaphore, &Semaphore) {
    (
      &mut self.command_buffer,
      &self.present_semaphore,
      &self.render_semaphore,
    )
  }

  pub fn present_semaphore(&self) -> &Semaphore {
    &self.present_semaphore
  }

  pub fn render_semaphore(&self) -> &Semaphore {
    &self.render_semaphore
  }

  pub fn command_buffer(&mut self) -> &mut CommandBuffer {
    &mut self.command_buffer
  }
}
