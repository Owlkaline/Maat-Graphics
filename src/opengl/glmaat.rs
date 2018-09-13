use settings::Settings;
use camera::Camera;
use font::GenericFont;
use drawcalls::DrawCall;

use graphics::CoreRender;
use graphics::Vertex2d;

use window::GlWindow;
use opengl::ResourceManager;
use opengl::TextureShader;
use opengl::FinalShader;

use gl;
use gl::types::*;

use winit;
use winit::dpi::LogicalSize;

use cgmath;
use cgmath::Vector4;
use cgmath::Matrix4;

use std::cmp;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::collections::HashMap;

pub struct GlMaat {
  camera: Camera,
  
  texture_projection: Matrix4<f32>,
  model_projection: Matrix4<f32>,
  
  resources: ResourceManager,
  
  texture_shader: TextureShader,
  final_shader: FinalShader,
  
  samples: u32,
  
  clear_colour: Vector4<f32>,
  min_dimensions: LogicalSize,
  
  window: GlWindow,
}

extern "system" fn opengl_debug(source: GLenum, _type: GLenum, id: GLuint, severity: GLenum, 
                    _length: GLsizei, messages: *const GLchar, _user: *mut c_void) {
  unsafe {
    println!("Source: {}, type: {}, id: {}, severity: {}, Message: {:?}", source, _type, id, severity,  CStr::from_ptr(messages));
  }
}

impl GlMaat {
  pub fn new() -> GlMaat {
    let mut settings = Settings::load();
    let dim = settings.get_resolution();
    let min_dim = settings.get_minimum_resolution();
    let fullscreen = settings.is_fullscreen();
    let vsync = settings.vsync_enabled();
    let triple_buffer = settings.triple_buffer_enabled();
    let mut samples = settings.get_msaa();
    
    let window = GlWindow::new(dim[0] as f64, dim[1] as f64, fullscreen, vsync/*, triple_buffer*/);
    
    let resources = ResourceManager::new();
    let texture_shader = TextureShader::new();
    let final_shader = FinalShader::new();
    
    if samples <= 0 {
      samples = 1;
    }
    
    let mut max_samples: GLint = 1;
    unsafe {
      gl::GetIntegerv(gl::MAX_FRAMEBUFFER_SAMPLES, &mut max_samples);
    }
    
    println!("Max MSAA: x{}", max_samples);
    samples = cmp::min(samples, max_samples as u32) as i32;
    println!("Current MSAA: x{}\n", samples);
    
    unsafe {
      gl::Enable(gl::DEBUG_OUTPUT);
      gl::DebugMessageCallback(opengl_debug, 0 as *const c_void);
    }
    
    GlMaat::set_viewport(LogicalSize::new(dim[0] as f64, dim[1] as f64), window.get_dpi_scale());
    unsafe {
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    
    GlMaat {
      camera: Camera::default_gl(),
      
      texture_projection: TextureShader::create_projection(dim[0] as f32, dim[1] as f32),
      model_projection: GlMaat::create_3d_projection(dim[0] as f32, dim[1] as f32),
      
      resources: resources,
      
      texture_shader: texture_shader,
      final_shader: final_shader,
      
      samples: samples,
      clear_colour: Vector4::new(0.0, 0.0, 0.0, 1.0),
      min_dimensions: LogicalSize::new(min_dim[0] as f64, min_dim[1] as f64),
      
      window: window,
    }
  }
  
  pub fn set_viewport(dim: LogicalSize, dpi: f64) {
    unsafe {
      gl::Viewport(0, 0, (dim.width as f64 * dpi) as i32, (dim.height as f64 * dpi) as i32);
    }
  }
  
  /**
  **  NEEDS TO BE MOVED ONCE 3D is implemented
  **/
  pub fn create_3d_projection(width: f32, height: f32) -> Matrix4<f32> {
    cgmath::perspective(cgmath::Deg(45.0), { width as f32 / height as f32 }, 0.01, 100.0) * cgmath::Matrix4::from_scale(-1.0)
  }
  
  pub fn with_title(mut self, title: String) -> GlMaat {
    self.window.set_title(title);
    self
  }
}

impl CoreRender for GlMaat {
  // Preload means it will load as soon as the call is made,
  // it is useful for loading the few textures needed to draw loading screens
  // but does stall the program until load is finished
  // 
  // Add is the recommened use for majority of the loading as it doesnt stall
  //
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
    
  }
  
  fn load_texture(&mut self, reference: String, location: String) {
    
  }
  
  // Load fonts
  fn preload_font(&mut self, reference: String, font_texture: String, font: &[u8]) {
    
  }
  
  fn add_font(&mut self, reference: String, font_texture: String, font: &[u8]) {
    
  }
  
  fn load_font(&mut self, reference: String, font_texture: String, font: &[u8]) {
    
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
    unsafe {
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
      gl::ClearColor(self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w);
    }
  }
  
  fn pre_draw(&mut self) {
    self.clear_screen();
  }
 
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
    
  }
  
  fn post_draw(&self) {
    
  }
  
  fn swap_buffers(&mut self) {
    self.window.swap_buffers();
  }
  
  fn screen_resized(&mut self, window_size: LogicalSize) {
    let mut window_size = window_size;
    if window_size.width <= 0.0 {
      window_size.width = self.min_dimensions.width;
    }
    if window_size.height <= 0.0 {
      window_size.height = self.min_dimensions.height;
    }
    GlMaat::set_viewport(window_size, self.get_dpi_scale());
    
    self.texture_projection = TextureShader::create_projection(window_size.width as f32, window_size.height as f32);
    self.model_projection = GlMaat::create_3d_projection(window_size.width as f32, window_size.height as f32);
    self.window.resize_screen(window_size);
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
    true
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
    self.clear_colour = Vector4::new(r, g, b, a);
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
