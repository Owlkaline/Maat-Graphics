use vk;
use winit;
use loader;
use loader::Loader;
use loader::FunctionPointers;

use std::ptr;
use std::mem;
use std::ffi::CStr;
use std::ffi::CString;

use std::borrow::Borrow;

use winit::dpi::LogicalSize;

use ownage::OwnedOrRef;
use ownage::check_errors;

use ENGINE_VERSION;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
unsafe fn create_surface(
    vk: &vk::InstancePointers,
    instance: &vk::Instance,
    instance_extensions: Vec<CString>,
    window: &winit::Window,
) -> vk::SurfaceKHR {
  use winit::os::unix::WindowExt;
  
  match (window.borrow().get_wayland_display(), window.borrow().get_wayland_surface()) {
    (Some(display), Some(surface)) => {//wayland
      if !instance_extensions.contains(&CString::new("VK_KHR_wayland_surface").unwrap()) {
        panic!("Missing extension VK_KHR_wayland_surface");
      }
      
      let surface = {
        let infos = vk::WaylandSurfaceCreateInfoKHR {
            sType: vk::STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0, // reserved
            display: display as *mut _,
            surface: surface as *mut _,
        };
        let mut output = mem::uninitialized();
        check_errors(vk.CreateWaylandSurfaceKHR(*instance,
                                                &infos,
                                                ptr::null(),
                                                &mut output));
        output
      };
      surface
    },
    _ => {
      //xlib
      if !instance_extensions.contains(&CString::new("VK_KHR_xlib_surface").unwrap()) {
        let surface = {
          let infos = vk::XlibSurfaceCreateInfoKHR {
            sType: vk::STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0, // reserved
            dpy: window.borrow().get_xlib_display().unwrap() as *mut _,
            window: window.borrow().get_xlib_window().unwrap() as _,
          };

          let mut output = mem::uninitialized();
          check_errors(vk.CreateXlibSurfaceKHR(*instance,
                                               &infos,
                                               ptr::null(),
                                               &mut output));
          output
        };
        
        surface
      } else {//xcb
        if !instance_extensions.contains(&CString::new("VK_KHR_xcb_surface").unwrap()) {
          panic!("Missing extension VK_KHR_xcb_surface");
        }
        
        let surface = {
          let infos = vk::XcbSurfaceCreateInfoKHR {
            sType: vk::STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
            pNext: ptr::null(),
            flags: 0, // reserved
            connection: window.borrow().get_xcb_connection().unwrap() as *mut _,
            window: window.borrow().get_xlib_window().unwrap() as _,
          };

          let mut output = mem::uninitialized();
          check_errors(vk.CreateXcbSurfaceKHR(*instance,
                                              &infos,
                                              ptr::null(),
                                              &mut output));
          output
        };
        
        surface
      }
    }
  }
}

pub struct VkWindow {
  vk_instance: vk::InstancePointers,
  vk_device: vk::DevicePointers,
  instance: vk::Instance,
  device: vk::Device,
  surface: vk::SurfaceKHR,
  swapchain: vk::SwapchainKHR,
  swapchain_format: vk::Format,
  images: Vec<vk::Image>,
  graphics_queue: vk::Queue,
  present_queue: vk::Queue,
  graphics_present_family_index: (u32, u32),
  window: winit::Window,
  events_loop: winit::EventsLoop,
}

impl VkWindow {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> VkWindow {
    let function_pointers = OwnedOrRef::Ref(loader::auto_loader().unwrap());
    let entry_points = function_pointers.entry_points();
    let supported_extensions = VkWindow::supported_extensions(entry_points);
    
    let (vk_instance, instance, available_extensions, enabled_layers) = {
      VkWindow::create_instance(entry_points, 
                                &function_pointers, 
                                app_name.to_string(), 
                                app_version, 
                                should_debug, 
                                supported_extensions)
    };
    
    let (window, events_loop, surface) = {
      VkWindow::create_window(&vk_instance, 
                              &instance, app_name, 
                              width, 
                              height, 
                              available_extensions.clone())
    };
    
    let (device, physical_device, device_available_extensions) = {
      VkWindow::create_suitable_device(&vk_instance, 
                                       &instance, &surface, 
                                       available_extensions.clone(), 
                                       enabled_layers)
    };
    let vk_device = VkWindow::create_device_isntance(&vk_instance, &device);
    let (swapchain, swapchain_format, graphics_family, present_family, graphics_queue, present_queue) = VkWindow::create_swapchain(&vk_instance, &vk_device, &physical_device, &device, &surface);
    let swapchain_images = VkWindow::create_swapchain_images(&vk_device, &device, &swapchain);
    VkWindow {
      vk_instance: vk_instance,
      vk_device: vk_device,
      instance: instance,
      device: device,
      surface: surface,
      swapchain: swapchain,
      swapchain_format: swapchain_format,
      images: swapchain_images,
      graphics_queue: graphics_queue,
      present_queue: present_queue,
      graphics_present_family_index: (graphics_family, present_family),
      window: window,
      events_loop: events_loop,
    }
  }
  
  pub fn get_events(&mut self) -> &mut winit::EventsLoop {
    &mut self.events_loop
  }
  
  pub fn device_pointers(&self) -> &vk::DevicePointers {
    &self.vk_device
  }
  
  pub fn device(&self) -> &vk::Device {
    &self.device
  }
  
  pub fn get_graphics_queue(&self) -> &vk::Queue {
    &self.graphics_queue
  }
  
  pub fn get_present_queue(&self) -> &vk::Queue {
    &self.present_queue
  }
  
  pub fn get_graphics_family(&self) -> u32 {
    self.graphics_present_family_index.0
  }
  
  fn supported_extensions(entry_points: &vk::EntryPoints) -> Vec<CString> {
    let properties: Vec<vk::ExtensionProperties> = unsafe {
      let mut num = 0;
      check_errors(entry_points.EnumerateInstanceExtensionProperties(
                                     ptr::null(), &mut num, ptr::null_mut()));
      let mut properties = Vec::with_capacity(num as usize);
      check_errors(entry_points.EnumerateInstanceExtensionProperties(
                        ptr::null(), &mut num, properties.as_mut_ptr()));
      properties.set_len(num as usize);
      properties
    };
    
    let supported_extensions: Vec<CString>
     = properties.iter().map(|x| unsafe { CStr::from_ptr(x.extensionName.as_ptr()) }.to_owned()).collect();
     
     supported_extensions
  }
  
  fn create_window(vk_instance: &vk::InstancePointers, instance: &vk::Instance, app_name: String, width: f32, height: f32, available_extensions: Vec<CString>) -> (winit::Window, winit::EventsLoop, vk::SurfaceKHR) {
    let events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new().with_title(app_name).with_dimensions(LogicalSize::new(width as f64, height as f64)).build(&events_loop).unwrap();
    
    let surface = unsafe { create_surface(&vk_instance, &instance, available_extensions.clone(), &window) };
    
    (window, events_loop, surface)
  }
  
  fn create_swapchain_images(vk_device: &vk::DevicePointers, device: &vk::Device, swapchain: &vk::SwapchainKHR) -> Vec<vk::Image> {
    
    let mut image_count = 0;
    let mut images: Vec<vk::Image>;
    
    unsafe {
      check_errors(vk_device.GetSwapchainImagesKHR(*device, *swapchain, &mut image_count, ptr::null_mut()));
      images = Vec::with_capacity(image_count as usize);
      check_errors(vk_device.GetSwapchainImagesKHR(*device, *swapchain, &mut image_count, images.as_mut_ptr()));
      images.set_len(image_count as usize);
    }
    
    images
  }
  
  fn create_swapchain(vk_instance: &vk::InstancePointers, vk_device: &vk::DevicePointers, physical_device: &vk::PhysicalDevice, device: &vk::Device, surface: &vk::SurfaceKHR) -> (vk::SwapchainKHR, vk::Format, u32, u32, vk::Queue, vk::Queue) {
    
    let mut surface_capabilities: vk::SurfaceCapabilitiesKHR = unsafe { mem::uninitialized() };
    
    unsafe {
      check_errors(vk_instance.GetPhysicalDeviceSurfaceCapabilitiesKHR(*physical_device, *surface, &mut surface_capabilities));
    }
    
    let current_extent = surface_capabilities.currentExtent;
    let supported_composite_alpha = surface_capabilities.supportedCompositeAlpha;
    let supported_usage_flags: vk::ImageUsageFlagBits = surface_capabilities.supportedUsageFlags;
    let current_transform: vk::SurfaceTransformFlagBitsKHR = surface_capabilities.currentTransform;
    
    let mut surface_formats: Vec<vk::SurfaceFormatKHR>;
    let mut num_surface_formats = 0;
    
    let mut present_modes: Vec<vk::PresentModeKHR>;
    let mut num_present_modes = 0;
    
    let mut image_count = surface_capabilities.minImageCount + 1;
    if surface_capabilities.maxImageCount > 0 && image_count > surface_capabilities.maxImageCount {
      image_count = surface_capabilities.maxImageCount;
    }
    
    unsafe {
      check_errors(vk_instance.GetPhysicalDeviceSurfaceFormatsKHR(*physical_device, *surface, &mut num_surface_formats, ptr::null_mut()));
      surface_formats = Vec::with_capacity(num_surface_formats as usize);
      check_errors(vk_instance.GetPhysicalDeviceSurfaceFormatsKHR(*physical_device, *surface, &mut num_surface_formats, surface_formats.as_mut_ptr()));
      surface_formats.set_len(num_surface_formats as usize);
      
      check_errors(vk_instance.GetPhysicalDeviceSurfacePresentModesKHR(*physical_device, *surface, &mut num_present_modes, ptr::null_mut()));
      present_modes = Vec::with_capacity(num_present_modes as usize);
      check_errors(vk_instance.GetPhysicalDeviceSurfacePresentModesKHR(*physical_device, *surface, &mut num_present_modes, present_modes.as_mut_ptr()));
      present_modes.set_len(num_surface_formats as usize);
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
    
    let (graphics_family, present_family, graphics_queue, present_queue) = VkWindow::find_queue_families(vk_instance, vk_device, device, physical_device, surface);
    
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
      oldSwapchain: 0,
    };
    let mut swapchain: vk::SwapchainKHR = unsafe { mem::uninitialized() };
    unsafe {
      check_errors(vk_device.CreateSwapchainKHR(*device, &swapchain_info, ptr::null(), &mut swapchain));
    }
    
    (swapchain, format, graphics_family, present_family, graphics_queue, present_queue)
  }
  
  fn find_queue_families(vk_instance: &vk::InstancePointers, vk_device: &vk::DevicePointers, device: &vk::Device, p_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR) -> (u32, u32, vk::Queue, vk::Queue) {
    let mut queue_count = 0;
    let mut queue_family_properties: Vec<vk::QueueFamilyProperties>;
    
    unsafe {
      vk_instance.GetPhysicalDeviceQueueFamilyProperties(*p_device, &mut queue_count, ptr::null_mut());
      queue_family_properties = Vec::with_capacity(queue_count as usize);
      vk_instance.GetPhysicalDeviceQueueFamilyProperties(*p_device, &mut queue_count, queue_family_properties.as_mut_ptr());
      queue_family_properties.set_len(queue_count as usize);
    }
    
    let mut graphics_family: i32 = -1;
    let mut present_family: i32 = -1;
    
    for i in 0..queue_family_properties.len() {
      let queue_family = &queue_family_properties[i];
      if queue_family.queueCount > 0 && VkWindow::has_graphics_bit(&queue_family.queueFlags) {
        graphics_family = i as i32;
      }
      
      let mut present_supported = 0;
      unsafe {
        check_errors(vk_instance.GetPhysicalDeviceSurfaceSupportKHR(*p_device, i as u32, *surface, &mut present_supported));
      }
      
      if queue_family.queueCount > 0 && present_supported != 0 {
         present_family = i as i32;
      }
      
      if graphics_family > 0 && present_family > 0 {
        break;
      }
    }
    
    let mut graphics_queue: vk::Queue = unsafe { mem::uninitialized() };
    let mut present_queue: vk::Queue = unsafe { mem::uninitialized() };
    
    unsafe {
      vk_device.GetDeviceQueue(*device, graphics_family as u32, 0, &mut graphics_queue);
      vk_device.GetDeviceQueue(*device, present_family as u32, 0, &mut present_queue);
    }
    
    (graphics_family as u32, present_family as u32, graphics_queue, present_queue)
  }
  
  fn print_physical_device_details(vk_instance: &vk::InstancePointers, physical_devices: &Vec<vk::PhysicalDevice>) {
    for i in 0..physical_devices.len() as usize {
      let mut device_prop: vk::PhysicalDeviceProperties = unsafe { mem::uninitialized() };
      
      unsafe {
        vk_instance.GetPhysicalDeviceProperties(physical_devices[i], &mut device_prop);
      }
      
      let device_name = device_prop.deviceName.iter().map(|a| { 
        let mut b = (*a as u8 as char).to_string();
        if b == "\u{0}".to_string() {
          b = "".to_string();
        }
        b
      }).collect::<String>();
      
      let device_type = device_prop.deviceType;
      let mut device_type_name = "";
      match device_type {
        vk::PHYSICAL_DEVICE_TYPE_OTHER => { device_type_name = "Other GPU"; },
        vk::PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU => { device_type_name = "Integrated GPU"; },
        vk::PHYSICAL_DEVICE_TYPE_DISCRETE_GPU => { device_type_name = "Discrete GPU"; },
        vk::PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU => { device_type_name = "Virtual GPU"; },
        vk::PHYSICAL_DEVICE_TYPE_CPU => { device_type_name = "CPU"; },
        _ => {},
      }
      
      println!("{}: {} -> {}", i, device_type_name, device_name);
    }
    
    for i in 0..physical_devices.len() {
      println!("Device: {}", i);
      let mut family_count = 0;
      
      unsafe {
        vk_instance.GetPhysicalDeviceQueueFamilyProperties(physical_devices[i], &mut family_count, ptr::null_mut());
      }
      
      let mut family_properties = Vec::with_capacity(family_count as usize);
      
      unsafe {
        vk_instance.GetPhysicalDeviceQueueFamilyProperties(physical_devices[i], &mut family_count, family_properties.as_mut_ptr());
        family_properties.set_len(family_count as usize);
      }
      
      let mut queue_index = 0;
      for j in 0..family_properties.len() {
        println!("  Queue: {}", j);
        let mut queue_flags = family_properties[j].queueFlags;
        if VkWindow::has_graphics_bit(&queue_flags) {
          println!("    Graphics: True");
          queue_flags -= 1;
        } else {
          println!("    Graphics: False");
        };
        if queue_flags >= 8 {
          println!("     Binding: True");
          queue_flags -= 8;
        } else {
          println!("     Binding: False");
        }
        if queue_flags >= 4 {
          println!("    Transfer: True");
          queue_flags -= 4;
        } else {
          println!("    Transfer: False");
        }
        if queue_flags != 0 {
          println!("     Compute: True");
        } else {
          println!("     Compute: False");
        }
      }
    }
  }
  
  fn create_suitable_device(vk_instance: &vk::InstancePointers, instance: &vk::Instance, surface: &vk::SurfaceKHR, available_extensions: Vec<CString>, layer_names: Vec<CString>) -> (vk::Device, vk::PhysicalDevice, Vec<CString>) {
    let layers_names_raw: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();
    
    let mut physical_device_count = 0;
    unsafe {
      check_errors(vk_instance.EnumeratePhysicalDevices(*instance, &mut physical_device_count, ptr::null_mut()));
    }
    println!("Number of usable GPUs: {}", physical_device_count);
    
    let mut physical_devices: Vec<vk::PhysicalDevice> = Vec::with_capacity(physical_device_count as usize);
    
    unsafe {
      check_errors(vk_instance.EnumeratePhysicalDevices(*instance, &mut physical_device_count, physical_devices.as_mut_ptr()));
      physical_devices.set_len(physical_device_count as usize);
    }
    
    VkWindow::print_physical_device_details(&vk_instance, &physical_devices);
    
    let mut device: vk::Device = unsafe { mem::uninitialized() };
    let mut device_available_extensions = Vec::new();
    let mut physical_device_index = 0;
    
    for i in 0..physical_devices.len() {
      let mut family_count = 0;
      
      unsafe {
        vk_instance.GetPhysicalDeviceQueueFamilyProperties(physical_devices[i], &mut family_count, ptr::null_mut());
      }
      
      let mut family_properties = Vec::with_capacity(family_count as usize);
      
      unsafe {
        vk_instance.GetPhysicalDeviceQueueFamilyProperties(physical_devices[i], &mut family_count, family_properties.as_mut_ptr());
        family_properties.set_len(family_count as usize);
      }
      
      let mut has_graphics_bit = false;
      let mut device_supports_surface: u32 = 0;
      let mut supported_queue_fam_index = 0;
      
      let mut queue_index = 0;
      for j in 0..family_properties.len() {
        let queue_count = family_properties[j].queueCount;
        let mut queue_flags = family_properties[j].queueFlags;
        if VkWindow::has_graphics_bit(&queue_flags) {
          has_graphics_bit = true;
        }
        
        if device_supports_surface == 0 {
          
          unsafe {
            vk_instance.GetPhysicalDeviceSurfaceSupportKHR(physical_devices[i], j as u32, *surface, &mut device_supports_surface);
          }
          
          if device_supports_surface != 0 {
            supported_queue_fam_index = j;
          }
        }
      }
      
      if has_graphics_bit && device_supports_surface != 0 {
        let mut property_count = 0;
        let mut device_extensions;
        
        unsafe {
          vk_instance.EnumerateDeviceExtensionProperties(physical_devices[i], ptr::null(), &mut property_count, ptr::null_mut());
          device_extensions = Vec::with_capacity(property_count as usize);
          vk_instance.EnumerateDeviceExtensionProperties(physical_devices[i], ptr::null(), &mut property_count, device_extensions.as_mut_ptr());
          device_extensions.set_len(property_count as usize);
        }
        
        let mut available_extensions = available_extensions;
        available_extensions.push(CString::new("VK_KHR_swapchain").unwrap());
        available_extensions.push(CString::new("VK_KHR_display_swapchain").unwrap());
//        available_extensions.push(CString::new("VK_KHR_sampler_mirror_clamp_to_edge").unwrap());
        //available_extensions.push(CString::new("VK_KHR_get_memory_requirements2").unwrap());
//        available_extensions.push(CString::new("VK_KHR_dedicated_allocation").unwrap());
  //      available_extensions.push(CString::new("VK_KHR_incremental_present").unwrap());
    //    available_extensions.push(CString::new("VK_EXT_debug_markers").unwrap());
        
        let supported_device_extensions: Vec<CString>
           = device_extensions.iter().map(|x| unsafe { CStr::from_ptr(x.extensionName.as_ptr()) }.to_owned()).collect();
          
          for supported_device_extension in supported_device_extensions {
            for available_extension in &available_extensions {
              if *available_extension == supported_device_extension {
                device_available_extensions.push(supported_device_extension.clone());
              }
            }
          }
          
        let device_available_extensions_raw: Vec<*const i8> = device_available_extensions.iter().map(|raw_name| raw_name.as_ptr()).collect();
        
        let mut device_queue_infos = Vec::with_capacity(family_properties.len());
        for j in 0..family_properties.len() {
          let queue_count = family_properties[j].queueCount;
          let queue_flags = family_properties[j].queueFlags;
          device_queue_infos.push( 
            vk::DeviceQueueCreateInfo {
              sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
              pNext: ptr::null(),
              flags: 0,//Default::default(),//queue_flags,
              queueFamilyIndex: j as u32,
              queueCount: family_count-1,
              pQueuePriorities: &1.0,
            }
          );
        }
        
        let mut features: vk::PhysicalDeviceFeatures = unsafe { mem::uninitialized() };
        
        unsafe {
          vk_instance.GetPhysicalDeviceFeatures(physical_devices[physical_device_index], &mut features);
        }
        
        features.robustBufferAccess = vk::TRUE;
        
        let device_info = vk::DeviceCreateInfo {
          sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
          pNext: ptr::null(),
          flags: 0,
          queueCreateInfoCount: family_properties.len() as u32,
          pQueueCreateInfos: device_queue_infos.as_ptr(),
          ppEnabledLayerNames: layers_names_raw.as_ptr(),
          enabledLayerCount: layers_names_raw.len() as u32,
          ppEnabledExtensionNames: device_available_extensions_raw.as_ptr(),
          enabledExtensionCount: device_available_extensions_raw.len() as u32,
          pEnabledFeatures: &features, // For more features use vk::GetPhysicalDeviceFeatures
        };
        
        unsafe {
          check_errors(vk_instance.CreateDevice(physical_devices[i], &device_info, ptr::null(), &mut device));
        }
        physical_device_index = i;
        break;
      }
    }
    
    (device, physical_devices[physical_device_index], device_available_extensions)
  }
  
  fn create_device_isntance(vk_instance: &vk::InstancePointers, device: &vk::Device) -> vk::DevicePointers {
    let vk_device = vk::DevicePointers::load(|name| unsafe {
      vk_instance.GetDeviceProcAddr(*device, name.as_ptr()) as *const _
    });
    
    vk_device
  }
  
  fn create_instance(entry_points: &vk::EntryPoints, function_pointers: &OwnedOrRef<FunctionPointers<Box<dyn Loader + Sync + Send>>>, app_name: String, app_version: u32, should_debug: bool, supported_extensions: Vec<CString>) -> (vk::InstancePointers, vk::Instance, Vec<CString>, Vec<CString>) {
    let app_name = CString::new(app_name).unwrap();
    let engine_name = CString::new("Maat-Graphics").unwrap();
    
    let layer_names = {
      if should_debug {
        [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()]
      } else {
        [CString::new("").unwrap()]
      }
    };
    let layers_names_raw: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();
    
    let ideal_extension_names: [CString; 9] = [
      CString::new("VK_KHR_surface").unwrap(),
      CString::new("VK_KHR_xlib_surface").unwrap(),
      CString::new("VK_KHR_xcb_surface").unwrap(),
      CString::new("VK_KHR_wayland_surface").unwrap(),
      CString::new("VK_KHR_android_surface").unwrap(),
      CString::new("VK_KHR_win32_surface").unwrap(),
      CString::new("VK_MVK_ios_surface").unwrap(),
      CString::new("VK_MVK_macos_surface").unwrap(),
      CString::new("VK_EXT_debug_utils").unwrap(),
    ];
    
    let mut available_extensions = Vec::new();
    for supported_extension in &supported_extensions {
      for ideal_extension in &ideal_extension_names {
        if ideal_extension == supported_extension {
          available_extensions.push(supported_extension.clone());
        }
      }
    }
    
    let available_extensions_raw: Vec<*const i8> = available_extensions.iter().map(|raw_name| raw_name.as_ptr()).collect();
    
    let appinfo = vk::ApplicationInfo {
      pApplicationName: app_name.as_ptr(),
      sType: vk::STRUCTURE_TYPE_APPLICATION_INFO,
      pNext: ptr::null(),
      applicationVersion: app_version,
      pEngineName: engine_name.as_ptr(),
      engineVersion: ENGINE_VERSION,
      apiVersion: (1 as u32) << 22 | (0 as u32) << 12 | (5 as u32),
    };
    
    let instance: vk::Instance = unsafe {
      let mut output = mem::uninitialized();
      let instance_info = vk::InstanceCreateInfo {
        sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pNext: ptr::null(),
        flags: Default::default(),
        pApplicationInfo: &appinfo,
        ppEnabledLayerNames: layers_names_raw.as_ptr(),
        enabledLayerCount: layers_names_raw.len() as u32,
        ppEnabledExtensionNames: available_extensions_raw.as_ptr(),
        enabledExtensionCount: available_extensions_raw.len() as u32,
      };
      
      check_errors(entry_points.CreateInstance(&instance_info, ptr::null(), &mut output));
      
      output
    };
    
    let vk_instance = {
      vk::InstancePointers::load(|name| unsafe {
        mem::transmute(function_pointers.get_instance_proc_addr(instance, name.as_ptr()))
      })
    };
    
    (vk_instance, instance, available_extensions, layer_names.to_vec())
  }
  
  fn has_graphics_bit(queue_flags: &u32) -> bool {
    queue_flags % 2 != 0 
  }
}

impl Drop for VkWindow {
  fn drop(&mut self) {
    unsafe {
      println!("Waiting for device to idle");
      self.vk_device.DeviceWaitIdle(self.device);
      println!("Destroying Device and Instance");
      self.vk_device.DestroySwapchainKHR(self.device, self.swapchain, ptr::null());
      self.vk_device.DestroyDevice(self.device, ptr::null());
      self.vk_instance.DestroyInstance(self.instance, ptr::null());
    }
  }
}

