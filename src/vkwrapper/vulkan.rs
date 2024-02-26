use std::default::Default;

use ash::vk;
use glam::{EulerRot, Quat, Vec3};

use crate::extra::gltf_loader::{GltfModel, Material, Node, Skin};
use crate::vkwrapper::{
  Buffer, ClearValues, CommandBuffer, ComputeShader, DescriptorSet, DescriptorWriter, Frame, Image,
  ImageBuilder, PassDescription, Renderpass, Scissors, Semaphore, Shader, Viewport, VkCommandPool,
  VkDevice, VkFrameBuffer, VkInstance, VkSwapchain, VkWindow,
};
use winit::event_loop::EventLoop;

const FRAMES_IN_FLIGHT: usize = 2;

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
      (1 << index) & memory_req.memory_type_bits != 0 && memory_type.property_flags & flags == flags
    })
    .map(|(index, _memory_type)| index as _)
}

pub struct Vulkan {
  instance: VkInstance,
  device: VkDevice,

  current_frame: usize,
  frames_in_flight: Vec<Frame>,
  max_frames_in_flight: usize,

  texture_renderpass: Renderpass,
  model_renderpass: Renderpass,
  framebuffer: VkFrameBuffer,

  swapchain: VkSwapchain,

  pool: VkCommandPool,

  draw_command_buffer: CommandBuffer,
  setup_command_buffer: CommandBuffer,

  depth_image: Image,

  present_complete_semaphore: Semaphore,
  rendering_complete_semaphore: Semaphore,

  scissors: Scissors,
  clear_values: ClearValues,
  viewports: Viewport,
}

impl Vulkan {
  pub fn new(
    window: &mut VkWindow,
    event_loop: &EventLoop<()>,
    screen_resolution: vk::Extent2D,
  ) -> Vulkan {
    let instance = VkInstance::new(window, event_loop);
    let device = VkDevice::new(&instance, event_loop, window);

    let mut swapchain = VkSwapchain::new(&instance, &device, screen_resolution);

    let pool = VkCommandPool::new(&device);
    let draw_command_buffer = CommandBuffer::new_one_time_submit(&device, &pool);
    let setup_command_buffer = CommandBuffer::new_one_time_submit(&device, &pool);

    let extent = swapchain.extent();
    let depth_image = ImageBuilder::new_depth(
      extent.width,
      extent.height,
      1,
      1,
      vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
    )
    .build_device_local(&device);
    let passes = vec![
      PassDescription::new(vk::Format::A8B8G8R8_SRGB_PACK32) //device.surface_format().format)
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
        .final_layout_depth_stencil(),
    ];

    let texture_renderpass = Renderpass::new(&device, passes);

    let passes = vec![
      PassDescription::new(vk::Format::A8B8G8R8_SRGB_PACK32) //device.surface_format().format)
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
        .final_layout_depth_stencil(),
    ];

    let model_renderpass = Renderpass::new(&device, passes);

    let framebuffer =
      VkFrameBuffer::new(&device, &mut swapchain, &depth_image, &texture_renderpass);

    let present_complete_semaphore = Semaphore::new(&device);
    let rendering_complete_semaphore = Semaphore::new(&device);

    let clear_values = ClearValues::new()
      .add_colour(0.2, 0.2, 0.2, 0.0)
      .add_depth(1.0, 0);
    let scissors = Scissors::new().add_scissor(0, 0, extent.width, extent.height);

    let viewports = Viewport::new(
      0.0,
      extent.height as f32,
      extent.width as f32,
      -(extent.height as f32),
      0.0,
      1.0,
    );

    let mut frames_in_flight = Vec::new();
    for _ in 0..FRAMES_IN_FLIGHT {
      frames_in_flight.push(Frame::new(&device));
    }

    Vulkan {
      instance,
      device,

      current_frame: 0,
      frames_in_flight,
      max_frames_in_flight: FRAMES_IN_FLIGHT,

      texture_renderpass,
      model_renderpass,

      swapchain,
      pool,

      draw_command_buffer,
      setup_command_buffer,

      depth_image,
      present_complete_semaphore,
      rendering_complete_semaphore,
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

    self.depth_image = ImageBuilder::new_depth(
      extent.width,
      extent.height,
      1,
      1,
      vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
    )
    .build_device_local(&self.device);

    self.framebuffer = VkFrameBuffer::new(
      &self.device,
      &mut self.swapchain,
      &self.depth_image,
      &self.texture_renderpass,
    );

    self.scissors = Scissors::new().add_scissor(0, 0, extent.width, extent.height);

    self.viewports = Viewport::new(
      0.0,
      extent.height as f32,
      extent.width as f32,
      -(extent.height as f32),
      0.0,
      1.0,
    );
  }

  pub fn copy_buffer_to_device_local_image(&mut self, src_buffer: &Buffer<u8>, dst_image: &Image) {
    Vulkan::record_submit_commandbuffer(
      &self.device,
      &mut self.setup_command_buffer,
      self.device.present_queue(),
      Vec::new(),
      &Semaphore::new(&self.device),
      &Semaphore::new(&self.device),
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
          device.internal().cmd_pipeline_barrier(
            texture_command_buffer.internal(),
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[texture_barrier],
          );
        }

        let mut buffer_copy_regions = Vec::new();

        buffer_copy_regions.push(
          vk::BufferImageCopy::builder()
            .image_subresource(
              vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .build(),
            )
            .image_extent(vk::Extent3D {
              width: dst_image.width(),   //image_dimensions.0,
              height: dst_image.height(), //image_dimensions.1,
              depth: 1,
            })
            .build(),
        );

        unsafe {
          device.internal().cmd_copy_buffer_to_image(
            texture_command_buffer.internal(),
            *src_buffer.internal(),
            dst_image.internal(),
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &buffer_copy_regions[..], //&[buffer_copy_regions.build()],
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
          device.internal().cmd_pipeline_barrier(
            texture_command_buffer.internal(),
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

  pub fn copy_buffer_to_device_local_buffer<T: Copy>(
    &mut self,
    src_buffer: &Buffer<T>,
    dst_buffer: &Buffer<T>,
  ) {
    Vulkan::record_submit_commandbuffer(
      &self.device,
      &mut self.setup_command_buffer,
      //&self.setup_commands_reuse_fence,
      self.device.present_queue(),
      Vec::new(),
      &Semaphore::new(&self.device), //[],
      &Semaphore::new(&self.device), //[],
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
          device.internal().cmd_pipeline_barrier(
            buffer_command_buffer.internal(),
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
          device.internal().cmd_copy_buffer(
            buffer_command_buffer.internal(),
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
          device.internal().cmd_pipeline_barrier(
            buffer_command_buffer.internal(),
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

  pub fn run_compute_simultaneous<T: Copy, L: Copy>(
    &mut self,
    compute_shader: &ComputeShader,
    push_constants: &[i32; 4 * 4],
    descriptor_sets: &Vec<vk::DescriptorSet>,
    light_visibility_buffer: &mut Buffer<T>,
    point_light_buffer: &mut Buffer<L>,
    x: u32,
    y: u32,
    z: u32,
  ) {
    Vulkan::record_submit_commandbuffer(
      &self.device,
      &mut self.setup_command_buffer,
      self.device.compute_queue(),
      Vec::new(),
      &Semaphore::new(&self.device),
      &Semaphore::new(&self.device),
      |device, compute_command_buffer| {
        let mut buffer_barriers_before = Vec::new();

        buffer_barriers_before.push(
          vk::BufferMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::TRANSFER_READ)
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .buffer(*light_visibility_buffer.internal())
            .size(std::mem::size_of::<T>() as u64 * (light_visibility_buffer.data().len() as u64))
            .build(),
        );

        buffer_barriers_before.push(
          vk::BufferMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::TRANSFER_READ)
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
            .buffer(*point_light_buffer.internal())
            .size(std::mem::size_of::<L>() as u64 * (point_light_buffer.data().len() as u64))
            .build(),
        );

        unsafe {
          device.internal().cmd_pipeline_barrier(
            compute_command_buffer.internal(),
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &buffer_barriers_before,
            &[],
          );

          device.internal().cmd_bind_descriptor_sets(
            compute_command_buffer.internal(),
            vk::PipelineBindPoint::COMPUTE,
            compute_shader.pipeline_layout(),
            0,
            descriptor_sets,
            &[],
          );

          let push_constant_data: Vec<u8> = {
            push_constants
              .iter()
              .map(|x| x.to_le_bytes().to_vec())
              .flatten()
              .collect()
          };

          device.internal().cmd_push_constants(
            compute_command_buffer.internal(),
            compute_shader.pipeline_layout(),
            vk::ShaderStageFlags::COMPUTE,
            0,
            &push_constant_data, //&[0 as u8; 128 * 4],
          );

          device.internal().cmd_bind_pipeline(
            compute_command_buffer.internal(),
            vk::PipelineBindPoint::COMPUTE,
            *compute_shader.pipeline().internal(),
          );

          device
            .internal()
            .cmd_dispatch(compute_command_buffer.internal(), x, y, z)
        }

        let mut buffer_barriers_after = Vec::new();

        buffer_barriers_after.push(
          vk::BufferMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .buffer(*light_visibility_buffer.internal())
            .size(std::mem::size_of::<T>() as u64 * (light_visibility_buffer.data().len() as u64))
            .build(),
        );

        buffer_barriers_after.push(
          vk::BufferMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .buffer(*point_light_buffer.internal())
            .size(std::mem::size_of::<L>() as u64 * (point_light_buffer.data().len() as u64))
            .build(),
        );

        unsafe {
          device.internal().cmd_pipeline_barrier(
            compute_command_buffer.internal(),
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &buffer_barriers_after,
            &[],
          );
        }
      },
    );
  }

  pub fn run_compute_with_buffer<T: Copy>(
    &mut self,
    compute_shader: &ComputeShader,
    descriptor_sets: &DescriptorSet,
    data: &mut Vec<T>,
  ) {
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

    let descriptor_set_writer =
      DescriptorWriter::builder().update_buffer(&dst_buffer, &descriptor_sets);
    descriptor_set_writer.build(&self.device);

    Vulkan::record_submit_commandbuffer(
      &self.device,
      &mut self.setup_command_buffer,
      self.device.compute_queue(),
      Vec::new(),
      &Semaphore::new(&self.device),
      &Semaphore::new(&self.device),
      |device, compute_command_buffer| {
        let buffer_copy = vk::BufferCopy::builder()
          .size(std::mem::size_of::<T>() as u64 * (src_buffer.data().len() as u64));

        unsafe {
          device.internal().cmd_copy_buffer(
            compute_command_buffer.internal(),
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
          device.internal().cmd_pipeline_barrier(
            compute_command_buffer.internal(),
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[buffer_barrier.build()],
            &[],
          );
        }

        unsafe {
          device.internal().cmd_bind_pipeline(
            compute_command_buffer.internal(),
            vk::PipelineBindPoint::COMPUTE,
            *compute_shader.pipeline().internal(),
          );

          device.internal().cmd_bind_descriptor_sets(
            compute_command_buffer.internal(),
            vk::PipelineBindPoint::COMPUTE,
            compute_shader.pipeline_layout(),
            0,
            &descriptor_sets.internal()[..],
            &[],
          );
        }

        unsafe {
          device.internal().cmd_dispatch(
            compute_command_buffer.internal(),
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
          device.internal().cmd_pipeline_barrier(
            compute_command_buffer.internal(),
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[buffer_barrier_end.build()],
            &[],
          );
        }

        let buffer_copy =
          vk::BufferCopy::builder().size(std::mem::size_of::<T>() as u64 * (data.len() as u64));

        unsafe {
          device.internal().cmd_copy_buffer(
            compute_command_buffer.internal(),
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
  pub fn record_submit_commandbuffer<F: FnOnce(&VkDevice, &mut CommandBuffer)>(
    device: &VkDevice,
    command_buffer: &mut CommandBuffer,
    _submit_queue: vk::Queue,
    wait_mask: Vec<vk::PipelineStageFlags>,
    wait_semaphores: &Semaphore,
    signal_semaphores: &Semaphore,
    f: F,
  ) {
    command_buffer.reset(device);
    command_buffer.begin(device);

    f(device, command_buffer);

    command_buffer.end(device);

    let is_compute = false;
    command_buffer.submit_queue(
      device,
      vec![wait_semaphores],
      vec![signal_semaphores],
      wait_mask,
      is_compute,
    );
  }

  pub fn draw_text<T: Copy>(
    &mut self,
    texture_descriptor: &DescriptorSet,
    shader: &Shader<T>,
    vertex_buffer: &Buffer<T>,
    data: Vec<f32>,
  ) {
    let draw_command_buffer = self.frames_in_flight[self.current_frame].command_buffer();

    draw_command_buffer.bind_descriptor_sets(
      &self.device,
      shader,
      0,
      vec![texture_descriptor],
      false,
    );

    draw_command_buffer.bind_graphics_pipeline(&self.device, shader);

    draw_command_buffer.set_viewport(&self.device, vec![&self.viewports]);

    draw_command_buffer.set_scissors(&self.device, vec![&self.scissors]);

    draw_command_buffer.bind_vertex(&self.device, 0, vertex_buffer);

    draw_command_buffer.push_constants(&self.device, shader, vk::ShaderStageFlags::VERTEX, data);

    draw_command_buffer.draw_buffer(&self.device, vertex_buffer);
  }

  pub fn draw_texture<T: Copy, L: Copy, S: Copy>(
    &mut self,
    texture_descriptor: &DescriptorSet,
    uniform_descriptor: &DescriptorSet,
    shader: &Shader<T>,
    vertex_buffer: &Buffer<T>,
    index_buffer: &Buffer<L>,
    instanced_buffer: Option<&Buffer<S>>,
    instance_count: usize,
    data: Vec<f32>,
  ) {
    let draw_command_buffer = self.frames_in_flight[self.current_frame].command_buffer();

    draw_command_buffer.bind_descriptor_sets(
      &self.device,
      shader,
      0,
      vec![uniform_descriptor],
      false,
    );

    draw_command_buffer.bind_descriptor_sets(
      &self.device,
      shader,
      1,
      vec![texture_descriptor],
      false,
    );

    draw_command_buffer.bind_graphics_pipeline(&self.device, shader);

    draw_command_buffer.set_viewport(&self.device, vec![&self.viewports]);

    draw_command_buffer.set_scissors(&self.device, vec![&self.scissors]);

    draw_command_buffer.bind_vertex(&self.device, 0, vertex_buffer);

    if let Some(buffer) = instanced_buffer {
      draw_command_buffer.bind_vertex(&self.device, 1, buffer);
    }

    draw_command_buffer.bind_index(&self.device, index_buffer);

    draw_command_buffer.push_constants(&self.device, shader, vk::ShaderStageFlags::VERTEX, data);

    if instanced_buffer.is_some() {
      draw_command_buffer.draw_indexed_instanced(
        &self.device,
        instance_count as u32,
        &index_buffer,
      );
    } else {
      draw_command_buffer.draw_indexed_buffer(&self.device, &index_buffer);
    }
  }

  pub fn draw_mesh<T: Copy>(
    &mut self,
    shader: &Shader<T>,
    mesh_descriptor: &DescriptorSet,
    uniform_descriptor: &DescriptorSet,
    dummy_skin: &DescriptorSet,
    data: Vec<f32>,
    model: &GltfModel,
  ) {
    let draw_command_buffer = self.frames_in_flight[self.current_frame].command_buffer();

    draw_command_buffer.bind_descriptor_sets(
      &self.device,
      shader,
      0,
      vec![uniform_descriptor],
      false,
    );

    draw_command_buffer.bind_graphics_pipeline(&self.device, shader);
    draw_command_buffer.set_viewport(&self.device, vec![&self.viewports]);
    draw_command_buffer.set_scissors(&self.device, vec![&self.scissors]);

    draw_command_buffer.bind_vertex(&self.device, 0, model.vertex_buffer());
    draw_command_buffer.bind_index(&self.device, model.index_buffer());

    let translation = Vec3::new(data[0], data[1], data[2]);
    //let rotation = Vec3::new(data[8], data[9], data[10]);
    let rotation = Quat::from_euler(
      EulerRot::YXZ,
      data[9].to_radians(),
      data[8].to_radians(),
      data[10].to_radians(),
    );
    let scale = Vec3::new(data[4], data[5], data[6]);

    for i in 0..model.nodes().len() {
      self.draw_node(
        shader,
        mesh_descriptor,
        i,
        translation,
        rotation,
        scale,
        &data,
        model.nodes(),
        &model.skins(),
        &model.materials(),
        dummy_skin,
      );
    }
  }

  fn draw_node<T: Copy>(
    &mut self,
    shader: &Shader<T>,
    mesh_descriptor: &DescriptorSet,
    idx: usize,
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
    data: &Vec<f32>,
    nodes: &Vec<Node>,
    skins: &Vec<Skin>,
    materials: &Vec<Material>,
    dummy_skin: &DescriptorSet,
  ) {
    let draw_command_buffer = self.frames_in_flight[self.current_frame].command_buffer();

    if nodes[idx].mesh.primitives.len() > 0 {
      let push_constant_data: [f32; 32] = [0.0; 32];
      let matrix: [f32; 16] =
        Node::calculate_global_matrix(nodes, idx, translation, rotation, scale).to_cols_array();

      let push_constant_data = push_constant_data
        .iter()
        .enumerate()
        .map(|(i, d)| {
          if i < 16 {
            matrix[i]
          } else {
            let data_offset = 16;
            if i < data_offset - 1 + data.len() {
              data[i - data_offset]
            } else {
              *d
            }
          }
        })
        .collect::<Vec<f32>>();

      draw_command_buffer.push_constants(
        &self.device,
        shader,
        vk::ShaderStageFlags::VERTEX,
        push_constant_data,
      );

      draw_command_buffer.bind_descriptor_sets(
        &self.device,
        shader,
        1,
        if skins.len() > 0 && nodes[idx].skin != -1 {
          vec![&skins[nodes[idx].skin as usize].descriptor_set]
        } else {
          vec![dummy_skin]
        },
        false,
      );

      for primitive in &nodes[idx].mesh.primitives {
        if primitive.index_count > 0 {
          let image_descriptor = if materials.len() > primitive.material_index as usize {
            materials[primitive.material_index as usize].descriptor()
          } else {
            mesh_descriptor
          };

          draw_command_buffer.bind_descriptor_sets(
            &self.device,
            shader,
            2,
            vec![image_descriptor],
            false,
          );

          draw_command_buffer.draw_indexed(
            &self.device,
            primitive.index_count,
            1,
            primitive.first_index,
            0,
          );
        }
      }
    }
  }

  pub fn end_renderpass(&mut self) {
    self.frames_in_flight[self.current_frame]
      .command_buffer()
      .end_renderpass(&self.device);
  }

  pub fn start_render(&mut self) -> Option<u32> {
    let present_index_result = unsafe {
      self.swapchain.swapchain_loader().acquire_next_image(
        *self.swapchain.internal(),
        std::u64::MAX,
        self.frames_in_flight[self.current_frame]
          .present_semaphore()
          .internal(),
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

    let command_buffer = self.frames_in_flight[self.current_frame].command_buffer();
    command_buffer.reset(&self.device);
    command_buffer.begin(&self.device);

    Some(present_index)
  }

  pub fn submit_commandbuffer(
    &mut self,
    command_buffer: &mut CommandBuffer,
    wait_semaphores: Vec<&Semaphore>,
    signal_semaphores: Vec<&Semaphore>,
    wait_stages: Vec<vk::PipelineStageFlags>,
    is_compute: bool,
  ) {
    command_buffer.submit_queue(
      &self.device,
      wait_semaphores,
      signal_semaphores,
      wait_stages,
      is_compute,
    );
  }

  pub fn end_render(&mut self, present_index: u32) {
    let (command_buffer, present_semaphore, render_semaphore) =
      self.frames_in_flight[self.current_frame].borrow_all();

    let wait_mask = vec![vk::PipelineStageFlags::BOTTOM_OF_PIPE];
    let _submit_queue = self.device.present_queue();

    let wait_semaphores = vec![present_semaphore];
    let signal_semaphores = vec![render_semaphore];

    command_buffer.end(&self.device);
    command_buffer.submit_queue(
      &self.device,
      wait_semaphores,
      signal_semaphores,
      wait_mask,
      false,
    );

    let present_info = vk::PresentInfoKHR {
      wait_semaphore_count: 1,
      p_wait_semaphores: &self.frames_in_flight[self.current_frame]
        .render_semaphore()
        .internal(),
      swapchain_count: 1,
      p_swapchains: self.swapchain.internal(),
      p_image_indices: &present_index,
      ..Default::default()
    };

    unsafe {
      match self
        .swapchain
        .swapchain_loader()
        .queue_present(self.device.present_queue(), &present_info)
      {
        Ok(_) => {}
        Err(vk_e) => {
          match vk_e {
            vk::Result::ERROR_OUT_OF_DATE_KHR => {
              //VK_ERROR_OUT_OF_DATE_KHR
              self.recreate_swapchain();
              return;
            }
            e => {
              panic!("Error: {}", e);
            }
          }
        }
      }
    };

    self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;
  }

  pub fn begin_renderpass_texture(&mut self, present_index: u32) {
    let command_buffer = self.frames_in_flight[self.current_frame].command_buffer();
    command_buffer.begin_renderpass(
      &self.device,
      &self.clear_values,
      &self.texture_renderpass,
      self.framebuffer.framebuffers()[present_index as usize],
      self.swapchain.extent(),
    );
  }

  pub fn begin_renderpass_model(&mut self, present_index: u32) {
    let command_buffer = self.frames_in_flight[self.current_frame].command_buffer();
    command_buffer.begin_renderpass(
      &self.device,
      &self.clear_values,
      &self.model_renderpass,
      self.framebuffer.framebuffers()[present_index as usize],
      self.swapchain.extent(),
    );
  }
}
