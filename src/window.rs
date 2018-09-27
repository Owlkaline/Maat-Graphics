use winit;
use winit::EventsLoop;
use winit::dpi::LogicalSize;
use winit::dpi::LogicalPosition;

use settings::Settings;

use vulkano_win_updated::VkSurfaceBuild;
use vulkano_win_updated::required_extensions;

use vulkano::device::Queue;
use vulkano::device::Device;
use vulkano::format;
use vulkano::instance::Instance;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::Surface;
use vulkano::image::SwapchainImage;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::CompositeAlpha;

use vulkano::device::DeviceExtensions;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::SwapchainCreationError;

use vulkano::instance;
use vulkano::instance::PhysicalDevice;
use vulkano::instance::debug::{DebugCallback, MessageTypes};

use std::env;
use std::sync::Arc;

use cgmath::Vector2;

pub struct VkWindow {
  is_amd_gpu: bool,
  events: EventsLoop,
  surface: Arc<Surface<winit::Window>>,
  queue: Arc<Queue>,
  device: Arc<Device>,
  swapchain: Arc<Swapchain<winit::Window>>,
  images: Vec<Arc<SwapchainImage<winit::Window>>>,
  _min_max_dim: Vector2<f32>,
}

impl VkWindow {
  pub fn new(width: f64, height: f64, _min_width: u32, _min_height: u32, fullscreen: bool, vsync: bool, triple_buffer: bool) -> VkWindow {
    let mut is_amd_gpu = false;
//    env::set_var("WINIT_HIDPI_FACTOR", "1.0");
    let instance = {
      // Window specific extensions grabbed from vulkano_win
      let extensions = required_extensions();
      
      println!("List of Vulkan debugging layers available to use:");
      let mut layers = instance::layers_list().unwrap();
      while let Some(l) = layers.next() {
        println!("\t{}", l.name());
      }
      
      let layer = "VK_LAYER_LUNARG_standard_validation";
      let layers = {
        let mut display_debug = false;
        match env::var("MAAT_DEBUG") {
          Ok(val) => {
            if val == "1".to_string() {
              display_debug = true;
            }
          },
          Err(_) => {},
        }
        
        if display_debug {
          vec![layer]
        } else {
          vec!()
        }
      };
      
      //Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
      Instance::new(None, &extensions, layers).expect("failed to create Vulkan instance")
    };
    
    let all = MessageTypes {
        error: true,
        warning: true,
        performance_warning: true,
        information: true,
        debug: true,
    };
    
    let _debug_callback = DebugCallback::new(&instance, all, |msg| {
        let ty = if msg.ty.error {
            "error"
        } else if msg.ty.warning {
            "warning"
        } else if msg.ty.performance_warning {
            "performance_warning"
        } else if msg.ty.information {
            "information"
        } else if msg.ty.debug {
            "debug"
        } else {
            panic!("no-impl");
        };
        println!("{} {}: {}", msg.layer_prefix, ty, msg.description);
    }).ok();
    
   // println!("{}, {}, {}", Limits::max_sampler_allocation_count(&instance), Limits::max_sampler_anisotropy(&instance), Limits::max_descriptor_set_samplers(&instance));
    
    let events_loop = winit::EventsLoop::new();
    let surface = {
      let temp_surface: Arc<Surface<winit::Window>>;
       
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
        temp_surface = winit::WindowBuilder::new()
                                          .with_fullscreen(Some(monitor))
                                          .with_title("Vulkan Fullscreen")
                                          .build_vk_surface(&events_loop, instance.clone())
                                          .unwrap()
      } else {
        // Windowed
        temp_surface = winit::WindowBuilder::new()
                                          .with_dimensions(LogicalSize::new(width, height))
                                          .with_resizable(false)
                                          .with_title("Vulkan Windowed")
                                          .build_vk_surface(&events_loop, instance.clone())
                                          .unwrap()
      }
      temp_surface
    };
    
    println!("Winit Vulkan Window created");
    
    let (physical, queue_family) = {
      let mut found_suitable_device = false;
      
      let mut physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
      println!("Vulkano API: {}", physical.api_version());
      println!("Driver Version: {}", physical.driver_version());
      //PhysicalDevice::uuid()
      
      for device in PhysicalDevice::enumerate(&instance) {
        physical = PhysicalDevice::from_index(&instance, device.index()).unwrap();
        
        for family in physical.queue_families() {
          if family.supports_graphics() && surface.is_supported(family).unwrap_or(false) {
           found_suitable_device = true;
           break;
          }
        }
        
        if found_suitable_device {
          println!("GPU {}: {} (type: {:?})", device.index(), device.name(), device.ty());
          if device.name().contains("AMD") {
            println!("AMD card has been seleceted, secondary command buffers won't be used");
            is_amd_gpu = true;
          } else {
            println!("Non AMD card seleceted, Using secondary command buffers");
          }
          println!("push constant Limit size: {}",device.limits().max_push_constants_size());
          break;
        }
      }
      
      let queue = physical.queue_families().find(|&q| {
          q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
        }
      ).expect("couldn't find a graphical queue family");
      
      (physical, queue)
    };
    
    let (device, mut queues) = {
      let device_ext = DeviceExtensions {
        khr_swapchain: true,
        .. DeviceExtensions::none()
      };
      
      Device::new(physical, physical.supported_features(), &device_ext, [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };
    
    let settings = Settings::load();
    let min_width = settings.get_minimum_resolution()[0];
    let min_height = settings.get_minimum_resolution()[1];
    
    let queue = queues.next().unwrap();
    let (swapchain, images) = {
      let caps = surface
                 .capabilities(physical)
                 .expect("failure to get surface capabilities");
      
      let dimensions = caps.current_extent.unwrap_or([min_width, min_height]);
      
      let format = {
        let mut final_supported_format =  caps.supported_formats[0].0.clone();
        
        for (format, _colour_space) in caps.supported_formats {
          if format == format::Format::B8G8R8A8Unorm  {
            final_supported_format = format::Format::B8G8R8A8Unorm;
            break;
          }
        }
        
        final_supported_format
      };
      
      let alpha = {
        let mut final_alpha = caps.supported_composite_alpha.iter().next().unwrap();
        
        if caps.supported_composite_alpha.supports(CompositeAlpha::Opaque) {
          final_alpha = CompositeAlpha::Opaque;
        }
        final_alpha
      };
      
      let min_image_count = caps.min_image_count;
      let supported_usage_flags = caps.supported_usage_flags;
      
      let mut present_mode = {
        if vsync {
          PresentMode::Fifo
        } else if triple_buffer {
          PresentMode::Mailbox
        }  else {
          PresentMode::Immediate
        }
      };
      
      if !caps.present_modes.supports(present_mode) {
        if present_mode == PresentMode::Mailbox {
          print!("Error {:?} mode (Triple buffering) not supported", present_mode);
        } else {
          print!("Error {:?} mode (Vsync Off) not supported", present_mode);
        }
        
        println!(", switched to {:?} mode (Vsync On) instead", PresentMode::Fifo);
        present_mode = PresentMode::Fifo;
      }
      
      println!("Using {:?} display mode\n", present_mode);
      
      //println!("Max MSAA: {}", physical.limits().max_sampler_anisotropy());
      println!("\nSwapchain:");
      println!("  Dimensions: {:?}", dimensions);
      println!("  Format: {:?}", format);
      
      Swapchain::new(Arc::clone(&device), surface.clone(), min_image_count, format,
                     dimensions, 1, supported_usage_flags, &queue,
                     SurfaceTransform::Identity, alpha, present_mode, true, None
                    ).expect("failed to create swapchain")
    };
    
    VkWindow {
      is_amd_gpu: is_amd_gpu,
      surface: surface,
      events: events_loop,
      queue: queue,
      device: device,
      swapchain: swapchain,
      images: images,
      _min_max_dim: Vector2::new(min_width as f32, min_height as f32),
    }
  }
  
  /// Sets the title of the window
  pub fn set_title(&mut self, title: String) {
    self.surface.window().set_title(&title);
  }
  
  pub fn gpu_is_amd(&self) -> bool {
    self.is_amd_gpu
  }
  
 /* fn get_max_msaa(&self) -> u32 {
    self.device.physical_device().limits().max_sampler_anisotropy() as u32
  }*/
  
  // Returns a clone of device
  pub fn get_device(&self) -> Arc<Device> {
    Arc::clone(&self.device)
  }
  
  // Returns a clone of the queue
  pub fn get_queue(&self) -> Arc<Queue> {
    Arc::clone(&self.queue)
  }
  
  // Returns the queue as a reference
  pub fn get_queue_ref(&self) -> &Arc<Queue> {
    &self.queue
  }
  
  // Recrates the swapchain to keep it relevant to the surface dimensions
  pub fn recreate_swapchain(&self, dimensions: [u32; 2]) -> Result<(Arc<Swapchain<winit::Window>>, Vec<Arc<SwapchainImage<winit::Window>>>), SwapchainCreationError> {
   /* let caps = self.surface
    .capabilities(self.device.physical_device())
    .expect("failure to get surface capabilities");
    
    let dimensions = caps.current_extent.unwrap_or([self.min_max_dim.x as u32, self.min_max_dim.y as u32]);*/
    
    println!("Window Resized, New Dimensions: {:?}", dimensions);
    self.swapchain.recreate_with_dimension(dimensions)
  }
  
  // Replaces entire swap chain memory with parameter swapchain
  pub fn replace_swapchain(&mut self, new_swapchain: Arc<Swapchain<winit::Window>>) {
    self.swapchain = new_swapchain;
  }
  
  // Returns a reference to the current swapchain image
  pub fn get_images(&self) -> &Vec<Arc<SwapchainImage<winit::Window>>> {
    &self.images
  }
  
  // Replaces the current swapchain image with parameter image with mem::replace
  pub fn replace_images(&mut self, new_images: Vec<Arc<SwapchainImage<winit::Window>>>) {
    self.images = new_images;
  }
  
  // Returns a clone of the swapchain
  pub fn get_swapchain(&self) -> Arc<Swapchain<winit::Window>> {
    Arc::clone(&self.swapchain)
  }
  
  // Returns the current swapchain format enum from vulkano::format::Format
  pub fn get_swapchain_format(&self) -> format::Format {
    self.swapchain.format()
  }
  
  /// Returns the dimensions of the window as u32
  pub fn get_dimensions(&self) -> LogicalSize {
   // let caps = self.surface
   //   .capabilities(self.device.physical_device())
   //   .expect("failure to get surface capabilities");
    
   // let dimensions = caps.current_extent.unwrap_or([self.min_max_dim.x as u32, self.min_max_dim.y as u32]);
    
   // LogicalSize::new(dimensions[0] as f64, dimensions[1] as f64)
    self.surface.window().get_inner_size().unwrap()
  }
  
  /// Returns a reference to the events loop
  pub fn get_events(&mut self) -> &mut EventsLoop {
    &mut self.events
  }
  
  /// Returns the current dpi scale factor
  ///
  /// Needed to solve issues with Hidpi monitors
  pub fn get_dpi_scale(&self) -> f64 {
    self.surface.window().get_hidpi_factor()
  }
  
  /// Enables the cursor to be drawn whilst over the window
  pub fn show_cursor(&mut self) {
    self.surface.window().hide_cursor(false);
  }
  
  /// Disables the cursor from being drawn whilst over the window
  pub fn hide_cursor(&mut self) {
    self.surface.window().hide_cursor(true);
  }
  
  pub fn set_cursor_position(&mut self, x: f32, y: f32) {
    let result = self.surface.window().set_cursor_position(LogicalPosition::new(x as f64, y as f64));
    match result {
      Ok(_) => {},
      Err(e) => {println!("{}", e);},
    }
  }
}
