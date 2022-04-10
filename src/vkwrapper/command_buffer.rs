use ash::vk;

use crate::vkwrapper::{
  Buffer, ClearValues, DescriptorSet, Fence, Renderpass, Scissors, Semaphore, Shader, Viewport,
  VkCommandPool, VkDevice,
};

pub struct CommandBuffer {
  cmd: vk::CommandBuffer,
  reuse_fence: Fence,
}

impl CommandBuffer {
  pub fn new_one_time_submit(device: &VkDevice, pool: &VkCommandPool) -> CommandBuffer {
    let cmd = pool.allocate_primary_command_buffers(&device, 1)[0];
    let reuse_fence = Fence::new_signaled(&device);

    CommandBuffer { cmd, reuse_fence }
  }

  pub fn internal(&self) -> vk::CommandBuffer {
    self.cmd
  }

  pub fn begin(&mut self, device: &VkDevice) {
    let command_buffer_begin_info =
      vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
      device
        .internal()
        .begin_command_buffer(self.cmd, &command_buffer_begin_info)
        .expect("Begin commandbuffer");
    }
  }

  pub fn begin_renderpass(
    &mut self,
    device: &VkDevice,
    clear_values: &ClearValues,
    renderpass: &Renderpass,
    framebuffer: vk::Framebuffer,
    swapchain_extent: vk::Extent2D,
  ) {
    let clear_values = clear_values.build();
    let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
      .render_pass(renderpass.internal())
      .framebuffer(framebuffer)
      .render_area(vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain_extent,
      })
      .clear_values(&clear_values);

    unsafe {
      device.internal().cmd_begin_render_pass(
        self.cmd,
        &renderpass_begin_info,
        vk::SubpassContents::INLINE,
      );
    }
  }

  pub fn bind_descriptor_sets<T: Copy>(
    &mut self,
    device: &VkDevice,
    shader: &Shader<T>,
    slot: u32,
    descriptors: Vec<&DescriptorSet>,
    is_compute: bool,
  ) {
    unsafe {
      device.internal().cmd_bind_descriptor_sets(
        self.cmd,
        if is_compute {
          vk::PipelineBindPoint::COMPUTE
        } else {
          vk::PipelineBindPoint::GRAPHICS
        },
        shader.pipeline_layout(),
        slot,
        &descriptors
          .iter()
          .map(|ds| ds.internal()[0])
          .collect::<Vec<vk::DescriptorSet>>(),
        &[],
      );
    }
  }

  pub fn bind_graphics_pipeline<T: Copy>(&mut self, device: &VkDevice, shader: &Shader<T>) {
    unsafe {
      device.internal().cmd_bind_pipeline(
        self.cmd,
        vk::PipelineBindPoint::GRAPHICS,
        *shader.graphics_pipeline().internal(),
      );
    }
  }

  pub fn set_viewport(&mut self, device: &VkDevice, viewports: Vec<&Viewport>) {
    unsafe {
      device.internal().cmd_set_viewport(
        self.cmd,
        0,
        &viewports
          .iter()
          .map(|vp| vp.build())
          .collect::<Vec<vk::Viewport>>(),
      );
    }
  }

  pub fn set_scissors(&mut self, device: &VkDevice, scissors: Vec<&Scissors>) {
    unsafe {
      device.internal().cmd_set_scissor(
        self.cmd,
        0,
        &scissors
          .iter()
          .map(|s| s.build())
          .flatten()
          .collect::<Vec<vk::Rect2D>>(),
      );
    }
  }

  pub fn bind_vertex<T: Copy>(&mut self, device: &VkDevice, slot: u32, buffer: &Buffer<T>) {
    unsafe {
      device
        .internal()
        .cmd_bind_vertex_buffers(self.cmd, slot, &[*buffer.internal()], &[0]);
    }
  }

  pub fn bind_index<T: Copy>(&mut self, device: &VkDevice, buffer: &Buffer<T>) {
    unsafe {
      device.internal().cmd_bind_index_buffer(
        self.cmd,
        *buffer.internal(),
        0,
        vk::IndexType::UINT32,
      );
    }
  }

  pub fn push_constants<T: Copy>(
    &mut self,
    device: &VkDevice,
    shader: &Shader<T>,
    stage: vk::ShaderStageFlags,
    data: Vec<f32>,
  ) {
    let mut constant_data: [u8; 128] = [0; 128];

    for i in 0..(32).min(data.len()) {
      let bytes = data[i].to_le_bytes();
      for j in 0..4 {
        constant_data[i * 4 + j] = bytes[j];
      }
    }

    unsafe {
      device.internal().cmd_push_constants(
        self.cmd,
        shader.pipeline_layout(),
        stage,
        0,
        &constant_data,
      );
    }
  }

  pub fn draw_indexed(
    &mut self,
    device: &VkDevice,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    first_instance: u32,
  ) {
    unsafe {
      device.internal().cmd_draw_indexed(
        self.cmd,
        index_count,
        instance_count,
        first_index,
        0,
        first_instance,
      );
    }
  }

  pub fn draw_indexed_instanced<T: Copy>(
    &mut self,
    device: &VkDevice,
    instance_count: u32,
    buffer: &Buffer<T>,
  ) {
    self.draw_indexed(device, buffer.data().len() as u32, instance_count, 0, 0);
  }

  pub fn draw_indexed_buffer<T: Copy>(&mut self, device: &VkDevice, buffer: &Buffer<T>) {
    self.draw_indexed(device, buffer.data().len() as u32, 1, 0, 0);
  }

  pub fn draw_buffer<T: Copy>(&mut self, device: &VkDevice, buffer: &Buffer<T>) {
    unsafe {
      //command_buffer: CommandBuffer,
      //vertex_count: u32,
      //instance_count: u32,
      //first_vertex: u32,
      //first_instance: u32
      device
        .internal()
        .cmd_draw(self.cmd, buffer.data().len() as u32, 1, 0, 0);
    }
  }

  pub fn end_renderpass(&mut self, device: &VkDevice) {
    unsafe {
      device.internal().cmd_end_render_pass(self.cmd);
    }
  }

  pub fn end(&mut self, device: &VkDevice) {
    unsafe {
      device
        .internal()
        .end_command_buffer(self.cmd)
        .expect("Failed to end command buffer");
    }
  }

  pub fn reset(&mut self, device: &VkDevice) {
    self.reuse_fence.wait(device);
    self.reuse_fence.reset(device);

    unsafe {
      device
        .internal()
        .reset_command_buffer(self.cmd, vk::CommandBufferResetFlags::RELEASE_RESOURCES)
        .expect("Reset command buffer failed.");
    }
  }

  pub fn submit_queue(
    &mut self,
    device: &VkDevice,
    wait_semaphores: Vec<&Semaphore>,
    signal_semaphores: Vec<&Semaphore>,
    wait_stages: Vec<vk::PipelineStageFlags>,
    is_compute: bool,
  ) {
    let command_buffers = vec![self.cmd];

    let wait_semaphore = &wait_semaphores
      .iter()
      .map(|s| s.internal())
      .collect::<Vec<vk::Semaphore>>();
    let signal_semaphore = &signal_semaphores
      .iter()
      .map(|s| s.internal())
      .collect::<Vec<vk::Semaphore>>();

    let submit_queue = {
      if is_compute {
        device.compute_queue()
      } else {
        device.present_queue()
      }
    };

    let submit_info = vk::SubmitInfo::builder()
      .wait_semaphores(&wait_semaphore)
      .wait_dst_stage_mask(&wait_stages)
      .command_buffers(&command_buffers)
      .signal_semaphores(&signal_semaphore);

    unsafe {
      device
        .internal()
        .queue_submit(submit_queue, &[*submit_info], self.reuse_fence.internal())
        .expect("queue submit failed.");
    }
  }
}
