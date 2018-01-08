use gl;

use winit;
use winit::EventsLoop;

use vulkano_win_updated::VkSurfaceBuild;
use vulkano_win_updated::required_extensions;
use vulkano_win_updated as vulkano_win;

use vulkano::device::Queue;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::instance::Instance;
use vulkano::swapchain::Swapchain;
//use vulkano::format::B8G8R8A8Unorm;
use vulkano::image::SwapchainImage;
use vulkano::swapchain::PresentMode;
use vulkano::instance::PhysicalDevice;
use vulkano::device::DeviceExtensions;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::SwapchainCreationError;
//use vulkano::swapchain::CompositeAlpha::Opaque;

use std::mem;
use std::sync::Arc;

use glutin;
use glutin::GlContext;

pub struct VkWindow {
  events: EventsLoop,
  window: vulkano_win::Window,
  queue: Arc<Queue>,
  device: Arc<Device>,
  swapchain: Arc<Swapchain>,
  images: Vec<Arc<SwapchainImage>>,
}

pub struct GlWindow {
  events: glutin::EventsLoop,
  window: glutin::GlWindow,
}

impl GlWindow {
  pub fn new(width: u32, height: u32, min_width: u32, min_height: u32, fullscreen: bool) -> GlWindow {
    println!("Using openGL");
    
    glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3));
    let events_loop = glutin::EventsLoop::new();
    let window = {
      let temp_window: glutin::WindowBuilder;
       
      if fullscreen {
       let monitor = {
         for (num, monitor) in events_loop.get_available_monitors().enumerate() {
           println!("Monitor #{}: {:?}", num, monitor.get_name());
         }

          let monitor = events_loop.get_available_monitors().nth(0).expect("Please enter a valid ID");

          println!("Using {:?}", monitor.get_name());

          monitor
        };
        // Fullscreen
        temp_window = glutin::WindowBuilder::new().with_fullscreen(Some(monitor))
                                           .with_title("Trephination - OpenGl")
      } else {
        // Windowed
        temp_window = glutin::WindowBuilder::new()
                                            .with_title("Trephination - OpenGl").with_decorations(true)
                                            .with_dimensions(width, height)
                                            .with_min_dimensions(min_width, min_height);
      }
      temp_window
    };
    
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    
    unsafe {
      gl_window.make_current().unwrap();
    }
    
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    
    GlWindow {
      events: events_loop,
      window: gl_window,
    }
  }
  
  pub fn get_dimensions(&self) -> [u32; 2] {
    let (width, height) = self.window.get_inner_size_pixels().unwrap();
    [width as u32, height as u32]
  }
  
  pub fn get_events(&mut self) -> &mut winit::EventsLoop {
    &mut self.events
  }
  
  pub fn swap_buffers(&mut self) {
    self.window.swap_buffers().unwrap();
  }
  
  pub fn resize_screen(&mut self, dimensions: [u32; 2]) {
    self.window.resize(dimensions[0], dimensions[1]);
  }
  
  pub fn get_dpi_scale(&self) -> f32 {
    self.window.hidpi_factor()
  }
}

impl VkWindow {
  pub fn new(width: u32, height: u32, min_width: u32, min_height: u32, fullscreen: bool) -> VkWindow {
    
    println!("Using Vulkan");
    
    let instance = {
      // Window specific extensions grabbed from vulkano_win
      let extensions = required_extensions();
      Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
    };
    
    let events_loop = winit::EventsLoop::new();
    let window = {
      let temp_window: vulkano_win::Window;
       
      if fullscreen {
       let monitor = {
         for (num, monitor) in events_loop.get_available_monitors().enumerate() {
           println!("Monitor #{}: {:?}", num, monitor.get_name());
         }

          let monitor = events_loop.get_available_monitors().nth(0).expect("Please enter a valid ID");

          println!("Using {:?}", monitor.get_name());

          monitor
        };
        
        // Fullscreen
        temp_window = winit::WindowBuilder::new().with_fullscreen(Some(monitor))
                                           .with_title("Trephination - Vulkan")
                                           .build_vk_surface(&events_loop, instance.clone())
                                           .unwrap();
      } else {
        // Windowed
        temp_window = winit::WindowBuilder::new().with_dimensions(width, height)
                                            .with_min_dimensions(min_width, min_height)
                                           .with_title("Trephination - Vulkan")
                                           .build_vk_surface(&events_loop, instance.clone())
                                           .unwrap();
      }
      temp_window
    };
    
    println!("Winit Vulkan Window created");
    
    let (physical, queue) = {
      let mut found_suitable_device = false;
      
      let mut physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
      
      for device in PhysicalDevice::enumerate(&instance) {
        physical = PhysicalDevice::from_index(&instance, device.index()).unwrap();
        
        for family in physical.queue_families() {
          if family.supports_graphics() && window.surface().is_supported(family).unwrap_or(false) {
           found_suitable_device = true;
           break;
          }
        }
        
        if found_suitable_device {
          println!("  {}: {} (type: {:?})", device.index(), device.name(), device.ty());
          break;
        }
      }
      
      let queue = physical.queue_families().find(|&q| {
          q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
        }
      ).expect("couldn't find a graphical queue family");
      
      (physical, queue)
    };
      /*
      let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
      
      let queue = physical.queue_families().find(|&q| {
          q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
        }
      ).expect("couldn't find a graphical queue family");
    }
    println!("Using device: {} (type: {:?})", physical.name(), physical.ty());
    for family in physical.queue_families() {
      println!("Found a queue family with {:?} queue(s)", family.queues_count());
    }*/
    
    let (device, mut queues) = {
      let device_ext = DeviceExtensions {
        khr_swapchain: true,
        .. DeviceExtensions::none()
      };
      
      Device::new(physical, physical.supported_features(), &device_ext, [(queue, 0.5)].iter().cloned()).expect("failed to create device")
    };
  
    let queue = queues.next().unwrap();
        let (swapchain, images) = {
      let caps = window.surface()
                 .capabilities(physical)
                 .expect("failure to get surface capabilities");
      
      let dimensions = caps.current_extent.unwrap_or([1024, 768]);
                   
      let format = caps.supported_formats[0].0;//B8G8R8A8Unorm;
      let alpha = caps.supported_composite_alpha.iter().next().unwrap();//Opaque;
      let min_image_count = caps.min_image_count;
      let supported_usage_flags = caps.supported_usage_flags;
      
      println!("\nSwapchain:");
      println!("  Dimensions: {:?}", dimensions);
      println!("  Format: {:?}", format);
      
      Swapchain::new(device.clone(), window.surface().clone(), min_image_count, format,
                     dimensions, 1, supported_usage_flags, &queue,
                     SurfaceTransform::Identity, alpha, PresentMode::Fifo, true, None
                    ).expect("failed to create swapchain")
    };
    
    VkWindow { window: window,
            events: events_loop,
            queue: queue,
            device: device,
            swapchain: swapchain,
            images: images,
          }
  }
  
  pub fn get_device(&self) -> Arc<Device> {
    self.device.clone()
  }
  
  pub fn get_queue(&self) -> Arc<Queue> {
    self.queue.clone()
  }
  
  pub fn get_queue_ref(&self) -> &Arc<Queue> {
    &self.queue
  }
  
  pub fn recreate_swapchain(&self, dimensions: [u32; 2]) -> Result<(Arc<Swapchain>, Vec<Arc<SwapchainImage>>), SwapchainCreationError> {
   // println!("re creating surface");
    let caps = self.window.surface()
    .capabilities(self.device.physical_device())
    .expect("failure to get surface capabilities");
   // println!("after caps");
    let dimensions = caps.current_extent.unwrap_or([1024, 768]);
    println!("Window Resized!");
    self.swapchain.recreate_with_dimension(dimensions)
  }
  
  pub fn replace_swapchain(&mut self, new_swapchain: Arc<Swapchain>) {
    mem::replace(&mut self.swapchain, new_swapchain);
  }
  
  pub fn get_images(&self) -> &Vec<Arc<SwapchainImage>> {
    &self.images
  }
  
  pub fn replace_images(&mut self, new_images: Vec<Arc<SwapchainImage>>) {
    mem::replace(&mut self.images, new_images);
  }
  
  pub fn get_swapchain(&self) -> Arc<Swapchain> {
    self.swapchain.clone()
  }
  
  pub fn get_swapchain_format(&self) -> Format {
    self.swapchain.format()
  }
  
  pub fn get_dimensions(&self) -> [u32; 2] {
    let (width, height) = self.window.window().get_inner_size_pixels().unwrap();
    [width as u32, height as u32]
  }
  
  pub fn get_events(&mut self) -> &mut EventsLoop {
    &mut self.events
  }
  
  pub fn get_dpi_scale(&self) -> f32 {
    self.window.hidpi_factor()
  }
  
  pub fn show_cursor(&mut self) {
    self.window.set_cursor(winit::MouseCursor::Default);
  }
  
  pub fn hide_cursor(&mut self) {
    self.window.set_cursor(winit::MouseCursor::NoneCursor);
  }
}
