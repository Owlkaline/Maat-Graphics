use vk;
use winit;
use loader;
use loader::Loader;
use loader::FunctionPointers;
use modules::Swapchain;
use modules::Instance;

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
    instance: &Instance,
    window: &winit::Window,
) -> vk::SurfaceKHR {
  use winit::os::unix::WindowExt;
  
  let vk = instance.pointers();
  let extensions = instance.get_extensions();
  let instance = instance.local_instance();
  
  match (window.borrow().get_wayland_display(), window.borrow().get_wayland_surface()) {
    (Some(display), Some(surface)) => {//wayland
      if !extensions.contains(&CString::new("VK_KHR_wayland_surface").unwrap()) {
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
      if !extensions.contains(&CString::new("VK_KHR_xlib_surface").unwrap()) {
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
        if !extensions.contains(&CString::new("VK_KHR_xcb_surface").unwrap()) {
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
  vk_device: vk::DevicePointers,
  instance: Instance,
  device: vk::Device,
  phys_device: vk::PhysicalDevice,
  surface: vk::SurfaceKHR,
  swapchain: Swapchain,
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
    
    let instance = Instance::new(app_name.to_string(), app_version, should_debug);
    
    let (window, events_loop, surface) = {
      VkWindow::create_window(&instance,
                              app_name, 
                              width, 
                              height)
    };
    
    let (device, physical_device, device_available_extensions) = {
      VkWindow::create_suitable_device(&instance, &surface)
    };
    
    let vk_device = VkWindow::create_device_instance(instance.pointers(), &device);
    let (graphics_family, present_family, graphics_queue, present_queue) = VkWindow::find_queue_families(&instance, &vk_device, &device, &physical_device, &surface);
    
    let swapchain = Swapchain::new(&instance, &vk_device, &physical_device, &device, &surface, graphics_family, present_family);
    
    VkWindow {
      vk_device: vk_device,
      instance: instance,
      device: device,
      phys_device: physical_device,
      surface: surface,
      swapchain: swapchain,
      graphics_queue: graphics_queue,
      present_queue: present_queue,
      graphics_present_family_index: (graphics_family, present_family),
      window: window,
      events_loop: events_loop,
    }
  }
  
  pub fn get_current_extent(&self) -> vk::Extent2D {
    self.get_capabilities().currentExtent
  }
  
  pub fn get_swapchain(&self) -> &vk::SurfaceKHR {
    self.swapchain.get_swapchain()
  }
  
  pub fn swapchain_image_views(&self) -> &Vec<vk::ImageView> {
    self.swapchain.get_image_views()
  }
  
  pub fn swapchain_format(&self) -> vk::Format {
    self.swapchain.get_format()
  }
  
  pub fn get_events(&mut self) -> &mut winit::EventsLoop {
    &mut self.events_loop
  }
  
  pub fn instance_pointers(&self) -> &vk::InstancePointers {
    &self.instance.pointers()
  } 
  
  pub fn device_pointers(&self) -> &vk::DevicePointers {
    &self.vk_device
  }
  
  pub fn device(&self) -> &vk::Device {
    &self.device
  }
  
  pub fn physical_device(&self) -> &vk::PhysicalDevice {
    &self.phys_device
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
  
  fn get_capabilities(&self) -> vk::SurfaceCapabilitiesKHR {
    self.instance.get_surface_capabilities(&self.phys_device, &self.surface)
  }
  
  fn create_window(instance: &Instance, app_name: String, width: f32, height: f32) -> (winit::Window, winit::EventsLoop, vk::SurfaceKHR) {
    let events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new().with_title(app_name).with_dimensions(LogicalSize::new(width as f64, height as f64)).build(&events_loop).unwrap();
    
    let surface = unsafe { create_surface(&instance, &window) };
    
    (window, events_loop, surface)
  }
  
  fn find_queue_families(instance: &Instance, vk_device: &vk::DevicePointers, device: &vk::Device, phys_device: &vk::PhysicalDevice, surface: &vk::SurfaceKHR) -> (u32, u32, vk::Queue, vk::Queue) {
    
    let queue_family_properties: Vec<vk::QueueFamilyProperties> = instance.get_queue_family_properties(phys_device);
    
    let mut graphics_family: i32 = -1;
    let mut present_family: i32 = -1;
    
    for i in 0..queue_family_properties.len() {
      let queue_family = &queue_family_properties[i];
      if queue_family.queueCount > 0 && VkWindow::has_graphics_bit(&queue_family.queueFlags) {
        graphics_family = i as i32;
      }
      
      let mut present_supported = instance.get_supported_display_queue_families(phys_device, surface, i as u32);
      
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
  
  fn create_suitable_device(instance: &Instance, surface: &vk::SurfaceKHR) -> (vk::Device, vk::PhysicalDevice, Vec<CString>) {
    let layer_names = instance.get_layers();
    let layers_names_raw: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();
    
    let physical_devices = instance.enumerate_physical_devices();
    
    VkWindow::print_physical_device_details(instance.pointers(), &physical_devices);
    
    let mut device: vk::Device = unsafe { mem::uninitialized() };
    let mut device_available_extensions = Vec::new();
    let mut physical_device_index = 0;
    
    for i in 0..physical_devices.len() {
      let family_properties = instance.get_device_queue_family_properties(&physical_devices[i]);
      
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
          
          device_supports_surface = instance.physical_device_supports_surface(&physical_devices[i], j as u32, surface);
          
          if device_supports_surface != 0 {
            supported_queue_fam_index = j;
          }
        }
      }
      
      if has_graphics_bit && device_supports_surface != 0 {
        let mut property_count = 0;
        let mut device_extensions = instance.enumerate_device_extension_properties(&physical_devices[i]);
        
        let mut available_extensions = instance.get_extensions();
        available_extensions.push(CString::new("VK_KHR_swapchain").unwrap());
        available_extensions.push(CString::new("VK_KHR_display_swapchain").unwrap());
//        available_extensions.push(CString::new("VK_KHR_sampler_mirror_clamp_to_edge").unwrap());
        available_extensions.push(CString::new("VK_KHR_get_memory_requirements2").unwrap());
        available_extensions.push(CString::new("VK_KHR_dedicated_allocation").unwrap());
        available_extensions.push(CString::new("VK_KHR_incremental_present").unwrap());
        available_extensions.push(CString::new("VK_EXT_debug_markers").unwrap());
        
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
              queueCount: family_properties.len() as u32-1,
              pQueuePriorities: &1.0,
            }
          );
        }
        
        let mut features: vk::PhysicalDeviceFeatures = instance.get_device_features(&physical_devices[physical_device_index]);
        
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
        
        device = instance.create_device(&physical_devices[i], &device_info);
        
        physical_device_index = i;
        break;
      }
    }
    
    (device, physical_devices[physical_device_index], device_available_extensions)
  }
  
  fn create_device_instance(vk_instance: &vk::InstancePointers, device: &vk::Device) -> vk::DevicePointers {
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
    let image_views = self.swapchain.get_image_views();
    unsafe {
      for image_view in image_views.iter() {
        self.vk_device.DestroyImageView(self.device, *image_view, ptr::null());
      }
      
      println!("Waiting for device to idle");
      self.vk_device.DeviceWaitIdle(self.device);
      println!("Destroying Device and Instance");
      self.vk_device.DestroySwapchainKHR(self.device, *self.swapchain.get_swapchain(), ptr::null());
      self.vk_device.DestroyDevice(self.device, ptr::null());
      self.instance.destroy();
    }
  }
}

