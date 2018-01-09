use drawcalls::DrawCall;
use std::collections::HashMap;
use font::GenericFont;

use winit;

use std::vec::Vec;

use cgmath::Vector2;
use cgmath::Vector3;

pub trait CoreRender {
  fn preload_model(&mut self, reference: String, location: String, texture: String);
  fn add_model(&mut self, reference: String, location: String, texture: String);
  fn load_model(&mut self, reference: String, location: String, texture: String);

  fn preload_texture(&mut self, reference: String, location: String);
  fn add_texture(&mut self, reference: String, location: String);
  fn load_texture(&mut self, reference: String, location: String);

  fn preload_font(&mut self, reference: String, font: &[u8], font_texture: String);
  fn add_font(&mut self, reference: String, font: &[u8], font_texture: String);  
  fn load_font(&mut self, reference: String, font: &[u8]);
  
  fn load_shaders(&mut self);
  fn init(&mut self);
  fn clear_screen(&mut self);
  fn pre_draw(&mut self);
  fn draw(&mut self, draw_calls: &Vec<DrawCall>);
  fn post_draw(&self);
  fn clean(&self);
  fn swap_buffers(&mut self);
  fn screen_resized(&mut self);
  fn get_dimensions(&self) -> [u32; 2];
  fn get_events(&mut self) -> &mut winit::EventsLoop;
  fn get_fonts(&self) -> HashMap<String, GenericFont>;
  fn get_dpi_scale(&self) -> f32;
  fn is_ready(&self) -> bool;
  fn dynamic_load(&mut self);
  fn show_cursor(&mut self);
  fn hide_cursor(&mut self);
  fn set_camera_location(&mut self, camera: Vector3<f32>, camera_rot: Vector2<f32>);
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32);
}

