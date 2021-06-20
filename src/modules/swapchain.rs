use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain},
};

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::{vk, Device, Entry, Instance};
use std::borrow::Cow;
use std::cell::RefCell;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::ops::Drop;

use crate::modules::{VkDevice, VkInstance, ImageBuilder, Image};

pub struct VkSwapchain {
  swapchain: vk::SwapchainKHR,
  swapchain_extent: vk::Extent2D,
  present_images: Vec<Image>,
  
  swapchain_loader: Swapchain,
  screen_resolution: vk::Extent2D,
}

impl VkSwapchain {
  pub fn new(instance: &VkInstance, device: &VkDevice, screen_resolution: vk::Extent2D) -> VkSwapchain {
    let swapchain_loader = Swapchain::new(instance, device);
    
    let (surface_capabilities, present_mode) = unsafe {
      let surface_capabilities = device.surface_loader()
          .get_physical_device_surface_capabilities(*device.phys_device(), *device.surface())
          .unwrap();
      let present_modes = device.surface_loader()
          .get_physical_device_surface_present_modes(*device.phys_device(), *device.surface())
          .unwrap();
      let present_mode = present_modes
          .iter()
          .cloned()
          .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
          .unwrap_or(vk::PresentModeKHR::IMMEDIATE);

      (surface_capabilities, present_mode)
    };

    let mut desired_image_count = surface_capabilities.min_image_count + 1;
    if surface_capabilities.max_image_count > 0
        && desired_image_count > surface_capabilities.max_image_count
    {
      desired_image_count = surface_capabilities.max_image_count;
    }
    
    let surface_resolution = match surface_capabilities.current_extent.width {
      std::u32::MAX => surface_capabilities.max_image_extent,
      std::u32::MIN => surface_capabilities.min_image_extent,
      _ => surface_capabilities.current_extent,
    };
    
    let pre_transform = if surface_capabilities
        .supported_transforms
        .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
    {
        vk::SurfaceTransformFlagsKHR::IDENTITY
    } else {
        surface_capabilities.current_transform
    };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(*device.surface())
        .min_image_count(desired_image_count)
        .image_color_space(device.surface_format().color_space)
        .image_format(device.surface_format().format)
        .image_extent(surface_resolution)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(pre_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .image_array_layers(1);
    let swapchain_extent = swapchain_create_info.image_extent;
    let swapchain = unsafe {
      swapchain_loader
          .create_swapchain(&swapchain_create_info, None)
          .unwrap()
    };
    
    let present_images_raw = unsafe {
      swapchain_loader.get_swapchain_images(swapchain).unwrap()
    };
    
    let mut present_images = Vec::new();
    for i in 0..present_images_raw.len() {
      present_images.push(
        ImageBuilder::new(device.surface_format().format, 1, 1)
            .tiling_optimal()
            .build_from_present_image(device, present_images_raw[i])
      );
    }

    VkSwapchain {
      swapchain, 
      swapchain_extent,
      present_images,
      
      swapchain_loader,
      screen_resolution,
    }
  }
  
  pub fn destroy(&self, device: &VkDevice) {
    unsafe {
      for image in &self.present_images {
        device.destroy_image_view(image.view(), None);
      }
      self.swapchain_loader.destroy_swapchain(self.swapchain, None);
    }
  }
  
  pub fn recreate(&mut self, instance: &VkInstance, device: &VkDevice) {
    *self = VkSwapchain::new(instance, device, self.screen_resolution);
  }
  
  pub fn internal(&self) -> &vk::SwapchainKHR {
    &self.swapchain
  }
  
  pub fn extent(&self) -> vk::Extent2D {
    self.swapchain_extent
  }
  
  pub fn swapchain_loader(&self) -> &Swapchain {
    &self.swapchain_loader
  }
  
  pub fn present_images(&self) -> &Vec<Image> {
    &self.present_images
  }
  
  pub fn screen_resolution(&self) -> vk::Extent2D {
    self.screen_resolution
  }
  
  pub fn set_screen_resolution(&mut self, width: u32, height: u32) {
    self.screen_resolution = vk::Extent2D {
      width,
      height,
    };
  }
}










