use vk;

use crate::ownage::check_errors;
use crate::modules::Instance;
use crate::modules::Device;

use std::ptr;
use std::mem;

pub struct Swapchain {
  swapchain: vk::SwapchainKHR,
  images: Vec<vk::Image>,
  image_views: Vec<vk::ImageView>,
  format: vk::Format,
}

impl Swapchain {
  pub fn new(instance: &Instance, device: &Device, surface: &vk::SurfaceKHR, graphics_family: u32, present_family: u32) -> Swapchain {
    
    let (swapchain, format) = Swapchain::create_swapchain(instance, device, surface, graphics_family, present_family, None);
    let images = Swapchain::create_swapchain_images(device, &swapchain);
    let image_views = Swapchain::create_image_views(device, &images, &format);
    
    Swapchain {
      swapchain: swapchain,
      images: images,
      image_views: image_views,
      format: format,
    }
  }
  /*
  pub fn recreate_swapchain_images(&mut self, instance: &Instance, vk: &vk::DevicePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR, graphics_family: u32, present_family: u32) {
    unsafe {
      for image_view in self.image_views.iter() {
        vk.DestroyImageView(*device, *image_view, ptr::null());
      }
    }
    
    let (swapchain, format) = Swapchain::create_swapchain(instance, vk, phys_device, device, surface, graphics_family, present_family, Some(self.swapchain));
    unsafe {
      vk.DestroySwapchainKHR(*device, self.swapchain, ptr::null());
    }
    
    self.swapchain = swapchain;
    self.format = format;
    self.images = Swapchain::create_swapchain_images(vk, device, &self.swapchain);
    self.image_views = Swapchain::create_image_views(vk, device, &self.images, &self.format);
  }*/
  
  pub fn get_format(&self) -> vk::Format {
    self.format
  }
  
  pub fn get_image_views(&self) -> &Vec<vk::ImageView> {
    &self.image_views
  }
  
  pub fn get_swapchain(&self) -> &vk::SwapchainKHR {
    &self.swapchain
  }
  
  pub fn destroy(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.local_device();
    
    unsafe {
      println!("Destroying Swapchain image views");
      for image_view in self.image_views.iter() {
        vk.DestroyImageView(*device, *image_view, ptr::null());
      }
      
      println!("Destroying Swapchain");
      vk.DestroySwapchainKHR(*device, self.swapchain, ptr::null());
    }
  }
  
  fn create_image_views(device: &Device, images: &Vec<vk::Image>, format: &vk::Format) -> Vec<vk::ImageView> {
    let vk = device.pointers();
    let device = device.local_device();
    
    let mut image_views = Vec::with_capacity(images.len());
    for image in images.iter() {
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
        viewType: vk::IMAGE_VIEW_TYPE_2D,
        format: *format,
        components: component,
        subresourceRange: subresource,
      };
      
      let mut image_view: vk::ImageView = unsafe { mem::uninitialized() };
      unsafe {
        vk.CreateImageView(*device, &image_view_create_info, ptr::null(), &mut image_view);
      }
      
      image_views.push(image_view);
    }
    
    image_views
  }
  
  fn create_swapchain_images(device: &Device, swapchain: &vk::SwapchainKHR) -> Vec<vk::Image> {
    let mut image_count = 0;
    let mut images: Vec<vk::Image>;
    
    let vk = device.pointers();
    let device = device.local_device();
    
    unsafe {
      check_errors(vk.GetSwapchainImagesKHR(*device, *swapchain, &mut image_count, ptr::null_mut()));
      images = Vec::with_capacity(image_count as usize);
      check_errors(vk.GetSwapchainImagesKHR(*device, *swapchain, &mut image_count, images.as_mut_ptr()));
      images.set_len(image_count as usize);
    }
    
    images
  }
  
  fn create_swapchain(instance: &Instance, device: &Device, surface: &vk::SurfaceKHR, graphics_family: u32, present_family: u32, old_swapchain: Option<vk::SwapchainKHR>) -> (vk::SwapchainKHR, vk::Format) {
    let vk = device.pointers();
    let phys_device = device.physical_device();
    let device = device.local_device();
    
    let mut surface_capabilities: vk::SurfaceCapabilitiesKHR = instance.get_surface_capabilities(phys_device, surface);
    
    let current_extent = surface_capabilities.currentExtent;
    let supported_composite_alpha = surface_capabilities.supportedCompositeAlpha;
    let supported_usage_flags: vk::ImageUsageFlagBits = surface_capabilities.supportedUsageFlags;
    let current_transform: vk::SurfaceTransformFlagBitsKHR = surface_capabilities.currentTransform;
    
    let mut surface_formats: Vec<vk::SurfaceFormatKHR> = instance.get_physical_device_formats(phys_device, surface);
    let mut present_modes: Vec<vk::PresentModeKHR> = instance.get_present_modes(phys_device, surface);
    
    let mut image_count = surface_capabilities.minImageCount + 1;
    if surface_capabilities.maxImageCount > 0 && image_count > surface_capabilities.maxImageCount {
      image_count = surface_capabilities.maxImageCount;
    }
    
    let (format, colour_space) = {
      let ideal_format = vk::FORMAT_B8G8R8A8_UNORM;
      let mut final_format = &surface_formats[0];
      for i in 0..surface_formats.len() {
        if surface_formats[i].format == ideal_format {
          println!("Using ideal swapchain format");
          final_format = &surface_formats[i];
        }
      }
      
      (final_format.format, final_format.colorSpace)
    };
    
    let mut present_mode = {
      if present_modes.contains(&vk::PRESENT_MODE_FIFO_KHR) {
        println!("Using Fifo present mode (vsync)");
        vk::PRESENT_MODE_FIFO_KHR
      } else if present_modes.contains(&vk::PRESENT_MODE_MAILBOX_KHR) {
        println!("Using Mailbox present mode (triple buffering)");
        vk::PRESENT_MODE_MAILBOX_KHR
      } else if present_modes.contains(&vk::PRESENT_MODE_IMMEDIATE_KHR) {
        println!("Using immediate present mode");
        vk::PRESENT_MODE_IMMEDIATE_KHR
      } else {
        panic!("No present mode found!");
      }
    };
    
    let alpha;
    if supported_composite_alpha % 2 != 0 {
      alpha = vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
    } else if supported_composite_alpha == 6 || supported_composite_alpha == 2 || supported_composite_alpha == 10 {
      alpha = vk::COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR;
    } else if supported_composite_alpha == 4 || supported_composite_alpha == 12 {
      alpha = vk::COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR;
    } else {
      alpha = vk::COMPOSITE_ALPHA_INHERIT_BIT_KHR;
    }
    
    let mut image_sharing_mode;
    let mut queue_family_index_count;
    let mut queue_family_indices: Vec<u32> = Vec::new();
    
    if graphics_family != present_family {
      image_sharing_mode = vk::SHARING_MODE_CONCURRENT;
      queue_family_index_count = 2;
      queue_family_indices = vec!(graphics_family, present_family);
    } else {
      image_sharing_mode = vk::SHARING_MODE_EXCLUSIVE;
      queue_family_index_count = 0;
    }
    
    let swapchain_info = vk::SwapchainCreateInfoKHR {
      sType: vk::STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
      pNext: ptr::null(),
      flags: 0,
      surface: *surface,
      minImageCount: image_count,
      imageFormat: format,
      imageColorSpace: colour_space,
      imageExtent: current_extent,
      imageArrayLayers: 1,
      imageUsage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
      imageSharingMode: image_sharing_mode,
      queueFamilyIndexCount: queue_family_index_count,
      pQueueFamilyIndices: queue_family_indices.as_ptr(),
      preTransform: current_transform,
      compositeAlpha: alpha,
      presentMode: present_mode,
      clipped: vk::TRUE,
      oldSwapchain: if old_swapchain.is_some() { old_swapchain.unwrap() } else { 0 },
    };
    
    let mut swapchain: vk::SwapchainKHR = unsafe { mem::uninitialized() };
    unsafe {
      check_errors(vk.CreateSwapchainKHR(*device, &swapchain_info, ptr::null(), &mut swapchain));
    }
    
    (swapchain, format)
  }
}

