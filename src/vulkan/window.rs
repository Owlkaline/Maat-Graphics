use vk;
use winit;

use crate::math;
use crate::Settings;

use crate::vulkan::sync::Semaphore;
use crate::vulkan::Swapchain;
use crate::vulkan::Instance;
use crate::vulkan::Device;

use cgmath::Vector2;
//use crate::imgui::{ImGui, FrameSize};
//use crate::imgui_winit_support;

use std::ptr;
use std::mem;
use std::u64::MAX;
use std::ffi::CString;

use std::sync::Arc;

use std::borrow::Borrow;

use winit::dpi::{LogicalSize, LogicalPosition};

use crate::vulkan::ownage::check_errors;

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSView, NSWindow};
#[cfg(target_os = "macos")]
use cocoa::base::id as cocoa_id;
#[cfg(target_os = "macos")]
use metal::CoreAnimationLayer;
#[cfg(target_os = "macos")]
use objc::runtime::YES;

#[cfg(target_os = "android")]
unsafe fn create_surface(
    instance: &Instance, window: &winit::Window,
) -> vk::SurfaceKHR {
  use winit::os::android::WindowExt;
  
  let vk = instance.pointers();
  let win = window;
  let extensions = instance.get_extensions();
  let window = win.borrow().get_native_window();
  
  if !extensions.contains(&CString::new("VK_KHR_android_surface").unwrap()) {
    panic!("Missing extension VK_KHR_android_surface");
  }
  
  let surface = {
    let infos = vk::AndroidSurfaceCreateInfoKHR {
      sType: vk::STRUCTURE_TYPE_ANDROID_SURFACE_CREATE_INFO_KHR,
      pNext: ptr::null(),
      flags: 0, // reserved
      window: window as *mut _,
    };
    
    let mut output = mem::MaybeUninit::uninit().assume_init();
    check_errors(vk.CreateAndroidSurfaceKHR(*instance.local_instance(),
                                            &infos,
                                            ptr::null(),
                                            &mut output));
    output
  };
  
  surface
}

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
        let mut output = mem::MaybeUninit::uninit().assume_init();
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

          let mut output = mem::MaybeUninit::uninit().assume_init();
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

          let mut output = mem::MaybeUninit::uninit().assume_init();
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

#[cfg(target_os = "windows")]
unsafe fn create_surface(
    instance: &Instance, win: &winit::Window,
) -> vk::SurfaceKHR {
  use winit::os::windows::WindowExt;
  
  let vk = instance.pointers();
  let extensions = instance.get_extensions();
  let hwnd = win.borrow().get_hwnd();
  
  if !extensions.contains(&CString::new("VK_KHR_win32_surface").unwrap()) {
    panic!("Missing extension VK_KHR_win32_surface");
  }
  
  let surface = {
    let infos = vk::Win32SurfaceCreateInfoKHR {
      sType: vk::STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
      pNext: ptr::null(),
      flags: 0, // reserved
      hinstance: ptr::null() as *const () as *mut _,
      hwnd: hwnd as *mut _,
    };
    
    let mut output = mem::MaybeUninit::uninit().assume_init();
    check_errors(vk.CreateWin32SurfaceKHR(*instance.local_instance(),
                                          &infos,
                                          ptr::null(),
                                          &mut output));
    output
  };
  
  surface
}



#[cfg(target_os = "macos")]
unsafe fn create_surface(
    instance: &Instance, win: &winit::Window,
) -> vk::SurfaceKHR {
    use winit::os::macos::WindowExt;
    
    let wnd: cocoa_id = mem::transmute(win.borrow().get_nswindow());
    
    let layer = CoreAnimationLayer::new();
    
    layer.set_edge_antialiasing_mask(0);
    layer.set_presents_with_transaction(false);
    layer.remove_all_animations();
    
    let view = wnd.contentView();
    
    layer.set_contents_scale(view.backingScaleFactor());
    view.setLayer(mem::transmute(layer.as_ref())); // Bombs here with out of memory
    view.setWantsLayer(YES);
    
    let view = win.borrow().get_nsview() as *const ();
    
    let vk = instance.pointers();
    let extensions = instance.get_extensions();
    
    if !extensions.contains(&CString::new("VK_MVK_macos_surface").unwrap()) {
      panic!("Missing extension VK_MVK_macos_surface");
    }
    
    let surface = {
      let infos = vk::MacOSSurfaceCreateInfoMVK {
        sType: vk::STRUCTURE_TYPE_MACOS_SURFACE_CREATE_INFO_MVK,
        pNext: ptr::null(),
        flags: 0, // reserved
        pView: view as *const _,
      };
      
      let mut output = mem::MaybeUninit::uninit().assume_init();
      check_errors(vk.CreateMacOSSurfaceMVK(*instance.local_instance(),
                                          &infos,
                                          ptr::null(),
                                          &mut output));
    output
  };
  
  surface
}


#[cfg(target_os = "ios")]
unsafe fn create_surface(
    instance: &Instance, win: &winit::Window,
) -> vk::SurfaceKHR {
    use winit::os::macos::WindowExt;
    
    let wnd: cocoa_id = mem::transmute(win.borrow().get_nswindow());
    
    let layer = CoreAnimationLayer::new();
    
    layer.set_edge_antialiasing_mask(0);
    layer.set_presents_with_transaction(false);
    layer.remove_all_animations();
    
    let view = wnd.contentView();
    
    layer.set_contents_scale(view.backingScaleFactor());
    view.setLayer(mem::transmute(layer.as_ref())); // Bombs here with out of memory
    view.setWantsLayer(YES);
    
    let view = win.borrow().get_nsview() as *const ();
    
    let vk = instance.pointers();
    let extensions = instance.get_extensions();
    
    if !extensions.contains(&CString::new("VK_MVK_ios_surface").unwrap()) {
      panic!("Missing extension VK_MVK_ios_surface");
    }
    
    let surface = {
      let infos = vk::IOSSurfaceCreateInfoMVK {
        sType: vk::STRUCTURE_TYPE_IOS_SURFACE_CREATE_INFO_MVK,
        pNext: ptr::null(),
        flags: 0, // reserved
        pView: view as *const _,
      };
      
      let mut output = mem::MaybeUninit::uninit().assume_init();
      check_errors(vk.CreateIOSSurfaceMVK(*instance.local_instance(),
                                          &infos,
                                          ptr::null(),
                                          &mut output));
    output
  };
  
  surface
}

pub struct VkWindow {
  instance: Arc<Instance>,
  device: Arc<Device>,
  surface: vk::SurfaceKHR,
  swapchain: Swapchain,
  graphics_queue: vk::Queue,
  present_queue: vk::Queue,
  graphics_present_family_index: (u32, u32),
  window: winit::Window,
  events_loop: winit::EventsLoop,
}

impl VkWindow {
  pub fn new(app_name: String, app_version: u32, should_debug: bool, settings: &Settings) -> VkWindow {
    let fullscreen = settings.is_fullscreen();
    let vsync = settings.vsync_enabled();
    let triple_buffer = settings.triple_buffer_enabled();
    let resolution = math::array2_to_vec2(settings.get_resolution());
    
    let instance = Instance::new(app_name.to_string(), app_version, should_debug);
    
    let (window, events_loop, surface) = {
      VkWindow::create_window(Arc::clone(&instance),
                              app_name, 
                              fullscreen,
                              resolution.x as f32, 
                              resolution.y as f32)
    };
    
    let device = Device::new(Arc::clone(&instance), &surface);
    
    let (graphics_family, present_family, graphics_queue, present_queue) = VkWindow::find_queue_families(Arc::clone(&instance), Arc::clone(&device), &surface);
    
    let swapchain = Swapchain::new(Arc::clone(&instance), Arc::clone(&device), &surface, 
                                   graphics_family, present_family, vsync, triple_buffer);
    
    VkWindow {
      instance: instance,
      device: device,
      surface: surface,
      swapchain: swapchain,
      graphics_queue: graphics_queue,
      present_queue: present_queue,
      graphics_present_family_index: (graphics_family, present_family),
      window: window,
      events_loop: events_loop,
    }
  }
  
  pub fn ref_window(&self) -> &winit::Window {
    &self.window
  }
  
  pub fn set_resizable(&mut self, resizable: bool) {
    self.window.set_resizable(resizable);
  }
  
  pub fn set_inner_size(&mut self, new_size: LogicalSize) {
    self.set_resizable(true);
    self.window.set_inner_size(new_size);
  }
  
  // Borderless fullscreen
  // self.window.set_decorations(false);
  // self.window.set_maximized(true);
  pub fn set_fullscreen(&mut self, fullscreen: bool) {
    if fullscreen {
      let monitor = self.window.get_current_monitor();
      self.window.set_fullscreen(Some(monitor));
    } else {
      self.window.set_fullscreen(None);
    }
  }
  
  pub fn set_icon(&mut self, location: String) {
    let image = image::open(&location.clone()).expect(&("Icon not found: ".to_string() + &location)).to_rgba();
    let (width, height) = image.dimensions();
    let image_data = image.clone().into_raw();
    let some_icon = winit::Icon::from_rgba(image_data, width, height);
    if let Ok(icon) = some_icon {
      self.window.set_window_icon(Some(icon));
    }
  }
  
  pub fn get_max_resolution(&self) -> Vector2<f32> {
    let monitor = self.window.get_current_monitor();
    
    let max_dim = monitor.get_dimensions();
    let dpi = monitor.get_hidpi_factor();
    
    Vector2::new(max_dim.width as f32*dpi as f32, max_dim.height as f32*dpi as f32)
  }
  
  pub fn get_hidpi_factor(&self) -> f32 {
    let dpi = self.window.get_hidpi_factor();
    
    dpi as f32
  }
  
  pub fn set_cursor_position(&self, new_pos: LogicalPosition) {
    if let Err(e) = self.window.set_cursor_position(new_pos) {
      // TODO: Turn into real error
      println!("Error: {}", e.to_string());
    }
  }
  
  pub fn get_current_extent(&self) -> vk::Extent2D {
    vk::Extent2D { width: self.get_capabilities().currentExtent.width * self.get_hidpi_factor() as u32, height: self.get_capabilities().currentExtent.height * self.get_hidpi_factor() as u32 }
  }
  
  pub fn recreate_swapchain(&mut self, settings: &Settings) {
    let vsync = settings.vsync_enabled();
    let triple_buffer = settings.triple_buffer_enabled();
    self.swapchain.recreate(Arc::clone(&self.instance), Arc::clone(&self.device), &self.surface, self.graphics_present_family_index.0, self.graphics_present_family_index.1, vsync, triple_buffer);
  }
  
  pub fn get_swapchain(&self) -> &Swapchain {
    &self.swapchain
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
  
  pub fn aquire_next_image(&self, device: Arc<Device>, image_available: &Semaphore) -> (vk::Result, usize) {
    let mut current_image = 0;
    let result;
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      result = vk.AcquireNextImageKHR(*device, *self.swapchain.get_swapchain(), MAX, *image_available.internal_object(), 0, &mut current_image);
    }
    
    (result, current_image as usize)
  }
  
  pub fn instance_pointers(&self) -> &vk::InstancePointers {
    &self.instance.pointers()
  } 
  
  pub fn device_pointers(&self) -> &vk::DevicePointers {
    &self.device.pointers()
  }
  
  pub fn instance(&self) -> Arc<Instance> {
    Arc::clone(&self.instance)
  }
  
  pub fn device(&self) -> Arc<Device> {
    Arc::clone(&self.device)
  }
  
  pub fn physical_device(&self) -> &vk::PhysicalDevice {
    &self.device.physical_device()
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
  
  pub fn get_max_mssa(&self) -> u32 {
    let device_properties = self.instance.get_device_properties(&self.device.physical_device());
    let limits = device_properties.limits;
    let msaa_limit = limits.framebufferColorSampleCounts.min(limits.framebufferDepthSampleCounts);
    
    msaa_limit
  }
  
  fn get_capabilities(&self) -> vk::SurfaceCapabilitiesKHR {
    let phys_device = self.device.physical_device();
    self.instance.get_surface_capabilities(phys_device, &self.surface)
  }
  
  fn create_window(instance: Arc<Instance>, app_name: String, fullscreen: bool, width: f32, height: f32) -> (winit::Window, winit::EventsLoop, vk::SurfaceKHR) {
    let events_loop = winit::EventsLoop::new();
    let window = {
      if fullscreen {
        for (num, monitor) in events_loop.get_available_monitors().enumerate() {
          println!("Monitor #{}: {:?}", num, monitor.get_name());
        }
        
        let monitor = events_loop.get_available_monitors().nth(0).expect("Please enter a valid ID");
        
        println!("Using {:?}", monitor.get_name());
        
        // Fullscreen
        winit::WindowBuilder::new()
                             .with_fullscreen(Some(monitor))
                             .with_title(app_name)
                             .with_resizable(false)
                             //  .build_vk_surface(&events_loop, instance.clone())
                             .build(&events_loop).unwrap()
      } else {
        winit::WindowBuilder::new()
                              .with_title(app_name)
                              .with_dimensions(LogicalSize::new(width as f64, height as f64))
                              .with_resizable(false)
                              .build(&events_loop).unwrap()
      }
    };
    let surface = unsafe { create_surface(&instance, &window) };
    
    (window, events_loop, surface)
  }
  
  fn find_queue_families(instance: Arc<Instance>, device: Arc<Device>, surface: &vk::SurfaceKHR) -> (u32, u32, vk::Queue, vk::Queue) {
    let phys_device = device.physical_device();
    
    let queue_family_properties: Vec<vk::QueueFamilyProperties> = instance.get_queue_family_properties(phys_device);
    
    let mut graphics_family: i32 = -1;
    let mut present_family: i32 = -1;
    
    for i in 0..queue_family_properties.len() {
      let queue_family = &queue_family_properties[i];
      if queue_family.queueCount > 0 && VkWindow::has_graphics_bit(&queue_family.queueFlags) {
        graphics_family = i as i32;
      }
      
      let present_supported = instance.get_supported_display_queue_families(phys_device, surface, i as u32);
      
      if queue_family.queueCount > 0 && present_supported != 0 {
         present_family = i as i32;
      }
      
      if graphics_family == present_family {
        break;
      }
      /*
      let mut valid_graphics_family: i32 = -1;
      let mut valid_presents_family: i32 = -1;
      
      println!("queue_family {}: flags: {}, gq: {}, pq: {}", i, queue_family.queueFlags, graphics_family, present_family);
      if graphics_family > 0 && present_family > 0 {
        valid_graphics_family = graphics_family;
        valid_presents_family = present_family;
        let graphics_queue: vk::Queue = device.get_device_queue(graphics_family as u32, 0);
        let present_queue: vk::Queue = device.get_device_queue(present_family as u32, 0);
        
        // TODO REmove this if state to enable Concurrent graphics families
        if graphics_queue == present_queue {
          break;
        }
      }*/
    }
    
    if graphics_family != present_family {
      panic!("This is my custom error brown, I dun fucked up!");
    }
    
    let graphics_queue: vk::Queue = device.get_device_queue(graphics_family as u32, 0);
    let present_queue: vk::Queue = device.get_device_queue(present_family as u32, 0);
    
    (graphics_family as u32, present_family as u32, graphics_queue, present_queue)
  }
  
  fn has_graphics_bit(queue_flags: &u32) -> bool {
    queue_flags % 2 != 0 
  }
  /*
  pub fn imgui_window(&mut self, imgui: &mut ImGui) -> FrameSize {
    imgui_winit_support::update_mouse_cursor(&imgui, &self.window);
    imgui_winit_support::get_frame_size(&self.window, self.get_hidpi_factor() as f64).unwrap()
  }*/
}

impl Drop for VkWindow {
  fn drop(&mut self) {
    self.swapchain.destroy(Arc::clone(&self.device));
    self.device.destroy();
    self.instance.destroy();
  }
}

