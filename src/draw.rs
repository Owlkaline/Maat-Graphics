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

use glam::{Vec3, Vec4};

pub struct Draw {
  position: Vec3,
  scale: Vec3,
  colour: Vec4,
  rotation: f32,
  colour_overlay_mix: f32,
  //text_outline: f32,
  //text_edge_width: f32,
  wrap: f32,
  text: Option<String>,
  texture: Option<String>,
  model: Option<String>,
}

impl Draw {
  pub fn new() -> Draw {
    Draw {
      position: Vec3::ZERO,
      scale: Vec3::ONE,
      colour: Vec4::new(0.0, 0.0, 0.0, 1.0),
      rotation: 0.0,
      colour_overlay_mix: 0.0,
      //text_outline: 0.0,
      //text_edge_width: 0.5,
      wrap: 100000000.0,
      text: None,
      texture: None,
      model: None,
    }
  }

  pub fn texture(texture: &str) -> Draw {
    Draw {
      texture: Some(texture.to_string()),
      colour_overlay_mix: 1.0,
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

  pub fn colour_overlay(mut self, percentage: f32) -> Draw {
    self.colour_overlay_mix = percentage;
    self
  }

  pub fn get_texture(&self) -> Option<String> {
    self.texture.clone()
  }

  pub fn get_text(&self) -> Option<String> {
    self.text.clone()
  }

  pub fn get_colour(&self) -> Vec4 {
    self.colour
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

  pub fn texture_data(&self) -> Vec<f32> {
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
      self.colour_overlay_mix,
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