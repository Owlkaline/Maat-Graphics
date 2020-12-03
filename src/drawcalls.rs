use crate::font::GenericCharacter;
use crate::font::GenericFont;
use crate::camera::PerspectiveCamera;
use crate::camera::PerspectiveCameraDirection;
use crate::camera::OrthoCamera;

use crate::graphics;

use crate::cgmath::{Vector2, Vector3, Vector4, Matrix4};

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
  // Ref, position, scale, rotation
  DrawModel((String, Vector3<f32>, Vector3<f32>, Vector3<f32>, bool)),
  DrawInstancedModel(String),
  // Ref, texture, position, scale, rotation
  DrawCustomShapeTextured((String, String, Vector2<f32>, Vector2<f32>, f32)),
  // Ref, position, scale, colour, rotation
  DrawCustomShapeColoured((String, Vector2<f32>, Vector2<f32>, Vector4<f32>, f32)),
  
  AddInstancedModelBuffer(String),
  // BufRef, position, scale, rotation, colour
  AddInstancedColoured((String, Vector2<f32>, Vector2<f32>, f32, Vector4<f32>)),
  // Ref, Position, Scale, Rotation, alpha
  AddInstancedTextured((String, Vector2<f32>, Vector2<f32>, f32, f32)),
  // instanced buffer Ref, Position, Scale, Rotation, colour, SpriteDetails(x,y,rows)
  AddInstancedSpriteSheet((String, Vector2<f32>, Vector2<f32>, f32, Vector4<f32>, Vector3<i32>)),
  //  reference, position, scale rotation colour hologram
  AddInstancedModel((String, Vector3<f32>, Vector3<f32>, Vector3<f32>, Vector4<f32>, bool)),
  // buffer ref
  DrawInstanced(String),
  
  // Font, Display Text, Position, Scale, Colour, Outline Colour, Edge_width, wrapped, wrap length, centered
  AddInstancedFont((String, String, Vector2<f32>, Vector2<f32>, Vector4<f32>, Vector3<f32>, Vector4<f32>, bool, u32, bool)), 
  DrawInstancedFont(String), 
  
  // Ref, location
  NewTexture((String, String)),
  NewFont, 
  NewModel,
  
  // Ref
  LoadTexture(String),
  LoadFont(String),
  LoadModel(String),
  UnloadTexture(String),
  UnloadFont(String),
  UnloadModel(String),
  
  NewShape,
  UpdateShape((String, Vec<graphics::Vertex2d>, Vec<u32>)),
  RemoveShape,
  
  NewDrawcallSet,
  DrawDrawcallSet,
  RemoveDrawcallSet,
  
  SetLight((Vector3<f32>, Vector3<f32>, f32)),
  
  SetTextureScale(f32),
  
  NewResolution(Vector2<i32>),
  NewDpi(f32),
  EnableDpi(bool),
  EnableVsync(bool),
  EnableFullscreen(bool),
  ScissorRender(Vector4<f32>),
  ResetScissorRender,
  SetCursorPosition((f32, f32)),
  
  EnableCursor(bool),
  
  // Some(OtherCamera) ,Some(x offset, y offset), Some(right and top size), velocity to lerp
  OrthoCamera((Option<OrthoCamera>, Option<Vector2<f32>>, Option<Vector2<f32>>, Vector2<f32>)),
  
  // Some(camera), Some(Direction, delta_time), Some(mouse_x_offset, moue_y_offset), Some(set_move_speed), Some(set_mouse_sensitivity)
  ModelCamera((Option<PerspectiveCamera>, Option<(PerspectiveCameraDirection, f32)>, Option<Vector2<f32>>, Option<f32>, Option<f32>)),
  
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
  pub fn draw_model(position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>, reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawModel((reference, position, scale, rotation, false)),
      coloured: true,
    }
  }
  
  pub fn draw_hologram_model(position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>, reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawModel((reference, position, scale, rotation, true)),
      coloured: true,
    }
  }
  
  pub fn draw_textured(position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, texture: String) -> DrawCall {
    let alpha = 1.0;
    DrawCall {
      draw_type: DrawType::DrawTextured((texture, position, scale, rotation, alpha)),
      coloured: true,
    }
  }
  
  pub fn add_instanced_coloured(buffer_reference: String, position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, colour: Vector4<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedColoured((buffer_reference, position, scale, rotation, colour)),
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
    debug_assert!(sprite_details.x < sprite_details.z, "Error sprite x location too large");
    debug_assert!(sprite_details.y < sprite_details.z, "Error sprite y location too large");
    debug_assert!(sprite_details.x > -1, "Error sprite x location has to be larger than -1");
    debug_assert!(sprite_details.y > -1, "Error sprite y location has to be larger than -1");
    DrawCall {
      draw_type: DrawType::AddInstancedSpriteSheet((texture, position, scale, rotation, Vector4::new(1.0, 1.0, 1.0, 1.0), sprite_details)),
      coloured: true,
    }
  }
  
  pub fn add_instanced_sprite_sheet_with_alpha(position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, texture: String, sprite_details: Vector3<i32>, alpha: f32) -> DrawCall {
    let mut draw = DrawCall::add_instanced_sprite_sheet(position, scale, rotation, texture, sprite_details);
    match &mut draw.draw_type {
      DrawType::AddInstancedSpriteSheet((_texture, _position, _scale, _rotation, colour, _sprite_details)) => {
        colour.w = alpha;
      },
      _ => {},
    }
    
    draw
  }
  
  pub fn add_instanced_sprite_sheet_coloured(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, rotation: f32, texture: String, sprite_details: Vector3<i32>) -> DrawCall {
    debug_assert!(sprite_details.x < sprite_details.z, "Error sprite x location too large");
    debug_assert!(sprite_details.y < sprite_details.z, "Error sprite y location too large");
    debug_assert!(sprite_details.x > -1, "Error sprite x location has to be larger than -1");
    debug_assert!(sprite_details.y > -1, "Error sprite y location has to be larger than -1");
    DrawCall {
      draw_type: DrawType::AddInstancedSpriteSheet((texture, position, scale, rotation, colour, sprite_details)),
      coloured: true,
    }
  }
  
  pub fn add_instanced_model(buffer_reference: String, position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedModel((buffer_reference, position, scale, rotation, Vector4::new(1.0, 1.0, 1.0, 1.0), false)),
      coloured: true,
    }
  }
  
  pub fn add_instanced_hologram_model(buffer_reference: String, position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedModel((buffer_reference, position, scale, rotation, Vector4::new(1.0, 1.0, 1.0, 1.0), true)),
      coloured: true,
    }
  }
  
  pub fn add_instanced_model_overwrite_colour(buffer_reference: String, position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>, colour: Vector3<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedModel((buffer_reference, position, scale, rotation, Vector4::new(colour.x, colour.y, colour.z, -1.0), false)),
      coloured: true,
    }
  }
  
  pub fn add_instanced_hologram_model_overwrite_colour(buffer_reference: String, position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>, colour: Vector3<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedModel((buffer_reference, position, scale, rotation, Vector4::new(colour.x, colour.y, colour.z, -1.0), true)),
      coloured: true,
    }
  }
  
  pub fn draw_instanced_model(buffer_reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawInstancedModel(buffer_reference),
      coloured: true,
    }
  }
  
  pub fn draw_instanced(buffer_reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawInstanced(buffer_reference),
      coloured: true,
    }
  }
  
  pub fn add_instanced_model_buffer(reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedModelBuffer(reference),
      coloured: false,
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
  
  pub fn draw_instanced_text(buffer_reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::DrawInstancedFont(buffer_reference),
      coloured: true,
    }
  }
  
  pub fn add_instanced_text(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedFont((font, display_text, position, scale, colour, DEFAULT_OUTLINE, DEFAULT_BASIC_EDGE_WIDTH, false, 0, false)),
      coloured: true,
    }
  }
  
  pub fn add_instanced_text_wrapped(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedFont((font, display_text, position, scale, colour, DEFAULT_OUTLINE, DEFAULT_BASIC_EDGE_WIDTH, true, wrap_length, false)),
      coloured: true,
    }
  }
  
  pub fn add_text_custom(position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, edge_width: Vector4<f32>, centered: bool, wrap_length: u32, display_text: String, font: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::AddInstancedFont((font, display_text, position, scale, colour, outline_colour, edge_width, true, wrap_length, centered)),
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
  
  pub fn set_light(position: Vector3<f32>, colour: Vector3<f32>, intensity: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::SetLight((position, colour, intensity)),
      coloured: false,
    }
  }
  
  pub fn set_camera(camera: PerspectiveCamera) -> DrawCall {
    DrawCall {
      draw_type: DrawType::ModelCamera((Some(camera), None, None, None, None)),
      coloured: false,
    }
  }
  
  pub fn set_camera_mouse_sensitivity(sensitivity: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::ModelCamera((None, None, None, None, Some(sensitivity))),
      coloured: false,
    }
  }
  
  pub fn set_camera_move_speed(speed: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::ModelCamera((None, None, None, Some(speed), None)),
      coloured: false,
    }
  }
  
  pub fn move_camera_forward(delta_time: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::ModelCamera((None, Some((PerspectiveCameraDirection::Forward, delta_time)), None, None, None)),
      coloured: false,
    }
  }
  
  pub fn move_camera_backward(delta_time: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::ModelCamera((None, Some((PerspectiveCameraDirection::Backward, delta_time)), None, None, None)),
      coloured: false,
    }
  }
  
  pub fn move_camera_left(delta_time: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::ModelCamera((None, Some((PerspectiveCameraDirection::Left, delta_time)), None, None, None)),
      coloured: false,
    }
  }
  
  pub fn move_camera_right(delta_time: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::ModelCamera((None, Some((PerspectiveCameraDirection::Right, delta_time)), None, None, None)),
      coloured: false,
    }
  }
  
  pub fn reset_ortho_camera() -> DrawCall {
    DrawCall {
      draw_type: DrawType::OrthoCamera((None, None, None, Vector2::new(0.0, 0.0))),
      coloured: false,
    }
  }
  
  pub fn lerp_ortho_camera_to_pos(position: Vector2<f32>, vel: Vector2<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::OrthoCamera((None, Some(position), None, vel)),
      coloured: false,
    }
  }
  
  pub fn lerp_ortho_camera_to_size(size: Vector2<f32>, vel: Vector2<f32>) -> DrawCall {
    DrawCall {
      draw_type: DrawType::OrthoCamera((None, None, Some(size), vel)),
      coloured: false,
    }
  }
  
  pub fn replace_ortho_camera(camera: OrthoCamera) -> DrawCall {
    DrawCall {
      draw_type: DrawType::OrthoCamera((Some(camera), None, None, Vector2::new(0.0, 0.0))),
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
  
  pub fn load_model(reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::LoadModel(reference),
      coloured: true,
    }
  }
  
  pub fn load_texture(reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::LoadTexture(reference),
      coloured: true,
    }
  }
  
  pub fn unload_model(reference: String) -> DrawCall {
    DrawCall {
      draw_type: DrawType::UnloadModel(reference),
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
  
  pub fn set_cursor_position(x: f32, y: f32) -> DrawCall {
    DrawCall {
      draw_type: DrawType::SetCursorPosition((x,y)),
      coloured: false,
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
  
  pub fn enable_cursor(enable: bool) -> DrawCall {
    DrawCall {
      draw_type: DrawType::EnableCursor(enable),
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
  
  let model = Vector4::new(translation.x + x_offset, 
                               translation.y - y_offset, 
                               size,
                               1.0);
  model
}

pub fn _calculate_text_model(translation: Vector3<f32>, size: f32, c: &GenericCharacter, letter: u8) -> Matrix4<f32> {
  let x_offset: f32 = c.get_offset_x()*size;
  let y_offset: f32 = {
    let mut temp = 0.0;
    
    if letter == '\'' as u8 ||
       letter == '\"' as u8 {
      temp = -size/16.333333333;
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
