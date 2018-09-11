use font::GenericCharacter;
use font::GenericFont;

use graphics;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix2;
use cgmath::Matrix4;

use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum DrawType {
  DrawText,
  DrawTextured,
  DrawColoured,
  DrawModel,
  DrawCustomShapeTextured,
  DrawCustomShapeColoured,
  
  DrawInstancedColoured,
  DrawInstancedTextured,
  DrawInstancedModel,
  
  NewTexture,
  NewText,
  NewModel,
  NewShape,
  
  RemoveTexture,
  RemoveText,
  RemoveModel,
  RemoveShape,
  
  UpdateShape,
  
  NewDrawcallSet,
  DrawDrawcallSet,
  RemoveDrawcallSet,
  
  None,
}

#[derive(Clone)]
pub struct DrawCall {
  reference_name: String,
  instanced_name: Option<String>,
  shape_name: Option<String>,
  
  text: Option<String>,
  colour: Vector4<f32>,
  position: Vector3<f32>,
  rotation: Vector3<f32>,
  scale: Vector3<f32>,
  
  black_white: bool,
  
  text_outline_colour: Vector3<f32>,
  text_wrapping: u32,
  text_centered: bool,
  text_edge_width: Vector4<f32>,
  
  draw_type: DrawType,
  
  new_shape: Option<(Vec<graphics::Vertex2d>, Vec<u32>)>,
}

impl DrawCall {
  fn empty() -> DrawCall {
    DrawCall {
      reference_name: "".to_string(),
      instanced_name: None,
      shape_name: None,
      
      text: None,
      colour: Vector4::new(1.0, 1.0, 1.0, 1.0),
      position: Vector3::new(0.0, 0.0, 0.0),
      rotation: Vector3::new(0.0, 0.0, 0.0),
      scale: Vector3::new(1.0, 1.0, 1.0),
      
      black_white: false,
      
      text_outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text_wrapping: 0,
      text_centered: false,
      text_edge_width: Vector4::new(0.1, 0.1, 0.1, 0.1),
      
      draw_type: DrawType::None,
      new_shape: None,
    }
  }
  
  pub fn draw_model(position: Vector3<f32>, rotation: Vector3<f32>, scale: Vector3<f32>, model: String) -> DrawCall {
    DrawCall {
      reference_name: model,
      position: position,
      rotation: rotation,
      scale: scale,
      draw_type: DrawType::DrawModel,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_textured(position: Vector2<f32>, scale: Vector2<f32>, texture: String) -> DrawCall {
    DrawCall {
      reference_name: texture,
      position: Vector3::new(position.x, position.y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      draw_type: DrawType::DrawTextured,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_instanced_textured(position: Vector2<f32>, scale: Vector2<f32>, texture: String, instance: String) -> DrawCall {
    DrawCall {
      reference_name: texture,
      instanced_name: Some(instance),
      position: Vector3::new(position.x, position.y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      draw_type: DrawType::DrawInstancedTextured,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_coloured(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>) -> DrawCall {
    DrawCall {
      colour: colour,
      position: Vector3::new(position.x, position.y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      draw_type: DrawType::DrawColoured,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_instanced_coloured(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, instance: String) -> DrawCall {
    DrawCall {
      reference_name: "".to_string(),
      instanced_name: Some(instance),
      colour: colour,
      position: Vector3::new(position.x, position.y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      draw_type: DrawType::DrawInstancedTextured,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_custom_shape_textured(position: Vector2<f32>, scale: Vector2<f32>, texture: String, shape: String) -> DrawCall {
    DrawCall {
      reference_name: texture,
      shape_name: Some(shape),
      position: Vector3::new(position.x, position.y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      draw_type: DrawType::DrawCustomShapeTextured,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_custom_shape_coloured(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, shape: String) -> DrawCall {
    DrawCall {
      colour: colour,
      shape_name: Some(shape),
      position: Vector3::new(position.x, position.y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      draw_type: DrawType::DrawCustomShapeColoured,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_text_basic(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      reference_name: font,
      text: Some(display_text),
      colour: colour,
      position: Vector3::new(position.x, position.y, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      text_edge_width: Vector4::new(0.5, 0.1, 0.1, 0.1),
      draw_type: DrawType::DrawText,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_text_basic_wrapped(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      text_wrapping: wrap_length,
      .. DrawCall::draw_text_basic(position, scale, colour, display_text, font)
    }
  }
  
  pub fn draw_text_basic_centered(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      text_centered: true,
      .. DrawCall::draw_text_basic(position, scale, colour, display_text, font)
    }
  }
  
  pub fn draw_text_basic_wrapped_centered(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      text_centered: true,
      text_wrapping: wrap_length,
      .. DrawCall::draw_text_basic(position, scale, colour, display_text, font)
    }
  }
  
  pub fn draw_text_outlined(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      reference_name: font,
      text: Some(display_text),
      colour: colour,
      text_outline_colour: outline_colour,
      position: Vector3::new(position.x, position.y, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      text_edge_width: Vector4::new(0.5, 0.1, 0.7, 0.1),
      draw_type: DrawType::DrawText,
      .. DrawCall::empty()
    }
  }
  
  pub fn draw_text_outlined_wrapped(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      text_wrapping: wrap_length,
      .. DrawCall::draw_text_outlined(position, scale, colour, outline_colour, display_text, font)
    }
  }
  
  pub fn draw_text_outlined_centered(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      text_centered: true,
      .. DrawCall::draw_text_outlined(position, scale, colour, outline_colour, display_text, font)
    }
  }
  
  pub fn draw_text_outlined_wrapped_centered(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      text_centered: true,
      text_wrapping: wrap_length,
      .. DrawCall::draw_text_outlined(position, scale, colour, outline_colour, display_text, font)
    }
  }
  
  pub fn draw_text_custom(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, edge_width: Vector4<f32>, centered: bool, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      reference_name: font,
      text: Some(display_text),
      colour: colour,
      text_outline_colour: outline_colour,
      position: Vector3::new(position.x, position.y, 0.0),
      scale: Vector3::new(scale.x, scale.y, 1.0),
      text_edge_width: edge_width,
      text_centered: centered,
      text_wrapping: wrap_length,
      draw_type: DrawType::DrawText,
      .. DrawCall::empty()
    }
  }
  
  pub fn update_custom_shape(vertices: Vec<graphics::Vertex2d>, indices: Vec<u32>, shape: String) -> DrawCall {
    DrawCall {
      shape_name: Some(shape),
      new_shape: Some((vertices, indices)),
      draw_type: DrawType::UpdateShape,
      .. DrawCall::empty()
    }
  }
  
  pub fn with_text(mut self, text: String) -> DrawCall {
    self.text = Some(text);
    self
  }
  
  pub fn with_position(mut self, position: Vector3<f32>) -> DrawCall {
    self.position = position;
    self
  }
  
  pub fn with_2D_rotation(mut self, rotation: f32) -> DrawCall {
    self.rotation.x = rotation;
    self
  }
  
  pub fn with_rotation(mut self, rotation: Vector3<f32>) -> DrawCall {
    self.rotation = rotation;
    self
  }
  
  pub fn with_scale(mut self, scale: Vector3<f32>) -> DrawCall {
    self.scale = scale;
    self
  }
  
  pub fn with_colour(mut self, colour: Vector4<f32>) -> DrawCall {
    self.colour = colour;
    self
  }
  
  pub fn with_outline_colour(mut self, outline_colour: Vector3<f32>) -> DrawCall {
    self.text_outline_colour = outline_colour;
    self
  }
  
  pub unsafe fn set_draw_type(&mut self, forced_type: DrawType) {
    self.draw_type = forced_type;
  }
  
  pub fn with_centered_text(mut self) -> DrawCall {
    self.text_centered = true;
    self
  }
  
  pub fn without_centered_text(mut self) -> DrawCall {
    self.text_centered = false;
    self
  }
  
  pub fn as_coloured(mut self) -> DrawCall {
    self.black_white = false;
    self
  }
  
  pub fn as_black_and_white(mut self) -> DrawCall {
    self.black_white = true;
    self
  }
  
  pub fn in_black_and_white(mut self, bw_enabled: bool) -> DrawCall {
    self.black_white = bw_enabled;
    self
  }
  
  pub fn draw_type(&self) -> DrawType {
    self.draw_type.clone()
  }
  
  pub fn model_name(&self) -> Option<String> {
    let mut result = None;
    
    match self.draw_type {
      DrawType::DrawModel | DrawType::DrawInstancedModel |
      DrawType::NewModel  | DrawType::RemoveModel => {
        result = Some(self.reference_name.clone());
      },
      _ => {}
    }
    
    result
  }
  
  pub fn texture_name(&self) -> Option<String> {
    let mut result = None;
    
    match self.draw_type {
      DrawType::DrawTextured           | DrawType::DrawCustomShapeTextured | 
      DrawType::DrawInstancedTextured | DrawType::NewTexture                | 
      DrawType::RemoveTexture          |
      DrawType::DrawColoured           | DrawType::DrawCustomShapeColoured | 
      DrawType::DrawInstancedColoured
       => {
        result = Some(self.reference_name.clone());
      },
      _ => {}
    }
    
    result
  }
  
  pub fn texture_name_unref(&self) -> Option<String> {
    let mut result = None;
    
    match self.draw_type {
      DrawType::DrawTextured           | DrawType::DrawCustomShapeTextured | 
      DrawType::DrawInstancedTextured | DrawType::NewTexture                | 
      DrawType::RemoveTexture
       => {
        result = Some((*self.reference_name).to_string());
      },
      _ => {}
    }
    
    result
  }
  
  pub fn shape_name(&self) -> Option<String> {
    let mut result = None;
    
    match self.draw_type {
      DrawType::DrawCustomShapeColoured | DrawType::DrawCustomShapeTextured |
      DrawType::NewShape           | DrawType::UpdateShape        |
      DrawType::RemoveShape => {
        result = self.shape_name.clone();
      },
      _ => {}
    }
    
    result
  }
  
  pub fn instance_name(&self) -> Option<String> {
    let mut result = None;
    
    match self.draw_type {
      DrawType::DrawInstancedColoured | DrawType::DrawInstancedTextured |
      DrawType::DrawInstancedModel => {
        result = self.instanced_name.clone();
      },
      _ => {}
    }
    
    result
  }
  
  pub fn font_name(&self) -> Option<String> {
    let mut result = None;
    
    match self.draw_type {
      DrawType::DrawText | DrawType::NewText | DrawType::RemoveText => {
        result = Some(self.reference_name.clone());
      },
      _ => {}
    }
    
    result
  }
  
  pub fn display_text(&self) -> Option<String> {
    let mut result = None;
    
    match self.draw_type {
      DrawType::DrawText | DrawType::NewText | DrawType::RemoveText => {
        result = self.text.clone();
      },
      _ => {}
    }
    
    result
  }
  
  pub fn outline_colour(&self) -> Option<Vector3<f32>> {
    let mut result = None;
    
    let temp_colour = self.text_outline_colour;
    
    match self.draw_type {
      DrawType::DrawText => {
        result = Some(temp_colour);
      },
      _ => {}
    }
    
    result
  }
  
  pub fn new_shape_details(&self) -> Option<(Vec<graphics::Vertex2d>, Vec<u32>)> {
    let mut result = None;
    
    match self.draw_type {
      DrawType::NewShape | DrawType::UpdateShape => {
        result = self.new_shape.clone();
      },
      _ => {}
    }
    
    result
  }
  
  pub fn get_type(&self) -> DrawType {
    self.draw_type.clone()
  }
  
  pub fn position(&self) -> Vector3<f32> {
    self.position
  }
  
  pub fn rotation_2D(&self) -> f32 {
    self.rotation.x
  }
  
  pub fn rotation(&self) -> Vector3<f32> {
    self.rotation
  }
  
  pub fn scale(&self) -> Vector3<f32> {
    self.scale
  }
  
  pub fn colour(&self) -> Vector4<f32> {
    self.colour
  }
  
  pub fn black_and_white_enabled(&self) -> bool {
    self.black_white
  }
  
  pub fn text_centered(&self) -> bool {
    self.text_centered
  }
  
  pub fn wrap_length(&self) -> u32 {
    self.text_wrapping
  }
  
  pub fn text_outline_colour(&self) -> Vector3<f32> {
    self.text_outline_colour
  }
  
  pub fn text_edge_width(&self) -> Vector4<f32> {
    self.text_edge_width
  }
  
  pub fn is_text(&self) -> bool {
    self.draw_type == DrawType::DrawText
  }
  
  pub fn is_texture(&self) -> bool {
    (self.draw_type == DrawType::DrawTextured || self.draw_type == DrawType::DrawColoured)
  }
  
  pub fn is_instanced_texture(&self) -> bool {
    self.draw_type == DrawType::DrawInstancedTextured
  }
  
  pub fn is_model(&self) -> bool {
    self.draw_type == DrawType::DrawModel
  }
  
  pub fn is_custom_shape(&self) -> bool {
    (self.draw_type == DrawType::DrawCustomShapeColoured || self.draw_type == DrawType::DrawCustomShapeTextured)
  }
  
  pub fn is_shape_update(&self) -> bool {
    self.draw_type == DrawType::UpdateShape
  }
}

pub fn get_text_length(text: String, size: f32, font: String, fonts: GenericFont) -> f32 {
  let mut length = 0.0;
  
  for letter in text.as_bytes() {
    let c = fonts.get_character(*letter as i32);
    
    length+=c.get_advance() as f32 * (size/640.0); 
  }
  
  length
}

pub fn setup_correct_wrapping(draw: DrawCall, fonts: GenericFont) -> Vec<DrawCall> {
  let mut new_draw_calls: Vec<DrawCall> = Vec::new();
  
  let init_translation = {
    let mut temp = 0.0;
    if draw.text_centered() {
      if let Some(display_text) = draw.display_text().clone() {
        if let Some(font_name) = draw.font_name().clone() {
          temp = get_text_length(display_text, draw.scale().x, font_name, fonts.clone())*0.5;
        }
      }
    }
    draw.position().x-temp
  };
  
  let mut translation = draw.position();
  translation.x = init_translation;
  
  //let mut position = 0;
  //let mut number_of_words = 0;
  //let finish_at_next_space = 0;
  //let mut last_space_position = 0;
  
  let mut y_offset = 0.0;
  
  let size = draw.scale().x;
  
  if draw.wrap_length() <= 0 {
    if let Some(display_text) = draw.display_text() {
      for letter in display_text.as_bytes() {
        if let Some(font_name) = draw.font_name() {
          let c = fonts.get_character(*letter as i32);
          
          //let size = draw.get_x_size();
          
          if *letter != ' ' as u8 {
            new_draw_calls.push(DrawCall::draw_text_custom(Vector2::new(translation.x, translation.y+y_offset), Vector2::new(size, size), draw.colour(), draw.text_outline_colour(), draw.text_edge_width(), false, 0, (*letter as char).to_string(), font_name));
          }
          translation.x+=c.get_advance() as f32 * (size/640.0); 
        }
      }
    }
  } else {
    // for wrapping
    if let Some(display_text) = draw.display_text() {
      for letter in display_text.as_bytes() {
        if let Some(font_name) = draw.font_name() {
          let c = fonts.get_character(*letter as i32);
          
          if *letter == ' ' as u8 {
            let distance = translation.x - init_translation;
            if distance > draw.wrap_length() as f32 {
              // new line
              translation.x = init_translation;
              y_offset += (size/10.0) * -1.0;//-32.0
            
             /* let temp_diff = new_draw_calls[last_space_position-number_of_words]
              for i in last_space_position-number_of_words..position-num_of_words {
                
              }*/
            } else {
              translation.x+=c.get_advance() as f32 * (size/640.0); 
            }
            
            //last_space_position = position;
            // number_of_words += 1;
          } else {
            
            new_draw_calls.push(DrawCall::draw_text_custom(Vector2::new(translation.x, translation.y+y_offset), Vector2::new(size, size), draw.colour(), draw.text_outline_colour(), draw.text_edge_width(), false, 0, (*letter as char).to_string(), font_name));
            
            translation.x+=c.get_advance() as f32 * (size/640.0); 
          }
          //position += 1;
        }
      }
    }
  }
  
  new_draw_calls
}

pub fn calculate_text_model(translation: Vector3<f32>, size: f32, c: &GenericCharacter, letter: u8) -> Matrix4<f32> {
  let x_offset: f32 = c.get_offset_x()*size;
  let y_offset: f32 = {
    let mut temp = 0.0;
    
    if letter == '\'' as u8 ||
       letter == '\"' as u8 {
      temp = -size/16.333333333;
      //println!("The letter is '");
    }
    
    if letter == 'p' as u8 ||
       letter == 'y' as u8 ||
       letter == 'g' as u8 ||
       letter == 'j' as u8 ||
       letter == 'q' as u8 ||
       letter == '@' as u8 ||
       letter == '$' as u8 {
      temp = c.get_offset_y()*size;
    }
    
    if letter == '-' as u8 {
      temp = -c.get_offset_y()*0.5*size;
    }
    temp
  };
  
  let mut model = Matrix4::from_translation(Vector3::new(
                                translation.x + x_offset, 
                                translation.y - y_offset, 
                                translation.z));
  model = model * Matrix4::from_nonuniform_scale(size, size, 1.0);
  
  model
}

pub fn calculate_text_uv(c: &GenericCharacter) -> Vector4<f32> {
  let x: f32 = c.get_x() as f32;
  let y: f32 = c.get_y() as f32;
  let x_w: f32 = x + c.get_width() as f32;
  let y_h: f32 = y + c.get_height() as f32;
  
  Vector4::new(x, y, x_w, y_h)
}

pub fn rotate(angle: f32) -> (f32, f32) {
  let crnt_pos = Vector2::new(0.0, 1.0);
  
  let rotation_mat = Matrix2::new(
    angle.cos(), -(angle.sin()),
    angle.sin(), angle.cos()
  );
  
  let new_pos = rotation_mat * crnt_pos;
  
  (new_pos.x, new_pos.y)
}
