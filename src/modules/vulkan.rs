use ash::version::{DeviceV1_0};
use ash::{vk};
use std::default::Default;

use crate::modules::{VkDevice, VkInstance, VkCommandPool, VkSwapchain, VkFrameBuffer, Scissors, 
                     ClearValues, Viewport, Fence, Semaphore, ImageBuilder, Image, Renderpass, 
                     PassDescription, VkWindow, Buffer, Shader, DescriptorSet,
                     ComputeShader, DescriptorWriter};
use crate::shader_handlers::gltf_loader::{GltfModel, Node, MeshImage, Skin, Material, Texture};

// Simple offset_of macro akin to C++ offsetof
#[macro_export]
macro_rules! offset_of {
  ($base:path, $field:ident) => {{
    #[allow(unused_unsafe)]
    unsafe {
      let b: $base = mem::zeroed();
      (&b.$field as *const _ as isize) - (&b as *const _ as isize)
    }
  }};
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

pub struct Vulkan {
  instance: VkInstance,
  device: VkDevice,
  
  texture_renderpass: Renderpass,
  model_renderpass: Renderpass,
  framebuffer: VkFrameBuffer,
  
  swapchain: VkSwapchain,
  
  pool: VkCommandPool,
  
  pub draw_command_buffer: vk::CommandBuffer,
  pub setup_command_buffer: vk::CommandBuffer,
  
  depth_image: Image,

  present_complete_semaphore: Semaphore,
  rendering_complete_semaphore: Semaphore,

  draw_commands_reuse_fence: Fence,
  setup_commands_reuse_fence: Fence,
  
  scissors: Scissors,
  clear_values: ClearValues,
  viewports: Viewport,
}

impl Vulkan {
  pub fn new(window: &mut VkWindow, screen_resolution: vk::Extent2D) -> Vulkan {
    
    let instance = VkInstance::new(window);
    let device = VkDevice::new(&instance, window);
    
    let mut swapchain = VkSwapchain::new(&instance, &device, screen_resolution);
    
    let pool = VkCommandPool::new(&device);
    let command_buffers = pool.allocate_primary_command_buffers(&device, 2);
    
    let setup_command_buffer = command_buffers[0];
    let draw_command_buffer = command_buffers[1];
    
    let extent = swapchain.extent();
    let depth_image = ImageBuilder::new_depth(extent.width, extent.height,
                                              1, 1, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                                    .build_device_local(&device);
    let passes = vec![
      PassDescription::new(device.surface_format().format)
                       .samples_1()
                       .attachment_load_op_load()
                       .attachment_store_op_store()
                       .attachment_layout_colour()
                       .initial_layout_present_src()
                       .final_layout_present_src(),
      PassDescription::new(vk::Format::D16_UNORM)
                       .samples_1()
                       .attachment_load_op_clear()
                       .attachment_layout_depth_stencil()
                       .initial_layout_undefined()
                       .final_layout_depth_stencil()
    ];
    
    let texture_renderpass = Renderpass::new(&device, passes);
    
    let passes = vec![
      PassDescription::new(device.surface_format().format)
                       .samples_1()
                       .attachment_load_op_clear()
                       .attachment_store_op_store()
                       .attachment_layout_colour()
                       .initial_layout_undefined()
                       .final_layout_present_src(),
      PassDescription::new(vk::Format::D16_UNORM)
                       .samples_1()
                       .attachment_load_op_clear()
                       .attachment_layout_depth_stencil()
                       .stencil_load_op_clear()
                       .initial_layout_undefined()
                       .final_layout_depth_stencil()
    ];
    let model_renderpass = Renderpass::new(&device, passes);
    
    let framebuffer = VkFrameBuffer::new(&device, &mut swapchain, &depth_image, &texture_renderpass);
    
    let draw_commands_reuse_fence = Fence::new_signaled(&device);
    let setup_commands_reuse_fence = Fence::new_signaled(&device);
    
    let present_complete_semaphore = Semaphore::new(&device);
    let rendering_complete_semaphore = Semaphore::new(&device);
    
    let clear_values = ClearValues::new().add_colour(0.2, 0.2, 0.2, 0.0).add_depth(1.0, 0);
    let scissors = Scissors::new().add_scissor(0, 0, extent.width, extent.height);
    
    let viewports = Viewport::new(0.0, extent.height as f32, 
                                  extent.width as f32,
                                  -(extent.height as f32),
                                  0.0, 1.0);

    Vulkan {
        instance,
        device,
        texture_renderpass,
        model_renderpass,
        swapchain,
        pool,
        
        draw_command_buffer,
        setup_command_buffer,
        depth_image,
        present_complete_semaphore,
        rendering_complete_semaphore,
        draw_commands_reuse_fence,
        setup_commands_reuse_fence,
        viewports,
        framebuffer,
        scissors,
        clear_values,
    }
  }
  
  pub fn swapchain(&mut self) -> &mut VkSwapchain {
    &mut self.swapchain
  }
  
  pub fn texture_renderpass(&self) -> &Renderpass {
    &self.texture_renderpass
  }
  
  pub fn model_renderpass(&self) -> &Renderpass {
    &self.texture_renderpass
  }
  
  pub fn scissors(&self) -> &Scissors {
    &self.scissors
  }
  
  pub fn viewports(&self) -> &Viewport {
    &self.viewports
  }
  
  pub fn recreate_swapchain(&mut self) {
    unsafe {
      let device = self.device.internal();
      
      device.device_wait_idle().unwrap();
      
      self.framebuffer.destroy(device);

      device.destroy_image_view(self.depth_image.view(), None);
      device.destroy_image(self.depth_image.internal(), None);
      device.free_memory(self.depth_image.memory(), None);
    }
    self.swapchain.destroy(&self.device);
    
    self.swapchain.recreate(&self.instance, &self.device);
    let extent = self.swapchain.extent();
    
    self.depth_image = ImageBuilder::new_depth(extent.width, extent.height,
                                              1, 1, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                                     .build_device_local(&self.device);
    
    self.framebuffer = VkFrameBuffer::new(&self.device, &mut self.swapchain, &self.depth_image, &self.texture_renderpass);

    self.scissors = Scissors::new().add_scissor(0, 0, extent.width, extent.height);

    self.viewports = Viewport::new(0.0, extent.height as f32, 
                                   extent.width as f32,
                                   -(extent.height as f32),
                                   0.0, 1.0);
  }
  
  pub fn render_texture<T: Copy, L: Copy>(
    &mut self,
    descriptor_sets: &DescriptorSet,//&Vec<vk::DescriptorSet>,
    shader: &Shader<T>,
    vertex_buffer: &Buffer<T>,
    index_buffer: &Buffer<L>,
  ) {
    let present_index_result = unsafe {
      self.swapchain.swapchain_loader()
          .acquire_next_image(
              *self.swapchain.internal(),
              std::u64::MAX,
              self.present_complete_semaphore.internal(),
              vk::Fence::null(),
          )
    };
    let (present_index, _) = match present_index_result {
      Ok(index) => index,
      Err(_) => {
        self.recreate_swapchain();
        return;
      }
    };
    
    let clear_values = self.clear_values.build();
    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(self.texture_renderpass.internal())
        .framebuffer(self.framebuffer.framebuffers()[present_index as usize])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent(),
        })
        .clear_values(&clear_values);

    Vulkan::record_submit_commandbuffer(
      &self.device,
      self.draw_command_buffer,
      &self.draw_commands_reuse_fence,
      self.device.present_queue(),
      &[vk::PipelineStageFlags::BOTTOM_OF_PIPE],
      &self.present_complete_semaphore,
      &self.rendering_complete_semaphore,
      |device, draw_command_buffer| { unsafe {
        device.cmd_begin_render_pass(
          draw_command_buffer,
          &render_pass_begin_info,
          vk::SubpassContents::INLINE,
        );
        
        device.cmd_bind_descriptor_sets(
          draw_command_buffer,
          vk::PipelineBindPoint::GRAPHICS,
          shader.pipeline_layout(),
          0,
          &descriptor_sets.internal()[..],
          &[],
        );
        
        device.cmd_bind_pipeline(
          draw_command_buffer,
          vk::PipelineBindPoint::GRAPHICS,
          *shader.graphics_pipeline().internal(),
        );
        
        device.cmd_set_viewport(draw_command_buffer, 0, &[self.viewports.build()]);
        device.cmd_set_scissor(draw_command_buffer, 0, &self.scissors.build());
        
        device.cmd_bind_vertex_buffers(
          draw_command_buffer,
          0,
          &[*vertex_buffer.internal()],
          &[0],
        );
        
        device.cmd_bind_index_buffer(
          draw_command_buffer,
          *index_buffer.internal(),
          0,
          vk::IndexType::UINT32,
        );
        
        device.cmd_draw_indexed(
          draw_command_buffer,
          index_buffer.data().len() as u32,
          1,
          0,
          0,
          1,
        );
        
        // Or draw without the index buffer
        // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
        device.cmd_end_render_pass(draw_command_buffer);
      }},
    );

    let present_info = vk::PresentInfoKHR {
      wait_semaphore_count: 1,
      p_wait_semaphores: &self.rendering_complete_semaphore.internal(),
      swapchain_count: 1,
      p_swapchains: self.swapchain.internal(),
      p_image_indices: &present_index,
      ..Default::default()
    };
    
    unsafe {
        match self.swapchain.swapchain_loader()
            .queue_present(self.device.present_queue(), &present_info) {
        Ok(_) => {
          
        },
        Err(vk_e) => {
          match vk_e {
            vk::Result::ERROR_OUT_OF_DATE_KHR => { //VK_ERROR_OUT_OF_DATE_KHR
              self.recreate_swapchain();
              return;
            },
            e => {
              panic!("Error: {}", e);
            }
          }
        }
      }
    };
  }
  
  pub fn copy_buffer_to_device_local_image(&mut self, src_buffer: &Buffer<u8>, dst_image: &Image) {
    Vulkan::record_submit_commandbuffer(
      &self.device,
      self.setup_command_buffer,
      &self.setup_commands_reuse_fence,
      self.device.present_queue(),
      &[],
      &Semaphore::new(&self.device),//[],
      &Semaphore::new(&self.device),//[],
      |device, texture_command_buffer| {
        let texture_barrier = vk::ImageMemoryBarrier {
          dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
          new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
          image: dst_image.internal(),
          subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1,
            layer_count: 1,
            ..Default::default()
          },
          ..Default::default()
        };
        
        unsafe {
          device.cmd_pipeline_barrier(
            texture_command_buffer,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[texture_barrier],
          );
        }
        
        let mut buffer_copy_regions = Vec::new();
        
         buffer_copy_regions.push(vk::BufferImageCopy::builder()
            .image_subresource(
              vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .build(),
            )
            .image_extent(vk::Extent3D {
              width: dst_image.width(),//image_dimensions.0,
              height: dst_image.height(),//image_dimensions.1,
              depth: 1,
            }).build());
        
        unsafe {
          device.cmd_copy_buffer_to_image(
            texture_command_buffer,
            *src_buffer.internal(),
            dst_image.internal(),
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &buffer_copy_regions[..],//&[buffer_copy_regions.build()],
          );
        }
        
        let texture_barrier_end = vk::ImageMemoryBarrier {
          src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
          dst_access_mask: vk::AccessFlags::SHADER_READ,
          old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
          new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
          image: dst_image.internal(),
          subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1,
            layer_count: 1,
            ..Default::default()
          },
          ..Default::default()
        };
        
        unsafe {
          device.cmd_pipeline_barrier(
            texture_command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[texture_barrier_end],
          );
        }
      },
    );
  }
  
  pub fn copy_buffer_to_device_local_buffer<T: Copy>(&mut self, src_buffer: &Buffer<T>, dst_buffer: &Buffer<T>) {
    Vulkan::record_submit_commandbuffer(
      &self.device,
      self.setup_command_buffer,
      &self.setup_commands_reuse_fence,
      self.device.present_queue(),
      &[],
      &Semaphore::new(&self.device),//[],
      &Semaphore::new(&self.device),//[],
      |device, buffer_command_buffer| {
        /*let buffer_barrier = vk::BufferMemoryBarrier {
          dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
          buffer: dst_buffer.internal(),
          size: std::mem::size_of::<T>() as u64 * (src_buffer.data().len() as u64)
          ..Default::default()
        };*/
        let buffer_barrier = vk::BufferMemoryBarrier::builder()
                                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                                .buffer(*dst_buffer.internal())
                                .size(std::mem::size_of::<T>() as u64 * (src_buffer.data().len() as u64));
        
        unsafe {
          device.cmd_pipeline_barrier(
            buffer_command_buffer,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[buffer_barrier.build()],
            &[],
          );
        }
        
        let buffer_copy = vk::BufferCopy::builder()
                             .size(std::mem::size_of::<T>() as u64 * (src_buffer.data().len() as u64));
        /*
        let buffer_copy_regions = vk::BufferImageCopy::builder()
            .image_subresource(
              vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .build(),
            )
            .image_extent(vk::Extent3D {
              width: dst_image.width(),//image_dimensions.0,
              height: dst_image.height(),//image_dimensions.1,
              depth: 1,
            });*/
        
        unsafe {
          device.cmd_copy_buffer(
            buffer_command_buffer,
            *src_buffer.internal(),
            *dst_buffer.internal(),
            &[buffer_copy.build()],
          );
          /*device.cmd_copy_buffer_to_image(
            texture_command_buffer,
            *src_buffer.internal(),
            dst_image.internal(),
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[buffer_copy_regions.build()],
          );*/
        }
        
        let buffer_barrier_end = vk::BufferMemoryBarrier::builder()
                                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                                    .dst_access_mask(vk::AccessFlags::SHADER_READ)
                                    .buffer(*dst_buffer.internal())
                                    .size(std::mem::size_of::<T>() as u64 * (src_buffer.data().len() as u64));
        /*
        let buffer_barrier_end = vk::BufferMemoryBarrier {
          src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
          dst_access_mask: vk::AccessFlags::SHADER_READ,
          buffer: dst_buffer.internal(),
          ..Default::default()
        };*/
        
        unsafe {
          device.cmd_pipeline_barrier(
            buffer_command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[buffer_barrier_end.build()],
            &[],
          );
        }
      },
    );
  }
  
  pub fn run_compute<T: Copy>(&mut self, compute_shader: &ComputeShader, 
                              descriptor_sets: &DescriptorSet, data: &mut Vec<T>) {
    let src_buffer = Buffer::<T>::builder()
                                     .data(data.to_vec())
                                     .usage_transfer_src_dst()
                                     .memory_properties_host_visible_coherent()
                                     .build(&self.device);
    let dst_buffer = Buffer::<T>::builder()
                                     .data(data.to_vec())
                                     .usage_transfer_storage_src_dst()
                                     .memory_properties_host_visible_coherent()
                                     .build(&self.device);
    
    let descriptor_set_writer = DescriptorWriter::builder()
                                                .update_storage_buffer(&dst_buffer, 
                                                                       &descriptor_sets);
    descriptor_set_writer.build(&self.device);
    
    Vulkan::record_submit_commandbuffer(
      &self.device,
      self.setup_command_buffer,
      &self.setup_commands_reuse_fence,
      self.device.present_queue(),
      &[],
      &Semaphore::new(&self.device),//[],
      &Semaphore::new(&self.device),//[],
      |device, compute_command_buffer| {
        
        let buffer_copy = vk::BufferCopy::builder()
                             .size(std::mem::size_of::<T>() as u64 * (src_buffer.data().len() as u64));
        
        unsafe {
          device.cmd_copy_buffer(
            compute_command_buffer,
            *src_buffer.internal(),
            *dst_buffer.internal(),
            &[buffer_copy.build()],
          );
        }
        
        let buffer_barrier = vk::BufferMemoryBarrier::builder()
                                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                                .dst_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE)
                                .buffer(*dst_buffer.internal())
                                .size(std::mem::size_of::<T>() as u64 * (src_buffer.data().len() as u64));
        
        unsafe {
          device.cmd_pipeline_barrier(
            compute_command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[buffer_barrier.build()],
            &[],
          );
        }
        
        unsafe {
          device.cmd_bind_pipeline(
            compute_command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            *compute_shader.pipeline().internal(),
          );
          
          device.cmd_bind_descriptor_sets(
            compute_command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            compute_shader.pipeline_layout(),
            0,
            &descriptor_sets.internal()[..],
            &[],
          );
        }
        
        unsafe {
          device.cmd_dispatch(
            compute_command_buffer,
            src_buffer.data().len() as u32,
            1,
            1,
          )
        }
        
        let buffer_barrier_end = vk::BufferMemoryBarrier::builder()
                                    .src_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE)
                                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                                    .buffer(*dst_buffer.internal())
                                    .size(std::mem::size_of::<T>() as u64 * (data.len() as u64));
        
        unsafe {
          device.cmd_pipeline_barrier(
            compute_command_buffer,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[buffer_barrier_end.build()],
            &[],
          );
        }
        
        let buffer_copy = vk::BufferCopy::builder()
                             .size(std::mem::size_of::<T>() as u64 * (data.len() as u64));
        
        unsafe {
          device.cmd_copy_buffer(
            compute_command_buffer,
            *dst_buffer.internal(),
            *src_buffer.internal(),
            &[buffer_copy.build()],
          );
        }
      },
    );
    
    unsafe { self.device.internal().device_wait_idle().unwrap() };
    *data = src_buffer.retrieve_buffer_data(&self.device);
  }
  
  pub fn device(&self) -> &VkDevice {
    &self.device
  }
  
  /// Helper function for submitting command buffers. Immediately waits for the fence before the command buffer
  /// is executed. That way we can delay the waiting for the fences by 1 frame which is good for performance.
  /// Make sure to create the fence in a signaled state on the first use.
  #[allow(clippy::too_many_arguments)]
  pub fn record_submit_commandbuffer<F: FnOnce(&VkDevice, vk::CommandBuffer)>(
      device: &VkDevice,
      command_buffer: vk::CommandBuffer,
      command_buffer_reuse_fence: &Fence,
      submit_queue: vk::Queue,
      wait_mask: &[vk::PipelineStageFlags],
      wait_semaphores: &Semaphore,
      signal_semaphores: &Semaphore,
      f: F,
  ) {
    unsafe {
        command_buffer_reuse_fence.wait(device);
        command_buffer_reuse_fence.reset(device);
        
        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Reset command buffer failed.");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Begin commandbuffer");
        f(device, command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("End commandbuffer");

        let command_buffers = vec![command_buffer];
        
        let wait_semaphore = [wait_semaphores.internal()];
        let signal_semaphore = [signal_semaphores.internal()];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphore)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphore);

        device
            .queue_submit(
                submit_queue,
                &[submit_info.build()],
                command_buffer_reuse_fence.internal(),
            )
            .expect("queue submit failed.");
    }
  }
  
  pub fn start_texture_render<T: Copy>(&mut self, shader: &Shader<T>, uniform_descriptor: &DescriptorSet) -> Option<u32> {
    let present_index_result = unsafe {
      self.swapchain.swapchain_loader()
          .acquire_next_image(
              *self.swapchain.internal(),
              std::u64::MAX,
              self.present_complete_semaphore.internal(),
              vk::Fence::null(),
          )
    };
    
    let (present_index, _) = match present_index_result {
      Ok(index) => index,
      Err(_) => {
        self.recreate_swapchain();
        return None;
      }
    };
    
    let clear_values = self.clear_values.build();
    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(self.texture_renderpass.internal())
        .framebuffer(self.framebuffer.framebuffers()[present_index as usize])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent(),
        })
        .clear_values(&clear_values);
    
    unsafe {
      self.draw_commands_reuse_fence.wait(&self.device);
      self.draw_commands_reuse_fence.reset(&self.device);
      
      self.device
        .reset_command_buffer(
          self.draw_command_buffer,
          vk::CommandBufferResetFlags::RELEASE_RESOURCES,
        )
        .expect("Reset command buffer failed.");

      let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
          .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

      self.device
        .begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)
        .expect("Begin commandbuffer");
      
      self.device.cmd_begin_render_pass(
        self.draw_command_buffer,
        &render_pass_begin_info,
        vk::SubpassContents::INLINE,
      );
      
      self.device.cmd_bind_descriptor_sets(
        self.draw_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        shader.pipeline_layout(),
        0,
        &uniform_descriptor.internal()[..],
        &[],
      );
    }
    
    Some(present_index)
  }
  
  pub fn start_model_render(&mut self) -> Option<u32> {
    let present_index_result = unsafe {
      self.swapchain.swapchain_loader()
          .acquire_next_image(
              *self.swapchain.internal(),
              std::u64::MAX,
              self.present_complete_semaphore.internal(),
              vk::Fence::null(),
          )
    };
    
    let (present_index, _) = match present_index_result {
      Ok(index) => index,
      Err(_) => {
        self.recreate_swapchain();
        return None;
      }
    };
    
    let clear_values = self.clear_values.build();
    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(self.model_renderpass.internal())
        .framebuffer(self.framebuffer.framebuffers()[present_index as usize])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent(),
        })
        .clear_values(&clear_values);
    
    unsafe {
      self.draw_commands_reuse_fence.wait(&self.device);
      self.draw_commands_reuse_fence.reset(&self.device);
      
      self.device
        .reset_command_buffer(
          self.draw_command_buffer,
          vk::CommandBufferResetFlags::RELEASE_RESOURCES,
        )
        .expect("Reset command buffer failed.");

      let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
          .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

      self.device
        .begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)
        .expect("Begin commandbuffer");
      
      self.device.cmd_begin_render_pass(
        self.draw_command_buffer,
        &render_pass_begin_info,
        vk::SubpassContents::INLINE,
      );
    }
    
    Some(present_index)
  }
  
  pub fn end_render(&mut self, present_index: u32) {
    let wait_mask = &[vk::PipelineStageFlags::BOTTOM_OF_PIPE];
    let submit_queue = self.device.present_queue();
    
    unsafe {
      // Or draw without the index buffer
      // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
      self.device.cmd_end_render_pass(self.draw_command_buffer);
      
      self.device
        .end_command_buffer(self.draw_command_buffer)
        .expect("End commandbuffer");
    }
    
    let command_buffers = vec![self.draw_command_buffer];
    
    let wait_semaphore = [self.present_complete_semaphore.internal()];
    let signal_semaphore = [self.rendering_complete_semaphore.internal()];
    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(&wait_semaphore)
        .wait_dst_stage_mask(wait_mask)
        .command_buffers(&command_buffers)
        .signal_semaphores(&signal_semaphore);
    
    unsafe {
      self.device
        .queue_submit(
          submit_queue,
          &[submit_info.build()],
          self.draw_commands_reuse_fence.internal(),
        )
        .expect("queue submit failed.");
    }
    
    let present_info = vk::PresentInfoKHR {
      wait_semaphore_count: 1,
      p_wait_semaphores: &self.rendering_complete_semaphore.internal(),
      swapchain_count: 1,
      p_swapchains: self.swapchain.internal(),
      p_image_indices: &present_index,
      ..Default::default()
    };
    
    unsafe {
        match self.swapchain.swapchain_loader()
            .queue_present(self.device.present_queue(), &present_info) {
        Ok(_) => {
          
        },
        Err(vk_e) => {
          match vk_e {
            vk::Result::ERROR_OUT_OF_DATE_KHR => { //VK_ERROR_OUT_OF_DATE_KHR
              self.recreate_swapchain();
              return;
            },
            e => {
              panic!("Error: {}", e);
            }
          }
        }
      }
    };
  }
  
  pub fn draw_texture<T: Copy, L: Copy>(
    &mut self,
    texture_descriptor: &DescriptorSet, 
    shader: &Shader<T>, 
    vertex_buffer: &Buffer<T>, 
    index_buffer: &Buffer<L>,
    data: Vec<f32>,
  ) {
    
    let mut push_constant_data: [u8; 128] = [0; 128];
    for i in 0..(32).min(data.len()) {
      let bytes = data[i].to_le_bytes();
      push_constant_data[i*4 + 0] = bytes[0];
      push_constant_data[i*4 + 1] = bytes[1];
      push_constant_data[i*4 + 2] = bytes[2];
      push_constant_data[i*4 + 3] = bytes[3];
    }
    
    // Pass in window size
    // TODO: Move to specialisation constant or uniform buffer
    let width_bytes = self.viewports.width().to_le_bytes();
    push_constant_data[30*4 + 0] = width_bytes[0];
    push_constant_data[30*4 + 1] = width_bytes[1];
    push_constant_data[30*4 + 2] = width_bytes[2];
    push_constant_data[30*4 + 3] = width_bytes[3];
    let height_bytes = self.viewports.height().to_le_bytes();
    push_constant_data[31*4 + 0] = height_bytes[0];
    push_constant_data[31*4 + 1] = height_bytes[1];
    push_constant_data[31*4 + 2] = height_bytes[2];
    push_constant_data[31*4 + 3] = height_bytes[3];
    
    unsafe {
      self.device.cmd_bind_descriptor_sets(
        self.draw_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        shader.pipeline_layout(),
        1,
        &texture_descriptor.internal()[..],
        &[],
      );
      
      self.device.cmd_bind_pipeline(
        self.draw_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        *shader.graphics_pipeline().internal(),
      );
      
      self.device.cmd_set_viewport(self.draw_command_buffer, 0, &[self.viewports.build()]);
      self.device.cmd_set_scissor(self.draw_command_buffer, 0, &self.scissors.build());
      
      self.device.cmd_bind_vertex_buffers(
        self.draw_command_buffer,
        0,
        &[*vertex_buffer.internal()],
        &[0],
      );
      
      self.device.cmd_bind_index_buffer(
        self.draw_command_buffer,
        *index_buffer.internal(),
        0,
        vk::IndexType::UINT32,
      );
      
      self.device.cmd_push_constants(
        self.draw_command_buffer,
        shader.pipeline_layout(),
        vk::ShaderStageFlags::VERTEX,
        0,
        &push_constant_data);
      
      self.device.cmd_draw_indexed(
        self.draw_command_buffer,
        index_buffer.data().len() as u32,
        1,
        0,
        0,
        1,
      );
    }
  }
  
  pub fn draw_mesh<T: Copy>(
    &mut self,
    shader: &Shader<T>, 
    uniform_descriptor: &DescriptorSet,
    dummy_texture: &DescriptorSet,
    dummy_skin: &DescriptorSet,
    data: Vec<f32>,
    model: &GltfModel
  ) {
    unsafe {
      self.device.cmd_bind_descriptor_sets(
        self.draw_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        shader.pipeline_layout(),
        0,
        &uniform_descriptor.internal()[..],
        &[],
      );
      
      self.device.cmd_bind_pipeline(
        self.draw_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        *shader.graphics_pipeline().internal(),
      );
      
      self.device.cmd_set_viewport(self.draw_command_buffer, 0, &[self.viewports.build()]);
      self.device.cmd_set_scissor(self.draw_command_buffer, 0, &self.scissors.build());
    }
    
    unsafe {
      self.device.cmd_bind_vertex_buffers(
        self.draw_command_buffer,
        0,
        &[*model.vertex_buffer().internal()],
        &[0],
      );
      
      self.device.cmd_bind_index_buffer(
        self.draw_command_buffer,
        *model.index_buffer().internal(),
        0,
        vk::IndexType::UINT32,
      );
    }
    
    let mut byte_data = Vec::new();
    for i in 0..(data.len().min(16)) {
      let bytes = data[i].to_le_bytes();
      byte_data.push(bytes[0]);
      byte_data.push(bytes[1]);
      byte_data.push(bytes[2]);
      byte_data.push(bytes[3]);
    }
    
    for i in 0..model.nodes().len() {
      self.draw_node(shader, i, &byte_data,
                     model.nodes(), 
                     model.images(), 
                     &model.skins(), 
                     &model.textures(), 
                     &model.materials(), 
                     dummy_texture, dummy_skin);
    }
  }
  
  fn draw_node<T: Copy>(&self, shader: &Shader<T>, idx: usize, data: &Vec<u8>,
                        nodes: &Vec<Node>, images: &Vec<MeshImage>, skins: &Vec<Skin>,
                        textures: &Vec<Texture>, materials: &Vec<Material>,
                        dummy_texture: &DescriptorSet,
                        dummy_skin: &DescriptorSet) {
    if nodes[idx].mesh.primitives.len() > 0 {
      let mut push_constant_data: [u8; 128] = [0; 128];
      let matrix = Node::get_node_matrix(nodes, idx);
      
      for i in 0..matrix.len() {
        let bytes = matrix[i].to_le_bytes();
        push_constant_data[i*4 + 0] = bytes[0];
        push_constant_data[i*4 + 1] = bytes[1];
        push_constant_data[i*4 + 2] = bytes[2];
        push_constant_data[i*4 + 3] = bytes[3];
      }
      
      let current_idx = matrix.len()*4;
      for i in current_idx..(current_idx+data.len()).min(128) {
        push_constant_data[i] = data[i-current_idx];
      }
      
      unsafe {
        self.device.cmd_push_constants(
          self.draw_command_buffer,
          shader.pipeline_layout(),
          vk::ShaderStageFlags::VERTEX,
          0,
          &push_constant_data);
      }
      
      if skins.len() > 0 && nodes[idx].skin != -1 {
        unsafe {
          self.device.cmd_bind_descriptor_sets(
            self.draw_command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            shader.pipeline_layout(),
            1,
            &skins[nodes[idx].skin as usize].descriptor_set.internal()[..],
            &[],
          );
        }
      } else {
        unsafe {
          self.device.cmd_bind_descriptor_sets(
            self.draw_command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            shader.pipeline_layout(),
            1,
            &dummy_skin.internal()[..],
            &[],
          );
        }
      }
      
      for primitive in &nodes[idx].mesh.primitives {
        if primitive.index_count > 0 {
          let image_descriptor = {
            if images.len() == 0 {
              dummy_texture
            } else {
              let idx = textures[materials[primitive.material_index as usize].base_colour_texture_index as usize].image_index as usize;
              &images[idx].descriptor_set
            }
          };
          
          unsafe {
            self.device.cmd_bind_descriptor_sets(
              self.draw_command_buffer,
              vk::PipelineBindPoint::GRAPHICS,
              shader.pipeline_layout(),
              2,
              &image_descriptor.internal()[..],
              &[],
            );
            
            self.device.cmd_draw_indexed(
              self.draw_command_buffer,
              primitive.index_count,
              1,
              primitive.first_index,
              0,
              1,
            );
          }
        }
      }
    }
    
    for child_idx in &nodes[idx].children {
      self.draw_node(shader, *child_idx as usize, data, nodes,
                     images, skins, textures, materials, 
                     dummy_texture, dummy_skin);
    }
  }
  
  pub fn end_renderpass(&mut self) {
    unsafe {
      self.device.cmd_end_render_pass(self.draw_command_buffer);
    }
  }
  
  pub fn start_render(&mut self) -> Option<u32> {
    let present_index_result = unsafe {
      self.swapchain.swapchain_loader()
          .acquire_next_image(
              *self.swapchain.internal(),
              std::u64::MAX,
              self.present_complete_semaphore.internal(),
              vk::Fence::null(),
          )
    };
    
    let (present_index, _) = match present_index_result {
      Ok(index) => index,
      Err(_) => {
        self.recreate_swapchain();
        return None;
      }
    };
    
    unsafe {
      self.draw_commands_reuse_fence.wait(&self.device);
      self.draw_commands_reuse_fence.reset(&self.device);
      
      self.device
        .reset_command_buffer(
          self.draw_command_buffer,
          vk::CommandBufferResetFlags::RELEASE_RESOURCES,
        )
        .expect("Reset command buffer failed.");

      let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
          .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

      self.device
        .begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)
        .expect("Begin commandbuffer");
    }
    
    Some(present_index)
  }
  
   pub fn new_end_render(&mut self, present_index: u32) {
    let wait_mask = &[vk::PipelineStageFlags::BOTTOM_OF_PIPE];
    let submit_queue = self.device.present_queue();
    
    unsafe {
      // Or draw without the index buffer
      // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
      //self.device.cmd_end_render_pass(self.draw_command_buffer);
      
      self.device
        .end_command_buffer(self.draw_command_buffer)
        .expect("End commandbuffer");
    }
    
    let command_buffers = vec![self.draw_command_buffer];
    
    let wait_semaphore = [self.present_complete_semaphore.internal()];
    let signal_semaphore = [self.rendering_complete_semaphore.internal()];
    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(&wait_semaphore)
        .wait_dst_stage_mask(wait_mask)
        .command_buffers(&command_buffers)
        .signal_semaphores(&signal_semaphore);
    
    unsafe {
      self.device
        .queue_submit(
          submit_queue,
          &[submit_info.build()],
          self.draw_commands_reuse_fence.internal(),
        )
        .expect("queue submit failed.");
    }
    
    let present_info = vk::PresentInfoKHR {
      wait_semaphore_count: 1,
      p_wait_semaphores: &self.rendering_complete_semaphore.internal(),
      swapchain_count: 1,
      p_swapchains: self.swapchain.internal(),
      p_image_indices: &present_index,
      ..Default::default()
    };
    
    unsafe {
        match self.swapchain.swapchain_loader()
            .queue_present(self.device.present_queue(), &present_info) {
        Ok(_) => {
          
        },
        Err(vk_e) => {
          match vk_e {
            vk::Result::ERROR_OUT_OF_DATE_KHR => { //VK_ERROR_OUT_OF_DATE_KHR
              self.recreate_swapchain();
              return;
            },
            e => {
              panic!("Error: {}", e);
            }
          }
        }
      }
    };
  }
  
  pub fn begin_renderpass_texture(&mut self, present_index: u32) {
    let clear_values = self.clear_values.build();
    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(self.texture_renderpass.internal())
        .framebuffer(self.framebuffer.framebuffers()[present_index as usize])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent(),
        })
        .clear_values(&clear_values);
    
    unsafe {
      self.device.cmd_begin_render_pass(
        self.draw_command_buffer,
        &render_pass_begin_info,
        vk::SubpassContents::INLINE,
      );
    }
  }
  
  pub fn begin_renderpass_model(&mut self, present_index: u32) {
    let clear_values = self.clear_values.build();
    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(self.model_renderpass.internal())
        .framebuffer(self.framebuffer.framebuffers()[present_index as usize])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent(),
        })
        .clear_values(&clear_values);
    
    unsafe {
      self.device.cmd_begin_render_pass(
        self.draw_command_buffer,
        &render_pass_begin_info,
        vk::SubpassContents::INLINE,
      );
    }
  }
}


