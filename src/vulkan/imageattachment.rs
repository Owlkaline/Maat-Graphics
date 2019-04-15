use vk;

use crate::vulkan::{Instance, Device};
use crate::vulkan::buffer::{Buffer, BufferUsage, CommandBuffer, CommandBufferBuilder};
use crate::vulkan::pool::{CommandPool};
use crate::vulkan::vkenums::{ImageType, ImageViewType, ImageLayout, ImageTiling, ImageAspect, SampleCount, 
                             ImageUsage, SharingMode, PipelineStage, Access, MemoryProperty, ComponentSwizzle};
use crate::vulkan::check_errors;

use image;

use std::mem;
use std::ptr;
use std::sync::Arc;

#[derive(Clone)]
pub struct ImageAttachment {
  image: vk::Image,
  image_view: vk::ImageView,
  memory: vk::DeviceMemory,
  format: vk::Format,
  width: u32,
  height: u32,
}

impl ImageAttachment {
  
  pub fn create_image_colour_attachment(instance: Arc<Instance>, device: Arc<Device>, image_type: &ImageType, tiling: &ImageTiling, usage: &ImageUsage, initial_layout: &ImageLayout, samples: &SampleCount, image_view_type: &ImageViewType, format: &vk::Format, width: u32, height: u32) -> ImageAttachment {
    let memory_property = MemoryProperty::DeviceLocal;
    let image_aspect = ImageAspect::Colour;
    
    let (image, memory) = ImageAttachment::create_image(Arc::clone(&instance), Arc::clone(&device), &memory_property, image_type, tiling, usage, initial_layout, samples, format, width, height);
    let image_view = ImageAttachment::create_image_view(Arc::clone(&device), &image, format, &image_aspect, image_view_type);
    
    ImageAttachment {
      image,
      image_view,
      memory,
      format: *format,
      width,
      height,
    }
  }
  
  pub fn create_image_depth_attachment(instance: Arc<Instance>, device: Arc<Device>, image_type: &ImageType, tiling: &ImageTiling, usage: &ImageUsage, initial_layout: &ImageLayout, samples: &SampleCount, image_view_type: &ImageViewType, format: &vk::Format, width: u32, height: u32) -> ImageAttachment {
    let memory_property = MemoryProperty::DeviceLocal;
    let image_aspect = ImageAspect::Depth;
    
    let (image, memory) = ImageAttachment::create_image(Arc::clone(&instance), Arc::clone(&device), &memory_property, image_type, tiling, usage, initial_layout, samples, format, width, height);
    let image_view = ImageAttachment::create_image_view(Arc::clone(&device), &image, format, &image_aspect, image_view_type);
    
    ImageAttachment {
      image,
      image_view,
      memory,
      format: *format,
      width,
      height,
    }
  }
  
  pub fn create_image_msaa_attachment(instance: Arc<Instance>, device: Arc<Device>, image_type: &ImageType, tiling: &ImageTiling, usage: &ImageUsage, initial_layout: &ImageLayout, final_layout: &ImageLayout, image_aspect: &ImageAspect, samples: &SampleCount, image_view_type: &ImageViewType, format: &vk::Format, command_pool: &CommandPool, graphics_queue: &vk::Queue, width: u32, height: u32) -> ImageAttachment {
    
    let memory_property = MemoryProperty::DeviceLocal;
    let (image, memory) = ImageAttachment::create_image(Arc::clone(&instance), Arc::clone(&device), &memory_property, image_type, tiling, usage, initial_layout, samples, format, width, height);
    
    
    ImageAttachment::transition_layout(Arc::clone(&device), &image, ImageLayout::Undefined, final_layout.clone(), &command_pool, graphics_queue);
    
    let image_view = ImageAttachment::create_image_view(Arc::clone(&device), &image, format, &image_aspect, image_view_type);
    
    ImageAttachment {
      image,
      image_view,
      memory,
      format: *format,
      width,
      height,
    }
  }
  
  pub fn create_texture_from_location(instance: Arc<Instance>, device: Arc<Device>, image: String, image_type: &ImageType, tiling: &ImageTiling, samples: &SampleCount, image_view_type: &ImageViewType, format: vk::Format, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> ImageAttachment {
    let image = image::open(&image.clone()).expect(&("No file or Directory at: ".to_string() + &image)).to_rgba();
    
    ImageAttachment::create_texture(Arc::clone(&instance), Arc::clone(&device), &image, image_type, tiling, samples, image_view_type, format, command_pool, graphics_queue)
  }
  
  pub fn create_texture(instance: Arc<Instance>, device: Arc<Device>, image: &image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>, image_type: &ImageType, tiling: &ImageTiling, samples: &SampleCount, image_view_type: &ImageViewType, format: vk::Format, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> ImageAttachment {
    println!("before texture");
    let (width, height) = image.dimensions();
    let image_data = image.clone().into_raw().clone();
    
    let texture_image_view: vk::ImageView;
    
    let staging_usage = BufferUsage::transfer_src_buffer();
    let image_usage = ImageUsage::transfer_dst_sampled();
    
    let staging_buffer = Buffer::cpu_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), staging_usage, 1, image_data);
    
    let memory_property = MemoryProperty::DeviceLocal;
    let (texture_image, texture_memory) = ImageAttachment::create_image(Arc::clone(&instance), Arc::clone(&device), &memory_property, image_type, tiling, &image_usage, &ImageLayout::Undefined, samples, &format, width, height);
    
    ImageAttachment::transition_layout(Arc::clone(&device), &texture_image, ImageLayout::Undefined, ImageLayout::TransferDstOptimal, &command_pool, graphics_queue);
    
    let cmd = CommandBuffer::begin_single_time_command(Arc::clone(&device), &command_pool);
    cmd.copy_buffer_to_image(Arc::clone(&device), &staging_buffer, texture_image, ImageAspect::Colour, width, height, 0);
    cmd.end_single_time_command(Arc::clone(&device), &command_pool, graphics_queue);
    
    ImageAttachment::transition_layout(Arc::clone(&device), &texture_image, ImageLayout::TransferDstOptimal, ImageLayout::ShaderReadOnlyOptimal, &command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    texture_image_view = ImageAttachment::create_image_view(Arc::clone(&device), &texture_image, &format, 
                                                            &ImageAspect::Colour, image_view_type);
    
    ImageAttachment {
      image: texture_image,
      image_view: texture_image_view,
      memory: texture_memory,
      format: format,
      width,
      height,
    }
  }
  
  pub fn create_texture_from_pixels(instance: Arc<Instance>, device: Arc<Device>, image_data: std::vec::Vec<u8>, width: u32, height: u32, image_type: &ImageType, tiling: &ImageTiling, samples: &SampleCount, image_view_type: &ImageViewType, format: vk::Format, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> ImageAttachment {
     let texture_image_view: vk::ImageView;
    
    let staging_usage = BufferUsage::transfer_src_buffer();
    let image_usage = ImageUsage::transfer_dst_sampled();
    
    let staging_buffer = Buffer::cpu_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), staging_usage, 1, image_data);
    
    let memory_property = MemoryProperty::DeviceLocal;
    let (texture_image, texture_memory) = ImageAttachment::create_image(Arc::clone(&instance), Arc::clone(&device), &memory_property, image_type, tiling, &image_usage, &ImageLayout::Undefined, samples, &format, width, height);
    
    ImageAttachment::transition_layout(Arc::clone(&device), &texture_image, ImageLayout::Undefined, ImageLayout::TransferDstOptimal, &command_pool, graphics_queue);
    
    let cmd = CommandBuffer::begin_single_time_command(Arc::clone(&device), &command_pool);
    cmd.copy_buffer_to_image(Arc::clone(&device), &staging_buffer, texture_image, ImageAspect::Colour, width, height, 0);
    cmd.end_single_time_command(Arc::clone(&device), &command_pool, graphics_queue);
    
    ImageAttachment::transition_layout(Arc::clone(&device), &texture_image, ImageLayout::TransferDstOptimal, ImageLayout::ShaderReadOnlyOptimal, &command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    texture_image_view = ImageAttachment::create_image_view(Arc::clone(&device), &texture_image, &format, 
                                                            &ImageAspect::Colour, image_view_type);
    
    ImageAttachment {
      image: texture_image,
      image_view: texture_image_view,
      memory: texture_memory,
      format: format,
      width,
      height,
    }
  }
  
  pub fn create_dummy_texture(instance: Arc<Instance>, device: Arc<Device>, image_type: &ImageType, tiling: &ImageTiling, samples: &SampleCount, image_view_type: &ImageViewType, format: vk::Format, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> ImageAttachment {
    let width = 2;
    let height = 2;
    
    let image_data = vec!(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    
    let texture_image_view: vk::ImageView;
    
    let staging_usage = BufferUsage::transfer_src_buffer();
    let image_usage = ImageUsage::transfer_dst_sampled();
    
    let staging_buffer = Buffer::cpu_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), staging_usage, 1, image_data);
    
    let memory_property = MemoryProperty::DeviceLocal;
    let (texture_image, texture_memory) = ImageAttachment::create_image(Arc::clone(&instance), Arc::clone(&device), &memory_property, image_type, tiling, &image_usage, &ImageLayout::Undefined, samples, &format, width, height);
    
    ImageAttachment::transition_layout(Arc::clone(&device), &texture_image, ImageLayout::Undefined, ImageLayout::TransferDstOptimal, &command_pool, graphics_queue);
    
    let cmd = CommandBuffer::begin_single_time_command(Arc::clone(&device), &command_pool);
    cmd.copy_buffer_to_image(Arc::clone(&device), &staging_buffer, texture_image, ImageAspect::Colour, width, height, 0);
    cmd.end_single_time_command(Arc::clone(&device), &command_pool, graphics_queue);
    
    ImageAttachment::transition_layout(Arc::clone(&device), &texture_image, ImageLayout::TransferDstOptimal, ImageLayout::ShaderReadOnlyOptimal, &command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    texture_image_view = ImageAttachment::create_image_view(Arc::clone(&device), &texture_image, &format, 
                                                            &ImageAspect::Colour, image_view_type);
    
    ImageAttachment {
      image: texture_image,
      image_view: texture_image_view,
      memory: texture_memory,
      format: format,
      width,
      height,
    }
  }
  
  pub fn create_texture_from_command_buffer(instance: Arc<Instance>, device: Arc<Device>, width: u32, height: u32, image: ImageAttachment, tiling: &ImageTiling, image_view_type: &ImageViewType, format: vk::Format, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> ImageAttachment {
    let image_usage = ImageUsage::transfer_dst_sampled();
    let memory_property = MemoryProperty::DeviceLocal;
    
    let (texture_image, texture_memory) = ImageAttachment::create_image(Arc::clone(&instance), Arc::clone(&device), &memory_property, &ImageType::Type2D, tiling, &image_usage, &ImageLayout::Undefined, &SampleCount::OneBit, &format, width, height);
    let texture_image_view: vk::ImageView;
    
    ImageAttachment::transition_layout(Arc::clone(&device), &image.get_image(), ImageLayout::Undefined, ImageLayout::TransferSrcOptimal, &command_pool, graphics_queue);
    
    ImageAttachment::transition_layout(Arc::clone(&device), &texture_image, ImageLayout::Undefined, ImageLayout::TransferDstOptimal, &command_pool, graphics_queue);
    
    let cmd = CommandBuffer::begin_single_time_command(Arc::clone(&device), &command_pool);
    cmd.copy_image(Arc::clone(&device),  width, height, 
                   &image, ImageLayout::TransferSrcOptimal, ImageAspect::Colour, 
                   &texture_image, ImageLayout::TransferDstOptimal, ImageAspect::Colour);
    cmd.end_single_time_command(Arc::clone(&device), &command_pool, graphics_queue);
     
     ImageAttachment::transition_layout(Arc::clone(&device), &texture_image, ImageLayout::TransferDstOptimal, ImageLayout::ShaderReadOnlyOptimal, &command_pool, graphics_queue);
     
    let texture_image_view = ImageAttachment::create_image_view(Arc::clone(&device), &texture_image, &format, 
                                                                &ImageAspect::Colour, image_view_type);
    
    ImageAttachment {
      image: texture_image,
      image_view: texture_image_view,
      memory: texture_memory,
      format: format,
      width,
      height,
    }
  }
  
  pub fn get_size(&self) -> (u32, u32) {
    (self.width, self.height)
  }
  
  pub fn get_image(&self) -> vk::Image {
    self.image
  }
  
  pub fn get_image_view(&self) -> vk::ImageView {
    self.image_view
  }
  
  pub fn get_image_memory(&self) -> vk::DeviceMemory {
    self.memory
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      
      vk.DestroyImageView(*device, self.image_view, ptr::null());
      vk.DestroyImage(*device, self.image, ptr::null());
      vk.FreeMemory(*device, self.memory, ptr::null());
    }
  }
  
  fn transition_layout(device: Arc<Device>, image: &vk::Image, old_layout: ImageLayout, new_layout: ImageLayout, command_pool: &CommandPool, graphics_queue: &vk::Queue) {
    
    let subresource_range = vk::ImageSubresourceRange {
      aspectMask: ImageAspect::Colour.to_bits(),
      baseMipLevel: 0,
      levelCount: 1,
      baseArrayLayer: 0,
      layerCount: 1,
    };
    
    let src_stage: PipelineStage;
    let dst_stage: PipelineStage;
    let mut src_access: Option<Access> = None;
    let dst_access: Access;
    
    if old_layout == ImageLayout::Undefined && new_layout == ImageLayout::TransferDstOptimal {
      dst_access = Access::TransferWrite;
      
      src_stage = PipelineStage::TopOfPipe;
      dst_stage = PipelineStage::Transfer;
    } else if old_layout == ImageLayout::TransferDstOptimal && new_layout == ImageLayout::ShaderReadOnlyOptimal {
      src_access = Some(Access::TransferWrite);
      dst_access = Access::ShaderRead;
      
      src_stage = PipelineStage::Transfer;
      dst_stage = PipelineStage::FragmentShader;
    } else if old_layout == ImageLayout::Undefined && new_layout == ImageLayout::ColourAttachmentOptimal {
      dst_access = Access::ColourAttachmentReadAndWrite;
      
      src_stage = PipelineStage::TopOfPipe;
      dst_stage = PipelineStage::ColorAttachmentOutput;
    } else if old_layout == ImageLayout::Undefined && new_layout == ImageLayout::TransferSrcOptimal {
      //src_access = Some(Access::ColourAttachmentRead);
      dst_access = Access::TransferRead;
      
      src_stage = PipelineStage::FragmentShader;
      dst_stage = PipelineStage::Transfer;
    } else {
      panic!("Error image transition not supported!");
    }
    
    let barrier = vk::ImageMemoryBarrier {
      sType: vk::STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
      pNext: ptr::null(),
      srcAccessMask: if src_access.is_some() { src_access.unwrap().to_bits() } else { 0 },
      dstAccessMask: dst_access.to_bits(),
      oldLayout: old_layout.to_bits(),
      newLayout: new_layout.to_bits(),
      srcQueueFamilyIndex: vk::QUEUE_FAMILY_IGNORED,
      dstQueueFamilyIndex: vk::QUEUE_FAMILY_IGNORED,
      image: *image,
      subresourceRange: subresource_range,
    };
    
    let command_buffer = CommandBuffer::begin_single_time_command(Arc::clone(&device), command_pool);
    command_buffer.pipeline_barrier(Arc::clone(&device), src_stage, dst_stage, barrier);
    command_buffer.end_single_time_command(Arc::clone(&device), command_pool, graphics_queue);
  }
  
  fn create_image(instance: Arc<Instance>, device: Arc<Device>, memory_property: &MemoryProperty, image_type: &ImageType, tiling: &ImageTiling, usage: &ImageUsage, initial_layout: &ImageLayout,  samples: &SampleCount, format: &vk::Format, width: u32, height: u32) -> (vk::Image, vk::DeviceMemory) {
    let mut image: vk::Image = unsafe { mem::uninitialized() };
    let mut memory: vk::DeviceMemory = unsafe { mem::uninitialized() };
    
    let vk = device.pointers();
    let vk_instance = instance.pointers();
    let phys_device = device.physical_device();
    let device = device.internal_object();
    
    let image_create_info = {
      vk::ImageCreateInfo {
        sType: vk::STRUCTURE_TYPE_IMAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        imageType: image_type.to_bits(),
        format: *format,
        extent: vk::Extent3D { width: width, height: height, depth: 1 },
        mipLevels: 1,
        arrayLayers: 1,
        samples: samples.to_bits(),
        tiling: tiling.to_bits(),
        usage: usage.to_bits(),
        sharingMode: SharingMode::Exclusive.to_bits(),
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: ptr::null(),
        initialLayout: initial_layout.to_bits(),
      }
    };
    
   let mut memory_requirements: vk::MemoryRequirements = unsafe { mem::uninitialized() };
    
    unsafe {
      check_errors(vk.CreateImage(*device, &image_create_info, ptr::null(), &mut image));
      vk.GetImageMemoryRequirements(*device, image, &mut memory_requirements);
    }
    
    let memory_type_bits_index = {
      
      let mut memory_properties: vk::PhysicalDeviceMemoryProperties = unsafe { mem::uninitialized() };
      
      unsafe {
        vk_instance.GetPhysicalDeviceMemoryProperties(*phys_device, &mut memory_properties);
      }
      
      let mut index: i32 = -1;
      let properties = memory_property.to_bits();
      for i in 0..memory_properties.memoryTypeCount as usize {
        if memory_requirements.memoryTypeBits & (1 << i) != 0 && memory_properties.memoryTypes[i].propertyFlags & properties == properties {
          index = i as i32;
        }
      }
      
      if index == -1 {
        panic!("Failed to find suitable memory type");
      }
      
      index
    };
    
    let memory_allocate_info = {
      vk::MemoryAllocateInfo {
        sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
        pNext: ptr::null(),
        allocationSize: memory_requirements.size,
        memoryTypeIndex: memory_type_bits_index as u32,
      }
    };
    
    unsafe {
      check_errors(vk.AllocateMemory(*device, &memory_allocate_info, ptr::null(), &mut memory));
      check_errors(vk.BindImageMemory(*device, image, memory, 0));
    }
    
    (image, memory)
  }
  
  fn create_image_view(device: Arc<Device>, image: &vk::Image, format: &vk::Format, image_aspect: &ImageAspect, image_view_type: &ImageViewType) -> vk::ImageView {
    let mut image_view: vk::ImageView = unsafe { mem::uninitialized() };
    
    let vk = device.pointers();
    let device = device.internal_object();
    
    let component = vk::ComponentMapping {
      r: ComponentSwizzle::Identity.to_bits(),
      g: ComponentSwizzle::Identity.to_bits(),
      b: ComponentSwizzle::Identity.to_bits(),
      a: ComponentSwizzle::Identity.to_bits(),
    };
    
    let subresource = vk::ImageSubresourceRange {
      aspectMask: image_aspect.to_bits(),
      baseMipLevel: 0,
      levelCount: 1,
      baseArrayLayer: 0,
      layerCount: 1,
    };
    
    let image_view_create_info = vk::ImageViewCreateInfo {
      sType: vk::STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      image: *image,
      viewType: image_view_type.to_bits(),
      format: *format,
      components: component,
      subresourceRange: subresource,
    };
    
    unsafe {
      vk.CreateImageView(*device, &image_view_create_info, ptr::null(), &mut image_view);
    }
    
    image_view
  }
}
