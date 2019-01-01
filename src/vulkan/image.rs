use vk;

use crate::vulkan::{Instance, Device};
use crate::vulkan::vkenums::{ImageType, ImageViewType, ImageLayout, ImageTiling, Sample, ImageUsage, SharingMode};
use crate::vulkan::check_errors;

use image;

use std::mem;
use std::ptr;

pub struct Image {
  image: vk::Image,
  image_view: vk::ImageView,
  memory: vk::DeviceMemory,
}

impl Image {
  pub fn new(instance: &Instance, device: &Device, location: String, image_type: ImageType, image_view_type: ImageViewType, usage: ImageUsage, format: &vk::Format, samples: Sample, initial_layout: ImageLayout, tiling: ImageTiling) -> Image {
    let image = image::open(&location.clone()).expect(&("No file or Directory at: ".to_string() + &location)).to_rgba(); 
    let (width, height) = image.dimensions();
    let image_data = image.into_raw().clone();
    
    let image_extent = vk::Extent3D { width: width, height: height, depth: 1 };
    
    let image_size: vk::DeviceSize = (width * height * 4).into();
    
    let mut texture_image: vk::Image = unsafe { mem::uninitialized() };
    let mut texture_memory: vk::DeviceMemory = unsafe { mem::uninitialized() };
    let mut texture_image_view: vk::ImageView = unsafe { mem::uninitialized() };
    
    Image::create_image(instance, device, image_type, usage, format, &image_extent, samples, initial_layout, tiling, &mut texture_image, &mut texture_memory);
    
    texture_image_view = Image::create_image_view(device, &texture_image, format, image_view_type);
    
    Image {
      image: texture_image,
      image_view: texture_image_view,
      memory: texture_memory,
    }
  }
  
  fn create_image(instance: &Instance, device: &Device, image_type: ImageType, usage: ImageUsage, format: &vk::Format, image_extent: &vk::Extent3D, samples: Sample, initial_layout: ImageLayout, tiling: ImageTiling, image: &mut vk::Image, image_memory: &mut vk::DeviceMemory) {
    
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
        extent: vk::Extent3D { width: image_extent.width, height: image_extent.height, depth: 1 },
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
      check_errors(vk.CreateImage(*device, &image_create_info, ptr::null(), image));
      vk.GetImageMemoryRequirements(*device, *image, &mut memory_requirements);
    }
    
    let memory_type_bits_index = {
      
      let mut memory_properties: vk::PhysicalDeviceMemoryProperties = unsafe { mem::uninitialized() };
      
      unsafe {
        vk_instance.GetPhysicalDeviceMemoryProperties(*phys_device, &mut memory_properties);
      }
      
      let mut index: i32 = -1;
      let properties = vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT;
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
      check_errors(vk.AllocateMemory(*device, &memory_allocate_info, ptr::null(), image_memory));
      check_errors(vk.BindImageMemory(*device, *image, *image_memory, 0));
    }
  }
  
  fn create_image_view(device: &Device, image: &vk::Image, format: &vk::Format, image_view_type: ImageViewType) -> vk::ImageView {
    let vk = device.pointers();
    let device = device.internal_object();
    
    let mut image_view: vk::ImageView = unsafe { mem::uninitialized() };
    
    let component = vk::ComponentMapping {
      r: vk::COMPONENT_SWIZZLE_IDENTITY,
      g: vk::COMPONENT_SWIZZLE_IDENTITY,
      b: vk::COMPONENT_SWIZZLE_IDENTITY,
      a: vk::COMPONENT_SWIZZLE_IDENTITY,
    };
    
    let subresource = vk::ImageSubresourceRange {
      aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
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
  
  pub fn destroy(&self, device: &Device) {
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      
      vk.DestroyImageView(*device, self.image_view, ptr::null());
      vk.DestroyImage(*device, self.image, ptr::null());
      vk.FreeMemory(*device, self.memory, ptr::null());
    }
  }
}
