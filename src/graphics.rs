use drawcalls::DrawCall;
use std::collections::HashMap;
use font::GenericFont;

use winit;

use std::vec::Vec;

pub trait CoreRender {
  fn add_font(&mut self, reference: String, font: &[u8], font_texture: String);
  fn add_texture(&mut self, reference: String, location: String);
  fn pre_load_texture(&mut self, reference: String, location: String);
  fn pre_load_font(&mut self, reference: String, font: &[u8], font_texture: String);
  fn load_texture(&mut self, reference: String, location: String);
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
}

