use vk;

use crate::vulkan::ownage::check_errors;
use crate::vulkan::Instance;
use crate::vulkan::Device;

use crate::vulkan::vkenums::{PresentMode, CompositeAlpha, ComponentSwizzle, ImageAspect, 
                             ImageViewType, SharingMode};

use crate::Logs;

use std::ptr;
use std::mem;
use std::sync::Arc; 

pub struct Swapchain {
  swapchain: vk::SwapchainKHR,
  images: Vec<vk::Image>,
  image_views: Vec<vk::ImageView>,
  format: vk::Format,
}

impl Swapchain {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, surface: &vk::SurfaceKHR, graphics_family: u32, present_family: u32, vsync: bool, triple_buffer: bool, logs: &mut Logs) -> Swapchain {
    
    let (swapchain, format) = Swapchain::create_swapchain(Arc::clone(&instance), Arc::clone(&device), surface, graphics_family, present_family, None, vsync, triple_buffer, logs);
    let images = Swapchain::get_swapchain_images(Arc::clone(&device), &swapchain);
    let image_views = Swapchain::create_image_views(Arc::clone(&device), &images, &format);
    
    Swapchain {
      swapchain: swapchain,
      images: images,
      image_views: image_views,
      format: format,
    }
  }
  
  pub fn recreate(&mut self, instance: Arc<Instance>, device: Arc<Device>, surface: &vk::SurfaceKHR, graphics_family: u32, present_family: u32, vsync: bool, triple_buffer: bool, logs: &mut Logs) {
    let old_swapchain = self.swapchain;
    let (swapchain, format) = Swapchain::create_swapchain(Arc::clone(&instance), Arc::clone(&device), surface, graphics_family, present_family, Some(old_swapchain), vsync, triple_buffer, logs);
    
    self.destroy(Arc::clone(&device));
    
    let images = Swapchain::get_swapchain_images(Arc::clone(&device), &swapchain);
    let image_views = Swapchain::create_image_views(Arc::clone(&device), &images, &format);
    
    self.swapchain = swapchain;
    self.format = format;
    self.images = images;
    self.image_views = image_views;
  }
  
  pub fn get_format(&self) -> vk::Format {
    self.format
  }
  
  pub fn get_image_views(&self) -> &Vec<vk::ImageView> {
    &self.image_views
  }
  
  pub fn get_swapchain(&self) -> &vk::SwapchainKHR {
    &self.swapchain
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    unsafe {
      println!("Destroying Swapchain image views");
      for image_view in self.image_views.iter() {
        vk.DestroyImageView(*device, *image_view, ptr::null());
      }
      
      println!("Destroying Swapchain");
      vk.DestroySwapchainKHR(*device, self.swapchain, ptr::null());
    }
  }
  
  fn create_image_views(device: Arc<Device>, images: &Vec<vk::Image>, format: &vk::Format) -> Vec<vk::ImageView> {
    let vk = device.pointers();
    let device = device.internal_object();
    
    let mut image_views = Vec::with_capacity(images.len());
    for image in images.iter() {
      let component = vk::ComponentMapping {
        r: ComponentSwizzle::Identity.to_bits(),
        g: ComponentSwizzle::Identity.to_bits(),
        b: ComponentSwizzle::Identity.to_bits(),
        a: ComponentSwizzle::Identity.to_bits(),
      };
      
      let subresource = vk::ImageSubresourceRange {
        aspectMask: ImageAspect::Colour.to_bits(),
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
        viewType: ImageViewType::Type2D.to_bits(),
        format: *format,
        components: component,
        subresourceRange: subresource,
      };
      
      let mut image_view: vk::ImageView = unsafe { mem::MaybeUninit::uninit().assume_init() };
      unsafe {
        vk.CreateImageView(*device, &image_view_create_info, ptr::null(), &mut image_view);
      }
      
      image_views.push(image_view);
    }
    
    image_views
  }
  
  fn get_swapchain_images(device: Arc<Device>, swapchain: &vk::SwapchainKHR) -> Vec<vk::Image> {
    let mut image_count = 0;
    let mut images: Vec<vk::Image>;
    
    let vk = device.pointers();
    let device = device.internal_object();
    
    unsafe {
      check_errors(vk.GetSwapchainImagesKHR(*device, *swapchain, &mut image_count, ptr::null_mut()));
      images = Vec::with_capacity(image_count as usize);
      check_errors(vk.GetSwapchainImagesKHR(*device, *swapchain, &mut image_count, images.as_mut_ptr()));
      images.set_len(image_count as usize);
    }
    
    images
  }
  
  fn create_swapchain(instance: Arc<Instance>, device: Arc<Device>, surface: &vk::SurfaceKHR, graphics_family: u32, present_family: u32, old_swapchain: Option<vk::SwapchainKHR>, vsync: bool, triple_buffer: bool, logs: &mut Logs) -> (vk::SwapchainKHR, vk::Format) {
    let vk = device.pointers();
    let phys_device = device.physical_device();
    let device = device.internal_object();
    
    let surface_capabilities: vk::SurfaceCapabilitiesKHR = instance.get_surface_capabilities(phys_device, surface);
    
    let current_extent = surface_capabilities.currentExtent;
    let supported_composite_alpha = surface_capabilities.supportedCompositeAlpha;
    //let supported_usage_flags: vk::ImageUsageFlagBits = surface_capabilities.supportedUsageFlags;
    let current_transform: vk::SurfaceTransformFlagBitsKHR = surface_capabilities.currentTransform;
    
    let surface_formats: Vec<vk::SurfaceFormatKHR> = instance.get_physical_device_formats(phys_device, surface);
    let present_modes: Vec<vk::PresentModeKHR> = instance.get_present_modes(phys_device, surface);
    
    let mut image_count = surface_capabilities.minImageCount + 1;
    if surface_capabilities.maxImageCount > 0 && image_count > surface_capabilities.maxImageCount {
      image_count = surface_capabilities.maxImageCount;
    }
    
    let (format, colour_space) = {
      let ideal_format = vk::FORMAT_B8G8R8A8_UNORM;
      let mut final_format = &surface_formats[0];
      for i in 0..surface_formats.len() {
        if surface_formats[i].format == ideal_format {
          logs.system_msg(&format!("Using ideal swapchain format"));
          final_format = &surface_formats[i];
        }
      }
      
      (final_format.format, final_format.colorSpace)
    };
    
    let present_mode: u32 = {
      let mut present_type = PresentMode::Immediate.to_bits();
      if triple_buffer && present_modes.contains(&PresentMode::Mailbox.to_bits()) {
        logs.system_msg(&format!("Using Mailbox present mode (triple buffering)"));
        present_type = PresentMode::Mailbox.to_bits();
      } else if vsync && present_modes.contains(&PresentMode::Fifo.to_bits()) {
        logs.system_msg(&format!("Using Fifo present mode (vsync)"));
        present_type = PresentMode::Fifo.to_bits();
      } else if present_modes.contains(&PresentMode::Immediate.to_bits()) {
        logs.system_msg(&format!("Using immediate present mode"));
      } else {
        if present_modes.contains(&PresentMode::Mailbox.to_bits()) {
          present_type = PresentMode::Mailbox.to_bits();
        } else if present_modes.contains(&PresentMode::Fifo.to_bits()) {
          present_type = PresentMode::Fifo.to_bits();
        } else {
          logs.system_msg(&format!("No present mode found!"));
          panic!("No present mode found!");
        }
      }
      
      present_type
    };
    
    let alpha;
    if supported_composite_alpha % 2 != 0 {
      alpha = CompositeAlpha::Opaque.to_bits()
    } else if supported_composite_alpha == 6 || supported_composite_alpha == 2 || supported_composite_alpha == 10 {
      alpha = CompositeAlpha::PreMultiplied.to_bits()
    } else if supported_composite_alpha == 4 || supported_composite_alpha == 12 {
      alpha = CompositeAlpha::PostMultiplied.to_bits()
    } else {
      alpha = CompositeAlpha::Inherit.to_bits()
    }
    
    let image_sharing_mode;
    let queue_family_index_count;
    let mut queue_family_indices: Vec<u32> = Vec::new();
    
    if graphics_family != present_family {
      logs.system_msg(&format!("Concurrent sharing enabled"));
      image_sharing_mode = SharingMode::Concurrent.to_bits();
      queue_family_index_count = 2;
      queue_family_indices = vec!(graphics_family, present_family);
    } else {
      logs.system_msg(&format!("Exclusive sharing enabled"));
      image_sharing_mode = SharingMode::Exclusive.to_bits();
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
    
    let mut swapchain: vk::SwapchainKHR = unsafe { mem::MaybeUninit::uninit().assume_init() };
    unsafe {
      check_errors(vk.CreateSwapchainKHR(*device, &swapchain_info, ptr::null(), &mut swapchain));
    }
    
    (swapchain, format)
  }
}

