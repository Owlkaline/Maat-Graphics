use graphics::CoreRender;
use graphics::Vertex2d;
use graphics::Vertex3d;
use drawcalls::DrawCall;

use settings::Settings;

use vulkan::FinalShader;
use vulkan::TextureShader;
use vulkan::ResourceManager;

use camera::Camera;
use font::GenericFont;
use window::VkWindow;

use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::sync::FlushError;
use vulkano::format::ClearValue;
use vulkano::pipeline::viewport::Viewport;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::SwapchainCreationError;

use winit;
use winit::dpi::LogicalSize;

use cgmath::Matrix4;

use std::collections::HashMap;

impl_vertex!(Vertex2d, position, uv);
impl_vertex!(Vertex3d, position, normal, tangent, uv, colour);

// NEEDS TO BE MOVED WHEN 3D is a thing
use cgmath::perspective;
use cgmath::Deg;
pub fn create_3d_projection(width: f32, height: f32) -> Matrix4<f32> {
  perspective(Deg(45.0), { width as f32 / height as f32 }, 0.1, 100.0)
}

pub struct VkMaat {
  camera: Camera,
  
  texture_projection: Matrix4<f32>,
  model_projection: Matrix4<f32>,
  
  resources: ResourceManager,
  
  texture_shader: TextureShader,
  final_shader: FinalShader,
  
  samples: u32,
  
  clear_colour: ClearValue,
  dynamic_state: DynamicState,
  
  recreate_swapchain: bool,
  previous_frame_end: Option<Box<GpuFuture>>,
  
  window: VkWindow,
}

impl VkMaat {
  pub fn new() -> VkMaat {
    
    let mut settings = Settings::load();
    let dim = settings.get_resolution();
    let min_dim = settings.get_minimum_resolution();
    let fullscreen = settings.is_fullscreen();
    let vsync = settings.vsync_enabled();
    let triple_buffer = settings.triple_buffer_enabled();
    let samples = settings.get_msaa();
    
    let window = VkWindow::new(dim[0] as f64, dim[1] as f64, min_dim[0], min_dim[1], fullscreen, vsync, triple_buffer);
    
    let device = window.get_device();
    let queue = window.get_queue();
    let swapchain_format = window.get_swapchain_format();
    
    let texture_shader = TextureShader::create(device.clone(), dim, samples);
    let (final_shader, future) = FinalShader::create(device.clone(), queue.clone(), swapchain_format);
    
    let mut previous_frame_end = Some(Box::new(now(device.clone())) as Box<GpuFuture>);
    previous_frame_end = Some(Box::new(future.join(Box::new(previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    
    let dynamic_state = DynamicState {
                          line_width: None,
                          viewports: Some(vec![Viewport {
                            origin: [0.0, 0.0],
                            dimensions: [dim[0] as f32, dim[1] as f32],
                            depth_range: 0.0 .. 1.0,
                          }]),
                          scissors: None,
                        };
    
    VkMaat {
      camera: Camera::default_vk(),
      
      texture_projection: TextureShader::create_projection(dim[0] as f32, dim[1] as f32),
      model_projection: create_3d_projection(dim[0] as f32, dim[1] as f32),
      
      resources: ResourceManager::new(),
      
      texture_shader: texture_shader,
      final_shader: final_shader,
      
      samples: samples,
      
      clear_colour: ClearValue::Float([0.0, 0.0, 0.0, 1.0]),
      dynamic_state: dynamic_state,
      
      recreate_swapchain: false,
      previous_frame_end: previous_frame_end,
      
      window: window,
    }
  }
  
  pub fn with_title(mut self, title: String) -> VkMaat {
    self.window.set_title(title);
    self
  }
}

impl CoreRender for VkMaat {
  // Load 3D models
  fn preload_model(&mut self, reference: String, location: String) {
    
  }
  
  fn add_model(&mut self, reference: String, location: String) {
    
  }
  
  fn load_model(&mut self, reference: String, location: String) {
    
  }
  
  // Load png images
  fn preload_texture(&mut self, reference: String, location: String) {
    
  }
  
  fn add_texture(&mut self, reference: String, location: String) {
    self.load_texture(reference, location);
  }
  
  fn load_texture(&mut self, reference: String, location: String) {
    let queue = self.window.get_queue();
    self.resources.load_texture(reference, location, queue);
  }
  
  // Load fonts
  fn preload_font(&mut self, reference: String, font: &[u8], font_texture: String) {
    
  }
  
  fn add_font(&mut self, reference: String, font: &[u8], font_texture: String) {
    
  }
  
  fn load_font(&mut self, reference: String, font: &[u8]) {
    
  }
  
  // Load custom goemetry
  fn load_static_geometry(&mut self, reference: String, verticies: Vec<Vertex2d>, indicies: Vec<u32>) {
    
  }
  
  fn load_dynamic_geometry(&mut self, reference: String, verticies: Vec<Vertex2d>, indicies: Vec<u32>) {
    
  }
  
  // Creates the data buffer needed for rendering instanced objects
  fn load_instanced(&mut self, reference: String, max_instances: i32) {
    
  }
  
  // Internal use until Custom Shaders are implemented
  fn load_shaders(&mut self) {
    
  }
  
  // Initalises everything
  fn init(&mut self) {
    
  }
  
  // Standard draw calls that should be called in 98% of cases
  fn clear_screen(&mut self) {
    self.previous_frame_end.as_mut().unwrap().cleanup_finished();
  }
  
  fn pre_draw(&mut self) {
    if self.recreate_swapchain {
      let mut dimensions = {
        let dim = self.window.get_dimensions();
        [dim.width as u32, dim.height as u32]
      };
      
      if dimensions[0] <= 0 {
        dimensions[0] = 1;
      }
      if dimensions[1] <= 0 {
        dimensions[1] = 1;
      }
      
      let (new_swapchain, new_images) = match self.window.recreate_swapchain(dimensions) {
        Ok(r) => r,
        Err(SwapchainCreationError::UnsupportedDimensions) => {
          println!("UnsupportedDimensions");
          return;
        },
        Err(err) => panic!("{:?}", err)
      };
      
      self.window.replace_swapchain(new_swapchain);
      self.window.replace_images(new_images);
      
      let device = self.window.get_device();
      let samples = self.samples;
      
      self.texture_shader.recreate_framebuffer(device, dimensions, samples);
      self.final_shader.empty_framebuffer();
      
      self.dynamic_state.viewports = Some(
        vec![Viewport {
          origin: [0.0, 0.0],
          dimensions: [dimensions[0] as f32, dimensions[1] as f32],
          depth_range: 0.0 .. 1.0,
        }]
      );
      
      self.recreate_swapchain = false;
    }
    
    let images = self.window.get_images();
    self.final_shader.recreate_framebuffer(images);
  }
  
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
    if self.recreate_swapchain == true {
      return;
    }
    
    let (image_num, acquire_future) = match swapchain::acquire_next_image(self.window.get_swapchain(), None) {
      Ok(r) => r,
      Err(AcquireError::OutOfDate) => {
        self.recreate_swapchain = true;
        return;
      },
      Err(err) => panic!("{:?}", err)
    };
    
    // draw_calls
    let command_buffer: AutoCommandBuffer = {
      let mut dimensions = {
        let dim = self.window.get_dimensions();
        [dim.width as u32, dim.height as u32]
      };
      
      let mut tmp_cmd_buffer = AutoCommandBufferBuilder::primary_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family()).unwrap();
      
      tmp_cmd_buffer = self.texture_shader.begin_renderpass(tmp_cmd_buffer, false, self.clear_colour);
      
      tmp_cmd_buffer = self.texture_shader.end_renderpass(tmp_cmd_buffer);
      tmp_cmd_buffer = self.final_shader.begin_renderpass(tmp_cmd_buffer, false, image_num);
      
      let texture_image = self.texture_shader.get_texture_attachment();
      tmp_cmd_buffer = self.final_shader.draw(tmp_cmd_buffer, &self.dynamic_state, [dimensions[0] as f32, dimensions[1] as f32], self.texture_projection, texture_image);
      
      self.final_shader.end_renderpass(tmp_cmd_buffer)
          .build().unwrap() as AutoCommandBuffer
    };
    
    let future = self.previous_frame_end.take().unwrap().join(acquire_future)
                     .then_execute(self.window.get_queue(), command_buffer).expect("future")
                     .then_swapchain_present(self.window.get_queue(), self.window.get_swapchain(), image_num)
                     .then_signal_fence_and_flush();
    
    match future {
      Ok(future) => {
        future.wait(None).unwrap();
        self.previous_frame_end = Some(Box::new(future) as Box<_>);
      }
      Err(FlushError::OutOfDate) => {
        self.recreate_swapchain = true;
        self.previous_frame_end = Some(Box::new(now(self.window.get_device())) as Box<_>);
      }
      Err(e) => {
        println!("{:?}", e);
        self.previous_frame_end = Some(Box::new(now(self.window.get_device())) as Box<_>);
      }
    }
  }
  
  fn post_draw(&self) {
    
  }
  
  fn swap_buffers(&mut self) {
    
  }
  
  fn screen_resized(&mut self, window_size: LogicalSize) {
    self.recreate_swapchain = true;
  }
  
  // Cleans up program
  fn clean(&self) {
    
  }
  
  // Getters and setters
  fn get_dimensions(&self) -> LogicalSize {
    self.window.get_dimensions()
  }
  
  fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn get_fonts(&self) -> HashMap<String, GenericFont> {
    HashMap::new()
  }
  
  fn get_dpi_scale(&self) -> f64 {
    self.window.get_dpi_scale()
  }
  
  fn is_ready(&self) -> bool {
    false
  }
  
  fn dynamic_load(&mut self) {
    
  }
  
  fn set_cursor_position(&mut self, x: f32, y: f32) {
    self.window.set_cursor_position(x, y);
  }
  
  fn show_cursor(&mut self) {
    self.window.show_cursor();
  }
  
  fn hide_cursor(&mut self) {
    self.window.hide_cursor();
  }
  
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
    self.clear_colour = ClearValue::Float([r, g, b, a]);
  }
  
  fn set_camera(&mut self, camera: Camera) {
    self.camera = camera;
  }
  
  fn get_camera(&self) -> Camera {
    self.camera.to_owned()
  }
  
  fn num_drawcalls(&self) -> u32 {
    0
  }
}


