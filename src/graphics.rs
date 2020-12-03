use crate::drawcalls::DrawCall;
use std::collections::HashMap;
use crate::font::GenericFont;
use crate::camera::PerspectiveCamera;
use crate::graphics;
use crate::gltf_interpreter::ModelDetails;
use crate::ModelData;

use crate::cgmath::{Vector2, Vector3};

use crate::winit;

use std::vec::Vec;

#[derive(Clone, PartialEq)]
pub struct Vertex2d { pub position: [f32; 2], pub uv: [f32; 2] }

#[derive(Clone, PartialEq)]
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
  
  fn add_terrain(&mut self, model: (ModelDetails, ModelData));
  
  // Load png images
  fn preload_texture(&mut self, reference: String, location: String);
  fn add_texture(&mut self, reference: String, location: String);
  
  fn set_icon(&mut self, location: String);
  
  // Load fonts
  fn preload_font(&mut self, reference: String, font_texture: String, font: &[u8]);
  fn add_font(&mut self, reference: String, font_texture: String, font: &[u8]);  
  
  fn create_instance_text_buffer(&mut self, buffer_reference: String, texture_reference: String);
  fn create_instance_texture_buffer(&mut self, buffer_reference: String, texture_reference: String);
  fn create_instance_colour_buffer(&mut self, reference: String);
  fn create_model_instance_buffer(&mut self, reference: String);
  
  // Load custom goemetry
  fn load_static_geometry(&mut self, reference: String, verticies: Vec<graphics::Vertex2d>, indicies: Vec<u32>);
  fn load_dynamic_geometry(&mut self, reference: String, verticies: Vec<graphics::Vertex2d>, indicies: Vec<u32>);
  
  // Internal use until Custom Shaders are implemented
  fn load_shaders(&mut self);
  
  // Initalises everything
  fn init(&self);
  
  // Standard draw calls that should be called in 98% of cases
  fn pre_draw(&mut self);
  fn draw(&mut self, draw_calls: &Vec<DrawCall>, delta_time: f32);
  fn post_draw(&self);
  
  // Getters and setters
  fn get_maximum_dimensions(&self) -> Vector2<f32>;
  fn get_physical_dimensions(&self) -> Vector2<f32>;
  fn get_virtual_dimensions(&self) -> Vector2<f32>;
  
  fn force_swapchain_recreate(&mut self);
  fn retrieve_models(&mut self) -> Vec<ModelData>;
  //fn get_events(&mut self) -> Vec<winit::event::Event<()>>;
  
  fn get_mouse_position(&mut self) -> Vector2<f32>;
  fn get_fonts(&self) -> HashMap<String, GenericFont>;
  fn get_dpi_scale(&self) -> f32;
  fn is_ready(&self) -> bool;
  fn set_cursor_position(&mut self, x: f32, y: f32);
  fn show_cursor(&mut self);
  fn hide_cursor(&mut self);
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32);
  fn set_camera(&mut self, camera: PerspectiveCamera);
  fn get_camera(&self) -> PerspectiveCamera;
  fn num_drawcalls(&self) -> u32;
  
  fn force_window_resize(&mut self, new_size: Vector2<f32>, fullscreen: bool);
}

