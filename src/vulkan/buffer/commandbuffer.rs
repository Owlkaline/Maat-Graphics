use vk;

use crate::vulkan::ImageAttachment;
use crate::vulkan::Device;
use crate::vulkan::RenderPass;
use crate::vulkan::Pipeline;
use crate::vulkan::Swapchain;
use crate::vulkan::sync::Semaphore;
use crate::vulkan::sync::Fence;
use crate::vulkan::pool::CommandPool;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::check_errors;
use crate::vulkan::buffer::UniformData;

use crate::vulkan::vkenums::{PipelineStage, ImageAspect, ImageLayout, ShaderStage, CommandBufferLevel,
                             PipelineBindPoint, SubpassContents, IndexType, Access, SampleCount};

use std::mem;
use std::ptr;
use std::sync::Arc;

use cgmath::Vector4;

pub struct CommandBuffer {
  command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
  pub fn primary(device: Arc<Device>, command_pool: &CommandPool) -> CommandBuffer {
    let command_pool = command_pool.local_command_pool();
    
    let command_buffer_allocate_info = {
      vk::CommandBufferAllocateInfo {
        sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: ptr::null(),
        commandPool: *command_pool,
        level: CommandBufferLevel::Primary.to_bits(),
        commandBufferCount: 1,
      }
    };
    
    let mut command_buffer: vk::CommandBuffer = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      check_errors(vk.AllocateCommandBuffers(*device, &command_buffer_allocate_info, &mut command_buffer));
    }
    
    CommandBuffer {
      command_buffer,
    }
  }
  
  pub fn from_buffer(command_buffer: vk::CommandBuffer) -> CommandBuffer {
    CommandBuffer {
      command_buffer,
    }
  }
  
  pub fn secondary(device: Arc<Device>, command_pool: &CommandPool) -> CommandBuffer {
    let command_pool = command_pool.local_command_pool();
    
    let command_buffer_allocate_info = {
      vk::CommandBufferAllocateInfo {
        sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: ptr::null(),
        commandPool: *command_pool,
        level: CommandBufferLevel::Secondary.to_bits(),
        commandBufferCount: 1,
      }
    };
    
    let mut command_buffer: vk::CommandBuffer = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      check_errors(vk.AllocateCommandBuffers(*device, &command_buffer_allocate_info, &mut command_buffer));
    }
    
    CommandBuffer {
      command_buffer,
    }
  }
  
  pub fn begin_single_time_command(device: Arc<Device>, command_pool: &CommandPool) -> CommandBuffer {
    let command_buffer = CommandBuffer::primary(Arc::clone(&device), command_pool);
    command_buffer.begin_command_buffer(Arc::clone(&device), CommandBufferLevel::Primary.to_bits());
    command_buffer
  }
  
  pub fn end_single_time_command(&self, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) {
    let submit_info = {
      vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 0,
        pWaitSemaphores: ptr::null(),
        pWaitDstStageMask: ptr::null(),
        commandBufferCount: 1,
        pCommandBuffers: &self.command_buffer,
        signalSemaphoreCount: 0,
        pSignalSemaphores: ptr::null(),
      }
    };
    
    self.end_command_buffer(Arc::clone(&device));
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      let command_pool = command_pool.local_command_pool();
      vk.QueueSubmit(*graphics_queue, 1, &submit_info, 0);
      vk.QueueWaitIdle(*graphics_queue);
      vk.FreeCommandBuffers(*device, *command_pool, 1, &self.command_buffer);
    }
  }
  
  pub fn begin_render_pass(&self, device: Arc<Device>, render_pass: &RenderPass, framebuffer: &vk::Framebuffer, clear_values: &Vec<vk::ClearValue>, width: u32, height: u32) {
    let vk = device.pointers();
    let render_pass_begin_info = {
      vk::RenderPassBeginInfo {
        sType: vk::STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
        pNext: ptr::null(),
        renderPass: *render_pass.internal_object(),
        framebuffer: *framebuffer,
        renderArea: vk::Rect2D { offset: vk::Offset2D {x: 0, y: 0 }, extent: vk::Extent2D { width: width, height: height, } },
        clearValueCount: clear_values.len() as u32,
        pClearValues: clear_values.as_ptr(),
      }
    };
    
    unsafe {
      vk.CmdBeginRenderPass(self.command_buffer, &render_pass_begin_info, SubpassContents::Inline.to_bits());
    }
  }
  
  pub fn end_render_pass(&self, device: Arc<Device>) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdEndRenderPass(self.command_buffer);
    }
  }
  
  pub fn begin_command_buffer(&self, device: Arc<Device>, flags: u32) {
    let vk = device.pointers();
    
    let command_buffer_begin_info = {
      vk::CommandBufferBeginInfo {
        sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
        pNext: ptr::null(),
        flags: flags,
        pInheritanceInfo: ptr::null(),
      }
    };
    
    unsafe {
      check_errors(vk.BeginCommandBuffer(self.command_buffer, &command_buffer_begin_info));
    }
  }
  
  pub fn end_command_buffer(&self, device: Arc<Device>) {
    let vk = device.pointers();
    
    unsafe {
      check_errors(vk.EndCommandBuffer(self.command_buffer));
    }
  }
  
  pub fn bind_graphics_descriptor_set(&self, device: Arc<Device>, pipeline: &Pipeline, descriptor_set: Vec<vk::DescriptorSet>, dynamic_offsets: Vec<u32>) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindDescriptorSets(self.command_buffer, PipelineBindPoint::Graphics.to_bits(), *pipeline.layout(), 0,
                               descriptor_set.len() as u32, descriptor_set.as_ptr(), 
                               dynamic_offsets.len() as u32, 
                               if dynamic_offsets.len() == 0 { ptr::null() } else { dynamic_offsets.as_ptr() });
    }
  }
  
  pub fn bind_graphics_pipeline(&self, device: Arc<Device>, pipeline: &Pipeline) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindPipeline(self.command_buffer, vk::PIPELINE_BIND_POINT_GRAPHICS, *pipeline.pipeline(0));
    }
  }
  
  pub fn bind_compute_descriptor_set(&self, device: Arc<Device>, pipeline: &Pipeline, descriptor_set: Vec<vk::DescriptorSet>) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindDescriptorSets(self.command_buffer, PipelineBindPoint::Compute.to_bits(), *pipeline.layout(), 0, descriptor_set.len() as u32, descriptor_set.as_ptr(), 0, ptr::null());
    }
  }
  
  pub fn bind_compute_pipeline(&self, device: Arc<Device>, pipeline: &Pipeline) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindPipeline(self.command_buffer, PipelineBindPoint::Compute.to_bits(), *pipeline.pipeline(0));
    }
  }
  
  pub fn bind_vertex_buffer(&self, device: Arc<Device>, set_id: u32, offset: u64, vertex_buffer: &vk::Buffer) {
    let vk = device.pointers();
    
    unsafe {
       let mut offsets = Vec::with_capacity(1);
       offsets.push(offset);
      vk.CmdBindVertexBuffers(self.command_buffer, set_id, 1, vertex_buffer, offsets.as_ptr());
    }
  }
  
  pub fn bind_index_buffer(&self, device: Arc<Device>, offset: u64, index_buffer: &vk::Buffer) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdBindIndexBuffer(self.command_buffer, *index_buffer, offset, IndexType::Uint32.to_bits());
    }
  }
  
  pub fn set_viewport(&self, device: Arc<Device>, x: f32, y: f32, width: f32, height: f32) {
    let vk = device.pointers();
    
    let viewport = vk::Viewport {
      x,
      y,
      width,
      height,
      minDepth: 0.0,
      maxDepth: 1.0,
    };
    
    unsafe {
      vk.CmdSetViewport(self.command_buffer, 0, 1, &viewport);
    }
  }
  
  pub fn set_scissor(&self, device: Arc<Device>, x: i32, y: i32, width: u32, height: u32) {
    let vk = device.pointers();
    
    let scissor = vk::Rect2D {
      offset: vk::Offset2D { x, y },
      extent: vk::Extent2D { width, height },
    };
    
    unsafe {
      vk.CmdSetScissor(self.command_buffer, 0, 1, &scissor);
    }
  }
  
  pub fn set_scissors(&self, device: Arc<Device>, scissors: Vec<Vector4<u32>>) {
    let vk = device.pointers();
    
    
    let mut scissor_rects = Vec::with_capacity(scissors.len());
    
    for scissor in scissors {
      scissor_rects.push(
        vk::Rect2D {
          offset: vk::Offset2D { x: scissor.x as i32, y: scissor.y as i32 },
          extent: vk::Extent2D { width: scissor.z, height: scissor.w },
        }
      );
    }
    
    unsafe {
      vk.CmdSetScissor(self.command_buffer, 0, scissor_rects.len() as u32, scissor_rects.as_ptr());
    }
  }
  
  pub fn push_constants(&self, device: Arc<Device>, pipeline: &Pipeline, shader_stage: ShaderStage, push_constant_data: UniformData) {
    let mut push_constant_data = push_constant_data;
    let size = push_constant_data.size(Arc::clone(&device));
    let data = push_constant_data.build(Arc::clone(&device));
    
    let vk = device.pointers();
    unsafe {
      vk.CmdPushConstants(self.command_buffer, *pipeline.layout(), shader_stage.to_bits(), 0, size as u32, data.as_ptr() as *const _);
    }
  }
  
  pub fn draw(&self, device: Arc<Device>, vertex_count: u32, instance_count: u32) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdDraw(self.command_buffer, vertex_count, instance_count, 0, 0);
    }
  }
  
  pub fn draw_indexed(&self, device: Arc<Device>, index_count: u32, index_offset: u32, vertex_offset: i32, instance_count: u32) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdDrawIndexed(self.command_buffer, index_count, instance_count, index_offset, vertex_offset, 0);
    }
  }
  
  pub fn dispatch(&self, device: Arc<Device>, x: u32, y: u32, z: u32) {
    let vk = device.pointers();
    
    unsafe {
      vk.CmdDispatch(self.command_buffer, x, y, z);
    }
  }
  
  pub fn copy_image(&self, device: Arc<Device>, width: u32, height: u32, src_image: &ImageAttachment, src_layout: ImageLayout, src_image_aspect: ImageAspect, dst_image: &vk::Image, dst_layout: ImageLayout, dst_image_aspect: ImageAspect) {
    
    let src_image_subresource_layers = vk::ImageSubresourceLayers {
      aspectMask: src_image_aspect.to_bits(),
      mipLevel: 0,
      baseArrayLayer: 0,
      layerCount: 1,
    };
    
    let dst_image_subresource_layers = vk::ImageSubresourceLayers {
      aspectMask: dst_image_aspect.to_bits(),
      mipLevel: 0,
      baseArrayLayer: 0,
      layerCount: 1,
    };
    
    let region = vk::ImageCopy {
      srcSubresource: src_image_subresource_layers,
      srcOffset: vk::Offset3D { x: 0, y: 0, z: 0 },
      dstSubresource: dst_image_subresource_layers,
      dstOffset: vk::Offset3D { x: 0, y: 0, z: 0 },
      extent: vk::Extent3D { width, height, depth: 1 },
    };
    
    unsafe {
      let vk = device.pointers();
      vk.CmdCopyImage(self.command_buffer, src_image.get_image(), src_layout.to_bits(), *dst_image, dst_layout.to_bits(), SampleCount::OneBit.to_bits(), &region);
    }
  }
  
  pub fn copy_buffer<T: Clone, U: Clone>(&self, device: Arc<Device>, src_buffer: &Buffer<T>, dst_buffer: &Buffer<U>, current_buffer: usize) 
{
    let buffer_copy = {
        vk::BufferCopy {
          srcOffset: 0,
          dstOffset: 0,
          size: src_buffer.max_size(),
        }
      };
    
    unsafe {
      let vk = device.pointers();
      vk.CmdCopyBuffer(self.command_buffer, *src_buffer.internal_object(current_buffer), *dst_buffer.internal_object(current_buffer), 1, &buffer_copy);
    }
  }
  
  pub fn copy_buffer_to_image<T: Clone>(&self, device: Arc<Device>, src_buffer: &Buffer<T>, dst_image: vk::Image, image_aspect: ImageAspect, width: u32, height: u32, current_buffer: usize) {
    
    let image_subresource_layers = vk::ImageSubresourceLayers {
      aspectMask: image_aspect.to_bits(),
      mipLevel: 0,
      baseArrayLayer: 0,
      layerCount: 1,
    };
    
    let region = vk::BufferImageCopy {
      bufferOffset: 0,
      bufferRowLength: 0,
      bufferImageHeight: 0,
      imageSubresource: image_subresource_layers,
      imageOffset: vk::Offset3D { x: 0, y: 0, z: 0 },
      imageExtent: vk::Extent3D { width, height, depth: 1 },
    };
    
    unsafe {
      let vk = device.pointers();
      vk.CmdCopyBufferToImage(self.command_buffer, *src_buffer.internal_object(current_buffer), dst_image, ImageLayout::TransferDstOptimal.to_bits(), 1, &region);
    }
  }
  
  pub fn pipeline_barrier(&self, device: Arc<Device>, src_stage: PipelineStage, dst_stage: PipelineStage, barrier: vk::ImageMemoryBarrier) {
    unsafe {
      let vk = device.pointers();
      vk.CmdPipelineBarrier(self.command_buffer, src_stage.to_bits(), dst_stage.to_bits(), 0, 0, ptr::null(), 0, ptr::null(), 1, &barrier);
    }
  }
  
  pub fn image_barrier(&self, device: Arc<Device>, src_mask: &Access, dst_mask: &Access, old_layout: &ImageLayout, new_layout: &ImageLayout, aspect: &ImageAspect, src_stage: PipelineStage, dst_stage: PipelineStage, src_queue_family: u32, dst_queue_family: u32, image: &ImageAttachment) {
    let subresource_range = vk::ImageSubresourceRange {
      aspectMask: aspect.to_bits(),
      baseMipLevel: 0,
      levelCount: 1,
      baseArrayLayer: 0,
      layerCount: 1,
    };
    
    let barrier = vk::ImageMemoryBarrier {
      sType: vk::STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
      pNext: ptr::null(),
      srcAccessMask: src_mask.to_bits(),
      dstAccessMask: dst_mask.to_bits(),
      oldLayout: old_layout.to_bits(),
      newLayout: new_layout.to_bits(),
      srcQueueFamilyIndex: src_queue_family,
      dstQueueFamilyIndex: dst_queue_family,
      image: image.get_image(),
      subresourceRange: subresource_range,
    };
    
    unsafe {
      let vk = device.pointers();
      vk.CmdPipelineBarrier(self.command_buffer, src_stage.to_bits(), dst_stage.to_bits(), 0, 0, ptr::null(), 0, ptr::null(), 1, &barrier);
    }
  }
  
  pub fn internal_object(&self) -> &vk::CommandBuffer {
    &self.command_buffer
  }
  
  pub fn submit(&self, device: Arc<Device>, swapchain: &Swapchain, current_image: u32, image_available: &Semaphore, render_finished: &Semaphore, fence: &Fence, graphics_queue: &vk::Queue) -> vk::Result {
    let pipeline_stage_flags = PipelineStage::ColorAttachmentOutput.to_bits();
    let submit_info: vk::SubmitInfo = {
      vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 1,
        pWaitSemaphores: image_available.internal_object(),
        pWaitDstStageMask: &pipeline_stage_flags,
        commandBufferCount: 1,
        pCommandBuffers: &self.command_buffer,
        signalSemaphoreCount: 1,
        pSignalSemaphores: render_finished.internal_object(),
      }
    };
    
    unsafe {
      let vk = device.pointers();
      check_errors(vk.QueueSubmit(*graphics_queue, 1, &submit_info, *fence.internal_object()));
    }
    
    let present_info_khr = {
      vk::PresentInfoKHR {
        sType: vk::STRUCTURE_TYPE_PRESENT_INFO_KHR,
        pNext: ptr::null(),
        waitSemaphoreCount: 1,
        pWaitSemaphores: render_finished.internal_object(),
        swapchainCount: 1,
        pSwapchains: swapchain.get_swapchain(),
        pImageIndices: &current_image,
        pResults: ptr::null_mut(),
      }
    };
    
    unsafe {
      let vk = device.pointers();
      vk.QueuePresentKHR(*graphics_queue, &present_info_khr)
    }
  }
  
  pub fn finish(&self, device: Arc<Device>, graphics_queue: &vk::Queue) {
    unsafe {
      let vk = device.pointers();
      vk.QueueWaitIdle(*graphics_queue);
    }
  }
  
  pub fn free(&self, device: Arc<Device>, command_pool: &CommandPool) {
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      vk.FreeCommandBuffers(*device, *command_pool.local_command_pool(), 1, &self.command_buffer);
    }
  }
  
  pub fn destroy(&self) {
    
  }
}
