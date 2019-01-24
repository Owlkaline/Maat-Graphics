use crate::font::GenericCharacter;
use crate::font::GenericFont;

use crate::graphics;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;

#[derive(Clone, PartialEq)]
pub enum DrawType {
  // Font, Display Text, Position, Scale, Colour, Outline Colour, Edge_width, wrapped, wrap length, centered
  DrawFont((String, String, Vector2<f32>, Vector2<f32>, Vector4<f32>, Vector3<f32>, Vector4<f32>, bool, u32, bool)), 
  // Ref, Position, Scale, Rotation
  DrawTextured((String, Vector2<f32>, Vector2<f32>, f32, f32)),
  // Ref, Position, Scale, Rotation, SpriteDetails(x,y,rows), colour
  DrawSpriteSheet((String, Vector2<f32>, Vector2<f32>, f32, Vector3<i32>, Vector4<f32>)),
  // Position, Scale, Colour, Rotation
  DrawColoured((Vector2<f32>, Vector2<f32>, Vector4<f32>, f32)),
  DrawModel,
  // Ref, texture, position, scale, rotation
  DrawCustomShapeTextured((String, String, Vector2<f32>, Vector2<f32>, f32)),
  // Ref, position, scale, colour, rotation
  DrawCustomShapeColoured((String, Vector2<f32>, Vector2<f32>, Vector4<f32>, f32)),
  
  AddInstancedColoured,
  // Ref, Position, Scale, Rotation
  AddInstancedTextured((String, Vector2<f32>, Vector2<f32>, f32, f32)),
  // Ref, Position, Scale, Rotation, SpriteDetails(x,y,rows)
  AddInstancedSpriteSheet((String, Vector2<f32>, Vector2<f32>, f32, f32, Vector3<i32>)),
  AddInstancedModel,
  DrawInstanced,
  
  // Ref, location
  NewTexture((String, String)),
  NewFont, 
  NewModel,
  
  // Ref
  LoadTexture((String)),
  LoadFont((String)),
  LoadModel,
  UnloadTexture((String)),
  UnloadFont((String)),
  UnloadModel,
  
  NewShape,
  UpdateShape((String, Vec<graphics::Vertex2d>, Vec<u32>)),
  RemoveShape,
  
  NewDrawcallSet,
  DrawDrawcallSet,
  RemoveDrawcallSet,
  
  SetTextureScale(f32),
  
  NewResolution(Vector2<i32>),
  NewDpi(f32),
  EnableDpi(bool),
  EnableVsync(bool),
  EnableFullscreen(bool),
  ScissorRender(Vector4<f32>),
  ResetScissorRender,
  
  // Some(x offset, y offset), Some(right and top size), velocity to lerp
  Camera((Option<Vector2<f32>>, Option<Vector2<f32>>, Vector2<f32>)),
  
  None,
}

const DEFAULT_OUTLINE: Vector3<f32> = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
const DEFAULT_BASIC_EDGE_WIDTH: Vector4<f32> = Vector4 { x: 0.5, y: 0.1, z: 0.1, w: 0.1 };
const DEFAULT_EDGE_WIDTH: Vector4<f32> = Vector4 { x: 0.5, y: 0.1, z: 0.7, w: 0.1 };

#[derive(Clone)]
pub struct DrawCall {
  draw_type: DrawType,
  coloured: bool,
}

impl DrawCall {
  pub fn draw_textured(position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, texture: String) -> DrawCall {
    let alpha = 1.0;
    DrawCall {
      draw_type: DrawType::DrawTextured((texture, position, scale, rotation, alpha)),
      coloured: true,
    }
  }
  
  pub fn draw_sprite_sheet(position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, texture: String, sprite_details: Vector3<i32>) -> DrawCall {
    debug_assert!(sprite_details.x < sprite_details.z, "Error sprite x location too large");
    debug_assert!(sprite_details.y < sprite_details.z, "Error sprite y location too large");
    debug_assert!(sprite_details.x > -1, "Error sprite x location has to be larger than -1");
    debug_assert!(sprite_details.y > -1, "Error sprite y location has to be larger than -1");
    DrawCall {
      draw_type: DrawType::DrawSpriteSheet((texture, position, scale, rotation, sprite_details, Vector4::new(1.0, 1.0, 1.0, 1.0))),
      coloured: true,
    }
  }
  
  pub fn draw_sprite_sheet_coloured(position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, texture: String, sprite_details: Vector3<i32>, colour: Vector4<f32>) -> DrawCall {
    debug_assert!(sprite_details.x < sprite_details.z, "Error sprite x location too large");
    debug_assert!(sprite_details.y < sprite_details.z, "Error sprite y location too large");
    debug_assert!(sprite_details.x > -1, "Error sprite x location has to be larger than -1");
    debug_assert!(sprite_details.y > -1, "Error sprite y location has to be larger than -1");
    DrawCall {
      draw_type: DrawType::DrawSpriteSheet((texture, position, scale, rotation, sprite_details, colour)),
      coloured: true,
    }
  }
  
  pub fn add_instanced_sprite_sheet(position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, texture: String, sprite_details: Vector3<i32>) -> DrawCall {
    let alpha = 1.0;
    debug_assert!(sprite_details.x < sprite_details.z, "Error sprite x location too large");
    debug_assert!(sprite_details.y < sprite_details.z, "Error sprite y location too large");
    debug_assert!(sprite_details.x > -1, "Error sprite x location has to be larger than -1");
    debug_assert!(sprite_details.y > -1, "Error sprite y location has to be larger than -1");
    DrawCall {
      draw_type: DrawType::AddInstancedSpriteSheet((texture, position, scale, rotation, alpha, sprite_details)),
      coloured: true,
    }
  }
  
  pub fn draw_instanced() -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawInstanced,
      coloured: true,
    }
  }
  
  pub fn draw_textured_with_alpha(position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, texture: String, alpha: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawTextured((texture, position, scale, rotation, alpha)),
      coloured: true,
    }
  }
  
  pub fn draw_coloured(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, rotation: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawColoured((position, scale, colour, rotation)),
      coloured: true,
    }
  }
  
  pub fn draw_custom_shape_textured(position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, texture: String, shape: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawCustomShapeTextured((shape, texture, position, scale, rotation)),
      coloured: true,
    }
  }
  
  pub fn draw_custom_shape_coloured(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, rotation: f32, shape: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawCustomShapeColoured((shape, position, scale, colour, rotation)),
      coloured: true,
    }
  }
  
  pub fn draw_text_basic(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, DEFAULT_OUTLINE, DEFAULT_BASIC_EDGE_WIDTH, false, 0, false)),
      coloured: true,
    }
  }
  
  pub fn draw_text_basic_wrapped(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, DEFAULT_OUTLINE, DEFAULT_BASIC_EDGE_WIDTH, true, wrap_length, false)),
      coloured: true,
    }
  }
  
  pub fn draw_text_basic_centered(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, DEFAULT_OUTLINE, DEFAULT_BASIC_EDGE_WIDTH, false, 0, true)),
      coloured: true,
    }
  }
  
  pub fn draw_text_basic_wrapped_centered(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, DEFAULT_OUTLINE, DEFAULT_BASIC_EDGE_WIDTH, true, wrap_length, true)),
      coloured: true,
    }
  }
  
  pub fn draw_text_outlined(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, outline_colour, DEFAULT_EDGE_WIDTH, false, 0, false)),
      coloured: true,
    }
  }
  
  pub fn draw_text_outlined_wrapped(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, outline_colour, DEFAULT_EDGE_WIDTH, true, wrap_length, false)),
      coloured: true,
    }
  }
  
  pub fn draw_text_outlined_centered(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, outline_colour, DEFAULT_EDGE_WIDTH, false, 0, true)),
      coloured: true,
    }
  }
  
  pub fn draw_text_outlined_wrapped_centered(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, outline_colour, DEFAULT_EDGE_WIDTH, true, wrap_length, true)),
      coloured: true,
    }
  }
  
  pub fn draw_text_custom(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, edge_width: Vector4<f32>, centered: bool, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawFont((font, display_text, position, scale, colour, outline_colour, edge_width, true, wrap_length, centered)),
      coloured: true,
    }
  }
  
  pub fn reset_camera() -> DrawCall {
    DrawCall {
      draw_type: DrawType::Camera((None, None, Vector2::new(0.0, 0.0))),
      coloured: false,
    }
  }
  
  pub fn lerp_camera_to_pos(position: Vector2<f32>, vel: Vector2<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::Camera((Some(position), None, vel)),
      coloured: false,
    }
  }
  
  pub fn lerp_camera_to_size(size: Vector2<f32>, vel: Vector2<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::Camera((None, Some(size), vel)),
      coloured: false,
    }
  }
  
  pub fn set_render_scissor(dim: Vector4<f32>) -> DrawCall{
    DrawCall {
      draw_type: DrawType::ScissorRender(dim),
      coloured: false,
    }
  }
  
  pub fn reset_render_scissor() -> DrawCall{
    DrawCall {
      draw_type: DrawType::ResetScissorRender,
      coloured: false,
    }
  }
  
  pub fn set_texture_scale(scale: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::SetTextureScale(scale),
      coloured: true,
    }
  }
  
  pub fn load_texture(reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::LoadTexture(reference),
      coloured: true,
    }
  }
  
  pub fn unload_texture(reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::UnloadTexture(reference),
      coloured: true,
    }
  }
  
  pub fn load_font(reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::LoadFont(reference),
      coloured: true,
    }
  }
  
  pub fn unload_font(reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::UnloadFont(reference),
      coloured: true,
    }
  }
  
  pub fn update_custom_shape(vertices: Vec<graphics::Vertex2d>, indices: Vec<u32>, shape: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::UpdateShape((shape, vertices, indices)),
      coloured: true,
    }
  }
  
  pub fn change_resolution(new_resolution: Vector2<i32>) -> DrawCall {
     DrawCall {
      draw_type: DrawType::NewResolution(new_resolution),
      coloured: false,
    }
  }
  
  pub fn change_dpi(new_dpi: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::NewDpi(new_dpi),
      coloured: false,
    }
  }
  
  pub fn enable_dpi(enable: bool) -> DrawCall {
    DrawCall {
      draw_type: DrawType::EnableDpi(enable),
      coloured: false,
    }
  }
  
  pub fn enable_vsync(enable: bool) -> DrawCall {
    DrawCall {
      draw_type: DrawType::EnableVsync(enable),
      coloured: false,
    }
  }
  
  pub fn enable_fullscreen(enable: bool) -> DrawCall {
    DrawCall {
      draw_type: DrawType::EnableFullscreen(enable),
      coloured: false,
    }
  }
  
  pub fn get_type(&self) -> DrawType {
    self.draw_type.clone()
  }
  
  pub fn model_name(&self) -> Option<String> {
    None
  }
  
  pub fn draw_textured_details(&self) -> Option<(String, Vector2<f32>, Vector2<f32>, f32, f32)> {
    let mut result = None;
    match self.draw_type {
      DrawType::DrawTextured(ref info) => {
        result = Some(info.clone());
      },
      _ => {},
    }
    result
  }
  
  pub fn draw_sprite_sheet_details(&self) -> Option<(String, Vector2<f32>, Vector2<f32>, f32, Vector3<i32>, Vector4<f32>)> {
    let mut result = None;
    match self.draw_type {
      DrawType::DrawSpriteSheet(ref info) => {
        result = Some(info.clone());
      },
      _ => {},
    }
    result
  }
  
  pub fn draw_coloured_details(&self) -> Option<(Vector2<f32>, Vector2<f32>, Vector4<f32>, f32)> {
    let mut result = None;
    match self.draw_type {
      DrawType::DrawColoured(ref info) => {
        result = Some(info.clone());
      },
      _ => {},
    }
    result
  }
  
  pub fn draw_shape_coloured_details(&self) -> Option<(String, Vector2<f32>, Vector2<f32>, Vector4<f32>, f32)> {
    let mut result = None;
    match self.draw_type {
      DrawType::DrawCustomShapeColoured(ref info) => {
        result = Some(info.clone());
      },
      _ => {},
    }
    result
  }
  
  pub fn draw_shape_textured_details(&self) -> Option<(String, String, Vector2<f32>, Vector2<f32>, f32)> {
    let mut result = None;
    match self.draw_type {
      DrawType::DrawCustomShapeTextured(ref info) => {
        result = Some(info.clone());
      },
      _ => {},
    }
    result
  }
  
  pub fn draw_font_details(&self) -> Option<(String, String, Vector2<f32>, Vector2<f32>, Vector4<f32>, Vector3<f32>, Vector4<f32>, bool, u32, bool)> {
    let mut result = None;
    match self.draw_type {
      DrawType::DrawFont(ref info) => {
        result = Some(info.clone());
      },
      _ => {},
    }
    result
  }
  
  pub fn in_black_and_white(mut self) -> DrawCall {
    self.coloured = false;
    self
  }
  
  pub fn is_black_and_white(&self) -> bool {
    !self.coloured
  }
}

pub fn get_text_length(text: String, size: f32, fonts: GenericFont) -> f32 {
  let mut length = 0.0;
  
  for letter in text.as_bytes() {
    let c = fonts.get_character(*letter as i32);
    
    length+=c.get_advance() as f32 * (size/640.0); 
  }
  
  length
}

pub fn setup_correct_wrapping(display_text: String, font: String, position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>,edge_width: Vector4<f32>, wrap_length: u32, centered: bool, fonts: GenericFont) -> Vec<DrawCall> {
  let mut new_draw_calls: Vec<DrawCall> = Vec::new();
  
  let init_translation = {
    let mut temp = 0.0;
    if centered {
      temp = get_text_length(display_text.clone(), scale.x, fonts.clone())*0.5;
    }
    position.x-temp
  };
  
  let mut translation = position;
  translation.x = init_translation;
  
  //let mut position = 0;
  //let mut number_of_words = 0;
  //let finish_at_next_space = 0;
  //let mut last_space_position = 0;
  
  let mut y_offset = 0.0;
  
  let size = scale.x;
  
  if wrap_length <= 0 {
      for letter in display_text.clone().as_bytes() {
          let c = fonts.get_character(*letter as i32);
          
          //let size = draw.get_x_size();
          
          if *letter != ' ' as u8 {
            new_draw_calls.push(DrawCall::draw_text_custom(Vector2::new(translation.x, translation.y+y_offset), Vector2::new(size, size), colour, outline_colour, edge_width, false, 0, (*letter as char).to_string(), font.clone()));
          }
          translation.x+=c.get_advance() as f32 * (size/640.0); 
        }
  } else {
    // for wrapping
      for letter in display_text.clone().as_bytes() {
          let c = fonts.get_character(*letter as i32);
          
          if *letter == ' ' as u8 {
            let distance = translation.x - init_translation;
            if distance > wrap_length as f32 {
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
            
            new_draw_calls.push(DrawCall::draw_text_custom(Vector2::new(translation.x, translation.y+y_offset), Vector2::new(size, size), colour, outline_colour, edge_width, false, 0, (*letter as char).to_string(), font.clone()));
            
            translation.x+=c.get_advance() as f32 * (size/640.0); 
          }
          //position += 1;
      }
  }
  
  new_draw_calls
}

pub fn calculate_text_info(translation: Vector3<f32>, size: f32, c: &GenericCharacter, letter: u8) -> Vector4<f32> {
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
      temp = c.get_offset_y()*0.5*size;
    }
    
    if letter == '-' as u8 {
      temp = -c.get_offset_y()*0.5*size;
    }
    temp
  };
  
  let mut model = Vector4::new(translation.x + x_offset, 
                               translation.y - y_offset, 
                               size,
                               1.0);
  model
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
      temp = c.get_offset_y()*0.5*size;
    }
    
    if letter == '-' as u8 {
      temp = -c.get_offset_y()*0.5*size;
    }
    temp
  };
  //let axis_z = Vector3::new(0.0, 0.0, 1.0).normalize();
  //let rotation: Matrix4<f32> = Matrix4::from_axis_angle(axis_z, Deg(450.0-rotation));
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
