use ash::vk;
use ash::version::DeviceV1_0;

use image::GenericImage;
use image::GenericImageView;

use crate::modules::{VkDevice, Memory, Buffer, Vulkan};

pub struct Image {
  image: vk::Image,
  memory: Memory<u8>,
  image_view: vk::ImageView,
  width: u32,
  height: u32,
}

impl Image {
  pub fn new_device_local(device: &VkDevice, image: vk::Image, mut image_view_info: vk::ImageViewCreateInfo,
                          width: u32, height: u32) -> Image {
    let memory: Memory<u8> = Memory::<u8>::new_image_memory(device, &image, vk::MemoryPropertyFlags::DEVICE_LOCAL);
    
    let image_view = unsafe {
        device.internal()
            .create_image_view(&image_view_info, None)
            .unwrap()
    };
    
    Image {
      image,
      memory,
      image_view,
      width,
      height,
    }
  }
  
  pub fn new_present_image(device: &VkDevice, image: vk::Image, mut image_view_info: vk::ImageViewCreateInfo) -> Image {
    let image_view = unsafe {
      device.internal()
        .create_image_view(&image_view_info, None)
        .unwrap()
    };
    
    Image {
      image,
      memory: Memory::<u8>::new_empty(),
      image_view,
      width: 1,
      height: 1,
    }
  }
  
  pub fn view(&self) -> vk::ImageView {
    self.image_view
  }
  
  pub fn memory(&self) -> vk::DeviceMemory {
    self.memory.internal()
  }
  
  pub fn memory_requirements(&self, device: &VkDevice) -> vk::MemoryRequirements {
    unsafe { device.internal().get_image_memory_requirements(self.image) }
  }
  
  pub fn internal(&self) -> vk::Image {
    self.image
  }
  
  pub fn width(&self) -> u32 {
    self.width
  }
  
  pub fn height(&self) -> u32 {
    self.height
  }
}

pub struct ImageBuilder {
  image_type: vk::ImageType,
  image_view_type: vk::ImageViewType,
  format: vk::Format,
  extent: vk::Extent3D,
  mip_levels: u32,
  array_layers: u32,
  samples: vk::SampleCountFlags,
  tiling: vk::ImageTiling,
  usage: vk::ImageUsageFlags,
  sharing_mode: vk::SharingMode,
  is_depth: bool,
}

impl ImageBuilder {
  pub fn new(format: vk::Format, mip_levels: u32, array_layers: u32) -> ImageBuilder {
    let image_type = vk::ImageType::TYPE_2D;
    let image_view_type = vk::ImageViewType::TYPE_2D;
    
    let samples = vk::SampleCountFlags::TYPE_1;
    let tiling = vk::ImageTiling::OPTIMAL;
    
    let extent = vk::Extent3D {
        width: 1,
        height: 1,
        depth: 1,
      };
    
    let sharing_mode = vk::SharingMode::EXCLUSIVE;
    
    ImageBuilder {
      image_type,
      image_view_type,
      format,
      extent,
      mip_levels,
      array_layers,
      samples,
      tiling,
      usage: Default::default(),
      sharing_mode,
      is_depth: false,
    }
  }
  /*
  pub fn new_from_loaded_image(device: &VkDevice, image: image::DynamicImage) -> ImageBuilder {
    let image_type = vk::ImageType::TYPE_2D;
    let image_view_type = vk::ImageViewType::TYPE_2D;
    
    let format = vk::Format::R8G8B8A8_UNORM;
    let extent = vk::Extent3D {
      width,
      height: dimensions.1,
      depth: 1,
    };
    
    let samples = vk::SampleCountFlags::TYPE_1;
    let tiling = vk::ImageTiling::OPTIMAL;
    
    let usage = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED;
    
    let sharing_mode = vk::SharingMode::EXCLUSIVE;
    
    ImageBuilder {
      image_type,
      image_view_type,
      format,
      extent,
      mip_levels: 1,
      array_layers: 1,
      samples,
      tiling,
      usage,
      sharing_mode,
      is_depth: false,
    }
  }*/
  
  pub fn new_depth(width: u32, height: u32, mip_levels: u32, array_layers: u32, usage: vk::ImageUsageFlags) -> ImageBuilder {
    let image_type = vk::ImageType::TYPE_2D;
    let image_view_type = vk::ImageViewType::TYPE_2D;
    let format = vk::Format::D16_UNORM;
    
    let samples = vk::SampleCountFlags::TYPE_1;
    let tiling = vk::ImageTiling::OPTIMAL;
    
    let extent = vk::Extent3D {
      width: width,
      height: height,
      depth: 1,
    };
    
    let sharing_mode = vk::SharingMode::EXCLUSIVE;
    
    ImageBuilder {
      image_type,
      image_view_type,
      format,
      extent,
      mip_levels,
      array_layers,
      samples,
      tiling,
      usage,
      sharing_mode,
      is_depth: true,
    }
  }
  
  pub fn set_dimensions(mut self, width: u32, height: u32) -> ImageBuilder {
    self.extent = vk::Extent3D {
      width,
      height,
      depth: 1,
    };
    self
  }
  
  pub fn usage(mut self, usage: vk::ImageUsageFlags) -> ImageBuilder {
    self.usage = usage;
    self
  }
  
  pub fn tiling_linear(mut self) -> ImageBuilder {
    self.tiling = vk::ImageTiling::LINEAR;
    self
  }
  
  pub fn tiling_optimal(mut self) -> ImageBuilder {
    self.tiling = vk::ImageTiling::OPTIMAL;
    self
  }
  
  pub fn samples_1(mut self) -> ImageBuilder {
    self.samples = vk::SampleCountFlags::TYPE_1;
    self
  }
  
  pub fn samples_2(mut self) -> ImageBuilder {
    self.samples = vk::SampleCountFlags::TYPE_2;
    self
  }
  
  pub fn samples_4(mut self) -> ImageBuilder {
    self.samples = vk::SampleCountFlags::TYPE_4;
    self
  }
  
  pub fn samples_8(mut self) -> ImageBuilder {
    self.samples = vk::SampleCountFlags::TYPE_8;
    self
  }
  
  pub fn samples_16(mut self) -> ImageBuilder {
    self.samples = vk::SampleCountFlags::TYPE_16;
    self
  }
  
  pub fn samples_32(mut self) -> ImageBuilder {
    self.samples = vk::SampleCountFlags::TYPE_32;
    self
  }
  
  pub fn samples_64(mut self) -> ImageBuilder {
    self.samples = vk::SampleCountFlags::TYPE_64;
    self
  }
  
  pub fn build_imageview(&self, device: &VkDevice, image: &vk::Image) -> vk::ImageViewCreateInfo {
     let mut image_view_info = vk::ImageViewCreateInfo::builder()
                                                      .view_type(self.image_view_type)
                                                      .format(self.format);
    let mut aspect_mask = vk::ImageAspectFlags::COLOR;
    
    if !self.is_depth {
      image_view_info = image_view_info.components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        });
    } else {
      aspect_mask = vk::ImageAspectFlags::DEPTH;
    }
    
    image_view_info = image_view_info.subresource_range(vk::ImageSubresourceRange::builder()
                                                        .aspect_mask(aspect_mask)
                                                        .level_count(self.mip_levels)
                                                        .layer_count(self.array_layers)
                                                        .build()
                                                      );
    image_view_info = image_view_info.image(*image);
    image_view_info.build()
  }
  
  pub fn build_from_present_image(&self, device: &VkDevice, image: vk::Image) -> Image {
    let image_view_info = self.build_imageview(device, &image);
    
    Image::new_present_image(device, image, image_view_info)
  }
  
  pub fn build_device_local(&self, device: &VkDevice) -> Image {
    let image = unsafe {
      let image_create_info = vk::ImageCreateInfo::builder()
        .image_type(self.image_type)
        .format(self.format)
        .extent(self.extent)
        .mip_levels(self.mip_levels)
        .array_layers(self.array_layers)
        .samples(self.samples)
        .tiling(self.tiling)
        .usage(self.usage)
        .sharing_mode(self.sharing_mode);
      
      device.internal().create_image(&image_create_info, None).unwrap()
    };
    
    let image_view_info = self.build_imageview(device, &image);
    
    Image::new_device_local(device, image, image_view_info, self.extent.width, self.extent.height)
  }
}
