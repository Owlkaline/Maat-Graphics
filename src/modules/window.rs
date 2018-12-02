use vk;
use winit;
use loader;
use loader::Loader;
use loader::FunctionPointers;

use std::ptr;
use std::mem;
use std::sync::Arc;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::c_void;

use std::ops::Deref;
use std::os::raw::c_char;
use std::borrow::Borrow;

use winit::dpi::LogicalSize;

use ownage::OwnedOrRef;

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

fn check_errors(result: vk::Result) -> bool {
    match result {
        vk::SUCCESS => true,
        vk::NOT_READY => { println!("Success: A fence or query has not yet completed"); true },
        vk::TIMEOUT => { println!("Success: A wait operation has not completed in the specified time"); true },
        vk::EVENT_SET => { println!("Success: An event is signaled"); true },
        vk::EVENT_RESET => { println!("Success: An event is unsignaled"); true },
        vk::INCOMPLETE => {println!("Success: A return array was too small for the result"); true },
        vk::ERROR_OUT_OF_HOST_MEMORY => panic!("Vulkan out of host memory"),
        vk::ERROR_OUT_OF_DEVICE_MEMORY => panic!("Vulkan out of device memory"),
        vk::ERROR_INITIALIZATION_FAILED => panic!("Vulkan initialization failed"),
        vk::ERROR_DEVICE_LOST => panic!("Vulkan device lost"),
        vk::ERROR_MEMORY_MAP_FAILED => panic!("Vulkan memorymap failed"),
        vk::ERROR_LAYER_NOT_PRESENT => panic!("Vulkan layer not present"),
        vk::ERROR_EXTENSION_NOT_PRESENT => panic!("Vulkan extension not present"),
        vk::ERROR_FEATURE_NOT_PRESENT => panic!("Vulkan feature not present"),
        vk::ERROR_INCOMPATIBLE_DRIVER => panic!("Vulkan incompatable driver"),
        vk::ERROR_TOO_MANY_OBJECTS => panic!("Vulkan too many objects"),
        vk::ERROR_FORMAT_NOT_SUPPORTED => panic!("Vulkan format not supported"),
        vk::ERROR_SURFACE_LOST_KHR => panic!("Vulkan surface last khr"),
        vk::ERROR_NATIVE_WINDOW_IN_USE_KHR => panic!("Vulkan window in use khr"),
        vk::SUBOPTIMAL_KHR => panic!("Vulkan suboptimal khr"),
        vk::ERROR_OUT_OF_DATE_KHR => panic!("Vulkan out of date khr"),
        vk::ERROR_INCOMPATIBLE_DISPLAY_KHR => panic!("Vulkan incompatable display khr"),
        vk::ERROR_VALIDATION_FAILED_EXT => panic!("Vulkan validation failed ext"),
        vk::ERROR_OUT_OF_POOL_MEMORY_KHR => panic!("Vulkan of out pool memory khr"),
        vk::ERROR_INVALID_SHADER_NV => panic!("Vulkan function returned \
                                               VK_ERROR_INVALID_SHADER_NV"),
        c => unreachable!("Unexpected error code returned by Vulkan: {}", c),
    }
}

pub struct VkWindow {
  vk_instance: vk::InstancePointers,
  vk_device: i32,//vk::DevicePointers,
  device: i32,//vk::Device,
  window: winit::Window,
  events_loop: winit::EventsLoop,
  surface: vk::SurfaceKHR,
  available_extensions: Vec<CString>,
}

impl VkWindow {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> VkWindow {
    let function_pointers = OwnedOrRef::Ref(loader::auto_loader().unwrap());
    let entry_points = function_pointers.entry_points();
    let supported_extensions = VkWindow::supported_extensions(entry_points);
    let (vk_instance, instance, available_extensions) = VkWindow::create_instance(entry_points, &function_pointers, app_name.to_string(), app_version, should_debug, supported_extensions);
    let (window, events_loop, surface) = VkWindow::create_window(&vk_instance, &instance, app_name, width, height, available_extensions.clone());
    
    VkWindow {
      vk_instance: vk_instance,
      vk_device: 0,
      device: 0,
      window: window,
      events_loop: events_loop,
      surface: surface,
      available_extensions: available_extensions,
    }
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
  /*
  fn create_device_isntance() -> vk::DevicePointers {
    
  }*/
  
  fn create_instance(entry_points: &vk::EntryPoints, function_pointers: &OwnedOrRef<FunctionPointers<Box<dyn Loader + Sync + Send>>>, app_name: String, app_version: u32, should_debug: bool, supported_extensions: Vec<CString>) -> (vk::InstancePointers, vk::Instance, Vec<CString>) {
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
      CString::new("VK_EXT_debug_utils").unwrap()
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
    
    (vk_instance, instance, available_extensions)
  }
}

