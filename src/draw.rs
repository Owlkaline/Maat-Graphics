//// Draw Crab at 150, 150 with size of 573x300 and rotate it by 45 degrees
//          (vec!(150.0, 150.0, 573.0, 300.0,
//                1.0, 0.0, 1.0, 1.0,
//                1.0, 45.0),
//                "rust_crab", None),
//
//          // Example drawing text
//          (vec!(text_x, text_y, text_size, 0.0, // x, y, size
//                1.0,    1.0,    1.0,       1.0, // r g b a (outline colour)
//                text_outline, text_edge_width), // text outline, text edge width, this are Signed Distanced feild parameters for text.
//                "", Some("The quick brown fox")),
//
//  vec!(0.0, 0.0, 0.0, 0.0, // (x y z nothing) defines where it should place the model, 4th parameter is not used.
//                 1.0, 1.0, 1.0),     // (scale x y z) defines what it should scale by.
//                 "example_model"     // Reference name for the model loaded in with vulkan.model_load function.
//          )

use glam::{Vec2, Vec3, Vec4};
use std::collections::HashMap;

const EMPTY: f32 = 0.0;

#[derive(Clone)]
pub struct Draw {
  position: Vec3,
  scale: Vec3,
  colour: Vec4,
  rotation: f32,
  colour_overlay: Vec3,
  //text_outline: f32,
  //text_edge_width: f32,
  wrap: f32,
  text: Option<String>,
  coloured_words: HashMap<usize, Vec4>,
  texture: Option<String>,
  model: Option<String>,
  sprite_sheet: Vec2, // rows idx
  flip_horz: bool,
  flip_vert: bool,
  camera_2d_pos: Option<Vec2>,
  instensity: f32,
  adding_buffer_data: bool,
  buffer_name: Option<String>,
}

impl Draw {
  pub fn new() -> Draw {
    Draw {
      position: Vec3::ZERO,
      scale: Vec3::ONE,
      colour: Vec4::new(0.0, 0.0, 0.0, 1.0),
      rotation: 0.0,
      colour_overlay: Vec3::splat(0.0),
      wrap: 100000000.0,
      text: None,
      coloured_words: HashMap::new(),
      texture: None,
      model: None,
      sprite_sheet: Vec2::new(1.0, 0.0),
      flip_horz: false,
      flip_vert: false,
      camera_2d_pos: None,
      instensity: -1.0,
      adding_buffer_data: false,
      buffer_name: None,
    }
  }

  pub fn texture(texture: &str) -> Draw {
    Draw {
      texture: Some(texture.to_string()),
      ..Draw::new()
    }
  }

  pub fn text(text: &str) -> Draw {
    Draw {
      text: Some(text.to_string()),
      ..Draw::new()
    }
  }

  pub fn model(model: &str) -> Draw {
    Draw {
      model: Some(model.to_string()),
      ..Draw::new()
    }
  }

  pub fn draw_buffer(buffer_name: &str) -> Draw {
    Draw {
      buffer_name: Some(buffer_name.to_owned()),
      ..Draw::new()
    }
  }

  pub fn instance_render(mut self, buffer_name: &str) -> Draw {
    self.buffer_name = Some(buffer_name.to_owned());
    self.adding_buffer_data = true;
    self
  }

  pub fn adding_buffer_data(&self) -> bool {
    self.adding_buffer_data
  }

  pub fn get_buffer(&self) -> &Option<String> {
    &self.buffer_name
  }

  pub fn instensity(mut self, v: f32) -> Draw {
    self.instensity = v;
    self
  }

  pub fn position(mut self, pos: Vec3) -> Draw {
    self.position = pos;
    self
  }

  pub fn scale(mut self, scale: Vec3) -> Draw {
    self.scale = scale;
    self
  }

  pub fn colour(mut self, colour: Vec4) -> Draw {
    self.colour = colour;
    self
  }

  pub fn rotation(mut self, rotation: f32) -> Draw {
    self.rotation = rotation;
    self
  }

  pub fn wrap(mut self, wrap: f32) -> Draw {
    self.wrap = wrap;
    self
  }

  pub fn colour_overlay(mut self, overlay: Vec3) -> Draw {
    self.colour_overlay = overlay;
    self
  }

  pub fn colour_word_n(mut self, n: usize, colour: Vec4) -> Draw {
    self.coloured_words.insert(n, colour);
    self
  }

  pub fn flip_horizontally(mut self, flip: bool) -> Draw {
    self.flip_horz = flip;
    self
  }

  pub fn flip_vertically(mut self, flip: bool) -> Draw {
    self.flip_vert = flip;
    self
  }

  pub fn sprite_sheet(mut self, rows: usize, img_num: usize) -> Draw {
    self.sprite_sheet = Vec2::new(rows as f32, img_num as f32);
    self
  }

  pub fn set_2d_camera_location(pos: Vec2) -> Draw {
    Draw {
      camera_2d_pos: Some(pos),
      ..Draw::new()
    }
  }

  pub fn get_texture(&self) -> Option<String> {
    self.texture.clone()
  }

  pub fn get_text(&self) -> Option<String> {
    self.text.clone()
  }

  pub fn get_camera(&self) -> Option<Vec2> {
    self.camera_2d_pos.clone()
  }

  pub fn get_colour(&self) -> Vec4 {
    self.colour
  }

  pub fn get_coloured_words(&mut self) -> HashMap<usize, Vec4> {
    self.coloured_words.drain().collect()
  }

  pub fn get_scale(&self) -> Vec3 {
    self.scale
  }

  pub fn get_position(&self) -> Vec3 {
    self.position
  }

  pub fn get_centered(&self) -> bool {
    false
  }

  pub fn get_wrap(&self) -> f32 {
    self.wrap
  }

  pub fn texture_data(&self, time: f32) -> Vec<f32> {
    vec![
      self.position.x,
      self.position.y,
      self.scale.x,
      self.scale.y,
      self.colour.x,
      self.colour.y,
      self.colour.z,
      self.colour.w,
      1.0,
      self.rotation,
      EMPTY,
      EMPTY,
      self.sprite_sheet.x,
      self.sprite_sheet.y,
      EMPTY,
      EMPTY,
      if self.flip_horz { 1.0 } else { 0.0 },
      if self.flip_vert { 1.0 } else { 0.0 },
      EMPTY,
      EMPTY,
      self.colour_overlay.x,
      self.colour_overlay.y,
      self.colour_overlay.z,
      EMPTY,
      EMPTY,
      EMPTY,
      EMPTY,
      EMPTY,
      EMPTY,
      EMPTY,
      self.instensity,
      time,
    ]
  }

  pub fn text_data(&self) -> Vec<f32> {
    vec![
      self.position.x,
      self.position.y,
      //self.scale.x,
      //0.0,
      //self.colour.x,
      //self.colour.y,
      //self.colour.z,
      //self.colour.w,
      //1.0,
      //0.0,
      //0.0,
      //1.0,
      //self.text_outline,
      //self.text_edge_width,
    ]
  }
}
