use drawcalls::DrawCall;
use std::collections::HashMap;
use font::GenericFont;
use camera::Camera;
use graphics;

use winit;
use winit::dpi::LogicalSize;

use std::vec::Vec;

#[derive(Clone)]
pub struct Vertex2d { pub position: [f32; 2], pub uv: [f32; 2] }

#[derive(Clone)]
pub struct Vertex3d { pub position: [f32; 3], pub normal: [f32; 3], pub tangent: [f32; 4], pub uv: [f32; 2], pub colour: [f32; 4] }

pub const DEFAULT_TEXTURE: &str = "Arial";

pub trait CoreRender {
  // Preload means it will load as soon as the call is made,
  // it is useful for loading the few textures needed to draw loading screens
  // but does stall the program until load is finished
  // 
  // Add is the recommened use for majority of the loading as it doesnt stall
  //
  // Load 3D models
  fn preload_model(&mut self, reference: String, location: String);
  fn add_model(&mut self, reference: String, location: String);
  fn load_model(&mut self, reference: String, location: String);
  
  // Load png images
  fn preload_texture(&mut self, reference: String, location: String);
  fn add_texture(&mut self, reference: String, location: String);
  fn load_texture(&mut self, reference: String, location: String);
  
  // Load fonts
  fn preload_font(&mut self, reference: String, font: &[u8], font_texture: String);
  fn add_font(&mut self, reference: String, font: &[u8], font_texture: String);  
  fn load_font(&mut self, reference: String, font: &[u8]);
  
  // Load custom goemetry
  fn load_static_geometry(&mut self, reference: String, verticies: Vec<graphics::Vertex2d>, indicies: Vec<u32>);
  fn load_dynamic_geometry(&mut self, reference: String, verticies: Vec<graphics::Vertex2d>, indicies: Vec<u32>);
  
  // Creates the data buffer needed for rendering instanced objects
  fn load_instanced(&mut self, reference: String, max_instances: i32);
  
  // Internal use until Custom Shaders are implemented
  fn load_shaders(&mut self);
  
  // Initalises everything
  fn init(&mut self);
  
  // Standard draw calls that should be called in 98% of cases
  fn clear_screen(&mut self);
  fn pre_draw(&mut self);
  fn draw(&mut self, draw_calls: &Vec<DrawCall>);
  fn post_draw(&self);
  fn swap_buffers(&mut self);
  fn screen_resized(&mut self, window_size: LogicalSize);
  
  // Cleans up program
  fn clean(&self);
  
  // Getters and setters
  fn get_dimensions(&self) -> LogicalSize;
  fn get_events(&mut self) -> &mut winit::EventsLoop;
  fn get_fonts(&self) -> HashMap<String, GenericFont>;
  fn get_dpi_scale(&self) -> f64;
  fn is_ready(&self) -> bool;
  fn dynamic_load(&mut self);
  fn show_cursor(&mut self);
  fn hide_cursor(&mut self);
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32);
  fn set_camera(&mut self, camera: Camera);
  fn get_camera(&self) -> Camera;
  fn num_drawcalls(&self) -> u32;
}

