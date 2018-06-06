use window::GlWindow;
use drawcalls;
use drawcalls::DrawCall;
use math;
use shaders::traits::ShaderFunctions;
use shaders::ShaderTexture;
use shaders::ShaderTextureInstanced;
use shaders::ShaderText;
use shaders::ShaderBloom;
use shaders::ShaderBlur;
use shaders::ShaderFinal;
use shaders::Shader3D;
use shaders::traits::Fbo;
use graphics;
use graphics::CoreRender;
use settings::Settings;
use font::GenericFont;
use camera::Camera;
use opengex_parser::OpengexPaser;

use cgmath;
use cgmath::Deg;
use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix3;
use cgmath::Matrix4;
use cgmath::InnerSpace;

use image;
use winit;

use gl;
use gl::types::*;

use std::env;
use std::ptr;
use std::mem;
use std::cmp;
use std::time;
use std::ffi::CStr;
use std::f32::consts;
use std::ffi::CString;
use std::os::raw::c_void;
use std::collections::HashMap;

pub const BLUR_DIM: u32 = 512;

pub const TEXTURE: usize = 0;
pub const TEXT: usize = 1;
pub const INSTANCED: usize = 2;
pub const BLOOM: usize = 3;
pub const BLUR: usize = 4;
pub const FINAL: usize = 5;

pub const MODEL: usize = 0;

pub const INSTANCE_DATA_LENGTH: usize = 22;

pub struct InstancedVao {
  vao: GLuint,
  vbo: [GLuint; 2],
  ebo: GLuint,
  num_vertices: GLint,
  num_indices: GLint,
  attrib: Vec<GLuint>,
  vbo_data: Vec<GLfloat>,
  max_instances: GLuint,
}

impl InstancedVao {
  pub fn new(max_instances: i32) -> InstancedVao {
    let mut vao: GLuint = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
    }
    
    InstancedVao {
      vao: vao,
      vbo: [0, 0],
      ebo: 0,
      num_vertices: 0,
      num_indices: 0,
      attrib: Vec::new(),
      vbo_data: Vec::new(),
      max_instances: max_instances as GLuint,
    }
  }
  
  pub fn cleanup(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &mut self.vbo[0]);
      gl::DeleteBuffers(1, &mut self.vbo[1]);
      gl::DeleteBuffers(1, &mut self.ebo);
      gl::DeleteVertexArrays(1, &mut self.vao);
      gl::GenVertexArrays(1, &mut self.vao);
    }
    
    self.attrib.clear();
    self.num_vertices = 0;
    self.num_indices = 0;
  }
  
  pub fn update_vbodata(&self, size: usize, new_data: Vec<GLfloat>) {
    //self.vbo_data = new_data;
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo[1]);
      gl::BufferSubData(gl::ARRAY_BUFFER, 0, (mem::size_of::<GLfloat>()*size*INSTANCE_DATA_LENGTH) as isize, mem::transmute(&new_data[0]));
    }
  }
  
  pub fn draw_indexed_instanced(&self, size: usize, draw_type: GLuint) {
    self.bind();
    self.bind_ebo();
    unsafe {
      gl::DrawElementsInstanced(draw_type, self.num_indices, gl::UNSIGNED_INT, ptr::null(), size as GLint);
    }
    self.unbind();
  }
  
  pub fn unbind(&self) {
    unsafe {
      for location in self.attrib.clone() {
        gl::DisableVertexAttribArray(location);
      }
      gl::BindVertexArray(0);
    }
  }
  
  pub fn bind(&self) {
    unsafe {
      gl::BindVertexArray(self.vao);
      for location in self.attrib.clone() {
        gl::EnableVertexAttribArray(location);
      }
    }
  }
  
  pub fn bind_ebo(&self) {
    unsafe {
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
    }
  }
  
  pub fn create_vbo(&mut self, vertices: Vec<GLfloat>, draw_type: GLuint) {
    let mut vbo: GLuint = 0;
    unsafe {
      gl::GenBuffers(1, &mut vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::BufferData(gl::ARRAY_BUFFER,
                     (mem::size_of::<GLuint>()*vertices.len()) as isize,
                     mem::transmute(&vertices[0]),
                     draw_type);
      self.set_vertex_attrib(0, 2, 4, 0);
      self.set_vertex_attrib(1, 2, 4, 2);
      self.num_vertices = (vertices.len()/3) as GLint;
    }
    
    let mut vbo_data: GLuint = 0;
    unsafe {
      gl::GenBuffers(1, &mut vbo_data);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo_data);
      gl::BufferData(gl::ARRAY_BUFFER,
                     (mem::size_of::<GLfloat>()*self.max_instances as usize*INSTANCE_DATA_LENGTH as usize) as isize,
                     ptr::null(),
                     draw_type);
    }
    
    self.set_vertex_instanced_attrib(2, 4, INSTANCE_DATA_LENGTH, 0);
    self.set_vertex_instanced_attrib(3, 4, INSTANCE_DATA_LENGTH, 4);
    self.set_vertex_instanced_attrib(4, 4, INSTANCE_DATA_LENGTH, 8);
    self.set_vertex_instanced_attrib(5, 4, INSTANCE_DATA_LENGTH, 12);
    self.set_vertex_instanced_attrib(6, 4, INSTANCE_DATA_LENGTH, 16);
    self.set_vertex_instanced_attrib(7, 2, INSTANCE_DATA_LENGTH, 20);
    
    self.vbo = [vbo, vbo_data];
  }
  
  pub fn create_ebo(&mut self, indices: Vec<GLuint>, draw_type: GLuint) {
    unsafe {
      gl::GenBuffers(1, &mut self.ebo);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
      gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                     (mem::size_of::<GLuint>()*indices.len()) as isize,
                     mem::transmute(&indices[0]),
                     draw_type);
      self.num_indices = indices.len() as GLint;
    }
  }
  
  pub fn set_vertex_attrib(&mut self, location: GLuint, size: GLint, total_size: usize, offset: usize) {
    unsafe {
      gl::VertexAttribPointer(location, size, gl::FLOAT, gl::FALSE, 
                              (total_size * mem::size_of::<GLfloat>()) as i32,
                              ptr::null().offset((offset * mem::size_of::<GLfloat>()) as isize));
      self.attrib.push(location);
    }
  }
  
  pub fn set_vertex_instanced_attrib(&mut self, location: GLuint, size: GLint, total_size: usize, offset: usize) {
    unsafe {
      gl::VertexAttribPointer(location, size, gl::FLOAT, gl::FALSE, 
                              (total_size * mem::size_of::<GLfloat>()) as i32,
                              ptr::null().offset((offset * mem::size_of::<GLfloat>()) as isize));
      gl::VertexAttribDivisor(location, 1);
      self.attrib.push(location);
    }
  }
  
  pub fn activate_texture(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE0 + i);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
  
  pub fn activate_texture1(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE1);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
}

pub struct Vao {
  vao: GLuint,
  vbo: GLuint,
  ebo: GLuint,
  num_vertices: GLint,
  num_indices: GLint,
  attrib: Vec<GLuint>,
}

impl Vao {
  pub fn new() -> Vao {
    let mut vao: GLuint = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
    }
    
    Vao {
      vao: vao,
      vbo: 0,
      ebo: 0,
      num_vertices: 0,
      num_indices: 0,
      attrib: Vec::new(),
    }
  }
  
  pub fn cleanup(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &mut self.vbo);
      gl::DeleteBuffers(1, &mut self.ebo);
      gl::DeleteVertexArrays(1, &mut self.vao);
      gl::GenVertexArrays(1, &mut self.vao);
    }
    
    self.attrib.clear();
    self.num_vertices = 0;
    self.num_indices = 0;
  }
  
  pub fn draw_indexed(&self, draw_type: GLuint) {
    self.bind();
    self.bind_ebo();
    unsafe {
      gl::DrawElements(draw_type, self.num_indices, gl::UNSIGNED_INT, ptr::null());
    }
  }
  
  pub fn draw(&self, draw_type: GLuint) {
    self.bind();
    //self.bind_ebo();
    unsafe {
      gl::DrawElements(draw_type, self.num_vertices, gl::UNSIGNED_INT, ptr::null());
    }
  }
  
  pub fn unbind(&self) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
      gl::BindVertexArray(0);
    }
  }
  
  pub fn bind(&self) {
    unsafe {
      gl::BindVertexArray(self.vao);
      for location in self.attrib.clone() {
        gl::EnableVertexAttribArray(location);
      }
    }
  }
  
  pub fn bind_ebo(&self) {
    unsafe {
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
    }
  }
  
  pub fn create_vbo(&mut self, vertices: Vec<GLfloat>, draw_type: GLuint) {
    let mut vbo: GLuint = 0;
    unsafe {
      gl::GenBuffers(1, &mut vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::BufferData(gl::ARRAY_BUFFER,
                     (mem::size_of::<GLuint>()*vertices.len()) as isize,
                     mem::transmute(&vertices[0]),
                     draw_type);
      self.num_vertices = (vertices.len()/3) as GLint;
    }
    self.vbo = vbo;
  }
  
  pub fn create_ebo(&mut self, indices: Vec<GLuint>, draw_type: GLuint) {
    unsafe {
      gl::GenBuffers(1, &mut self.ebo);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
      gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                     (mem::size_of::<GLuint>()*indices.len()) as isize,
                     mem::transmute(&indices[0]),
                     draw_type);
      self.num_indices = indices.len() as GLint;
    }
  }
  
  pub fn set_vertex_attrib(&mut self, location: GLuint, size: GLint, total_size: usize, offset: usize) {
    unsafe {
      gl::VertexAttribPointer(location, size, gl::FLOAT, gl::FALSE, 
                              (total_size * mem::size_of::<GLfloat>()) as i32,
                              ptr::null().offset((offset * mem::size_of::<GLfloat>()) as isize));
      self.attrib.push(location);
    }
  }
  
  pub fn activate_texture(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE0 + i);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
  
  pub fn activate_texture1(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE1);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
}

pub struct GL2D {
  shaders: Vec<Box<ShaderFunctions>>,
  vao: Vao,
  projection: Matrix4<f32>,
  custom_vao: HashMap<String, Vao>,
  instanced_vao: HashMap<String, InstancedVao>,
}

pub struct GL3D {
  camera: Camera,
  
  shaders: Vec<Box<ShaderFunctions>>,
  models: HashMap<String, Vao>,
  projection: Matrix4<f32>,
}

#[derive(Clone)]
pub struct ModelInfo {
  location: String,
  texture: String,
}

pub struct RawGl {
  ready: bool,
  shader_id: Vec<GLuint>,
  fonts: HashMap<String, GenericFont>,
  textures: HashMap<String, GLuint>,
  texture_paths: HashMap<String, String>,
  model_paths: HashMap<String, ModelInfo>,
  
  clear_colour: Vector4<f32>,
  
  gl2D: GL2D,
  gl3D: GL3D,
  framebuffer: Fbo,
  framebuffer_bloom: Fbo,
  framebuffer_blur_ping: Fbo,
  framebuffer_blur_pong: Fbo,
  
  view: Matrix4<f32>,
  scale: Matrix4<f32>,
  
  min_dimensions: [u32; 2],
  pub window: GlWindow,
}

extern "system" fn opengl_debug(source: GLenum, _type: GLenum, id: GLuint, severity: GLenum, 
                    length: GLsizei, messages: *const GLchar, user: *mut c_void) {
  unsafe {
    println!("Source: {}, type: {}, id: {}, severity: {}, Message: {:?}", source, _type, id, severity,  CStr::from_ptr(messages));
  }
}

impl RawGl {
  pub fn new() -> RawGl {
    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    //avoid_winit_wayland_hack
    println!("Forcing x11");
    env::set_var("WINIT_UNIX_BACKEND", "x11");
    
    let mut settings = Settings::load();
    let width = settings.get_resolution()[0];
    let height = settings.get_resolution()[1];
    let min_width = settings.get_minimum_resolution()[0];
    let min_height = settings.get_minimum_resolution()[1];
    let fullscreen = settings.is_fullscreen();
    let mut msaa_samples = settings.get_msaa();
    let vsync = settings.vsync_enabled();
    
    let window = GlWindow::new(width, height, min_width, min_height, fullscreen, vsync);
    
    let proj_2d = RawGl::load_2d_projection(width as f32, height as f32);
    let proj_3d = RawGl::load_3d_projection(width as f32, height as f32);
    
    let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0.0, 0.0, -1.0), cgmath::Point3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, -1.0, 0.0));
    let scale = cgmath::Matrix4::from_scale(0.1);
    
    if msaa_samples <= 0 {
      msaa_samples = 1;
    }
    
    let mut max_samples: GLint = 1;
    unsafe {
      gl::GetIntegerv(gl::MAX_FRAMEBUFFER_SAMPLES, &mut max_samples);
    }
    
    println!("Max MSAA: x{}", max_samples);
    let msaa_samples: i32 = cmp::min(msaa_samples, max_samples as u32) as i32;
    println!("Current MSAA: x{}\n", msaa_samples);
    
    unsafe {
      gl::Enable(gl::DEBUG_OUTPUT);
      gl::DebugMessageCallback(opengl_debug, 0 as *const c_void);
    }
    
    RawGl {
      ready: false,
      shader_id: Vec::with_capacity(3),
      fonts: HashMap::with_capacity(1),
      textures: HashMap::with_capacity(40),
      texture_paths: HashMap::with_capacity(40),
      model_paths: HashMap::with_capacity(10),

      clear_colour: Vector4::new(0.0, 0.0, 0.0, 1.0),

      gl2D: GL2D {
        shaders: Vec::with_capacity(3),
        vao: Vao::new(),
        projection: proj_2d,
        custom_vao: HashMap::new(),
        instanced_vao: HashMap::new(),
      },
      
      gl3D: GL3D {
        camera: Camera::default_gl(),
        
        shaders: Vec::new(),
        models: HashMap::new(),
        projection: proj_3d,
      },
      framebuffer: Fbo::new(msaa_samples, width as i32, height as i32),
      framebuffer_bloom: Fbo::new(1, width as i32, height as i32),
      framebuffer_blur_ping: Fbo::new(1, BLUR_DIM as i32, BLUR_DIM as i32),
      framebuffer_blur_pong: Fbo::new(1, BLUR_DIM as i32, BLUR_DIM as i32),
      
      view: view,
      scale: scale,
      
      min_dimensions: [min_width, min_height],
      window: window,
    }
  }
  
  pub fn set_viewport(&self, width: u32, height: u32) {
    unsafe {
      gl::Viewport(0, 0, (width as f32 * self.get_dpi_scale()) as i32, (height as f32 * self.get_dpi_scale()) as i32);
    }
  }
  
  pub fn with_title(mut self, title: String) -> RawGl {
    self.window.set_title(title);
    self
  }
  
  pub fn load_2d_projection(width: f32, height: f32) -> Matrix4<f32> {
    cgmath::ortho(0.0, width, 0.0, height, -1.0, 1.0)
  }
  
  pub fn load_3d_projection(width: f32, height: f32) -> Matrix4<f32> {
    cgmath::perspective(cgmath::Deg(45.0), { width as f32 / height as f32 }, 0.01, 100.0) * cgmath::Matrix4::from_scale(-1.0)
  }
  
  fn load_2d_vao(&mut self) {
    let square_verts: Vec<GLfloat> = vec!(
       0.5,  0.5, 1.0, 1.0, // top right
       0.5, -0.5, 1.0, 0.0, // bottom right
      -0.5, -0.5, 0.0, 0.0, // bottom left
      -0.5,  0.5, 0.0, 1.0, // top left 
    );
  
    let square_indices: Vec<GLuint> = vec!(  // note that we start from 0!
      0, 1, 3,   // first triangle
      1, 2, 3    // second triangle
    );
    
    self.gl2D.vao.bind();
    self.gl2D.vao.create_ebo(square_indices, gl::STATIC_DRAW);
    self.gl2D.vao.create_vbo(square_verts, gl::STATIC_DRAW);
    
    self.gl2D.vao.set_vertex_attrib(0, 2, 4, 0);
    self.gl2D.vao.set_vertex_attrib(1, 2, 4, 2);
  }
  
  fn load_2d_instanced_vao(&mut self, reference: String, max_instances: i32) {
    let square_verts: Vec<GLfloat> = vec!(
       0.5,  0.5, 1.0, 1.0, // top right
       0.5, -0.5, 1.0, 0.0, // bottom right
      -0.5, -0.5, 0.0, 0.0, // bottom left
      -0.5,  0.5, 0.0, 1.0, // top left 
    );
  
    let square_indices: Vec<GLuint> = vec!(  // note that we start from 0!
      0, 1, 3,   // first triangle
      1, 2, 3    // second triangle
    );
    
    let mut vao = InstancedVao::new(max_instances);
    vao.bind();
    vao.create_ebo(square_indices, gl::STATIC_DRAW);
    vao.create_vbo(square_verts, gl::STATIC_DRAW);
    
    vao.unbind();
    
    self.gl2D.instanced_vao.insert(reference, vao);
  }
  
  fn load_custom_2d_vao(&mut self, reference: String, verts: Vec<GLfloat>, indicies: Vec<GLuint>, is_dynamic: bool) {
    let mut vao = Vao::new();
    vao.bind();
    
    if is_dynamic {
      println!("is dynamic");
      vao.create_ebo(indicies, gl::STREAM_DRAW);
      vao.create_vbo(verts, gl::STREAM_DRAW);
    } else {
      vao.create_ebo(indicies, gl::STATIC_DRAW);
      vao.create_vbo(verts, gl::STATIC_DRAW);
    }
    
    vao.set_vertex_attrib(0, 2, 4, 0);
    vao.set_vertex_attrib(1, 2, 4, 2);
    
    self.gl2D.custom_vao.insert(reference, vao);
  }
  
  fn update_vao(&mut self, draw: &DrawCall) {
    let mut verts: Vec<GLfloat> = Vec::new();
    for v in draw.get_new_vertices() {
      verts.push(v.position[0] as GLfloat);
      verts.push(v.position[1] as GLfloat);
      verts.push(v.uv[0] as GLfloat);
      verts.push(v.uv[1] as GLfloat);
    };
    
    let index = draw.get_new_indices().iter().map(|i| {
      *i as GLuint
    }).collect::<Vec<GLuint>>();
    
    let reference = draw.get_text().clone();
    
    if let Some(custom_vao) = self.gl2D.custom_vao.get_mut(&reference) {
      custom_vao.cleanup();
      custom_vao.bind();
      custom_vao.create_vbo(verts, gl::STREAM_DRAW);
      custom_vao.create_ebo(index, gl::STREAM_DRAW);
      custom_vao.set_vertex_attrib(0, 2, 4, 0);
      custom_vao.set_vertex_attrib(1, 2, 4, 2);
    } else {
      println!("Error: custom vao doesnt exist: {:?}", reference);
    }
  }
  
  fn draw_3d(&mut self, draw: &DrawCall) {
    unsafe {
      gl::Enable(gl::DEPTH_TEST);
      gl::DepthFunc(gl::LESS);
    }
    
    if self.gl3D.models.contains_key(draw.get_texture()) {
      let model = self.gl3D.models.get(draw.get_texture()).expect("Invalid model name").clone();
      
      let rotation_x: Matrix4<f32> = Matrix4::from_angle_x(Deg(draw.get_x_rotation()));
      let rotation_y: Matrix4<f32> = Matrix4::from_angle_y(Deg(draw.get_y_rotation()));
      let rotation_z: Matrix4<f32> = Matrix4::from_angle_z(Deg(draw.get_z_rotation()));
        
      let transformation: Matrix4<f32> = (cgmath::Matrix4::from_translation(draw.get_translation()) * cgmath::Matrix4::from_scale(draw.get_size().x)) * (rotation_x*rotation_y*rotation_z);
      
      let lighting_position: Matrix4<f32> =
        Matrix4::from_cols(
          Vector4::new(0.0, -0.6, 25.0, -1.0),
          Vector4::new(7.0, -0.6, 25.0, -1.0),
          Vector4::new(-2000000.0, 1000000.0, -2000000.0, -1.0),
          Vector4::new(0.0, 0.0, 0.0, -1.0)
        );
      
      let reflectivity = 1.0;
      let shine_damper = 10.0;
      
      let lighting_colour: Matrix4<f32> =
        // (R, G, B, n/a)
        Matrix4::from_cols(
          Vector4::new(0.0, 0.0, 1.0, -1.0), // colour + shinedamper
          Vector4::new(1.0, 0.0, 0.0, -1.0),  // colour + reflectivity
          Vector4::new(0.4, 0.4, 0.4, -1.0), //sun
          Vector4::new(0.0, 0.0, 0.0, -1.0)
        );
      
      // (Intensity, 1)
      let attenuation: Matrix4<f32> =
        Matrix4::from_cols(
          Vector4::new(0.1, 0.25, 0.25, -1.0),
          Vector4::new(0.1, 0.25, 0.25, -1.0),
          Vector4::new(0.5, 0.0, 0.0, -1.0),
          Vector4::new(0.0, 0.0, 0.0, -1.0)
        );
      
      let view = self.gl3D.camera.get_view_matrix();/*self.scale;*///self.view Matrix4::from_angle_y(Deg(180.0))  self.scale;
      
      let mut texture = String::from(graphics::DEFAULT_TEXTURE);
      if self.textures.contains_key(draw.get_texture()) {
        texture = draw.get_texture().clone();
      }
      let texture = *self.textures.get(&texture).unwrap();
      
      self.gl3D.shaders[MODEL].Use();
      self.gl3D.shaders[MODEL].set_mat4(String::from("transformation"), transformation);
      self.gl3D.shaders[MODEL].set_mat4(String::from("view"), view);
      self.gl3D.shaders[MODEL].set_mat4(String::from("projection"), self.gl3D.projection);
      self.gl3D.shaders[MODEL].set_mat4(String::from("lightpositions"), lighting_position);
      self.gl3D.shaders[MODEL].set_mat4(String::from("lightcolours"), lighting_colour);
      self.gl3D.shaders[MODEL].set_mat4(String::from("attenuations"), attenuation);
      self.gl3D.shaders[MODEL].set_float(String::from("shine_damper"), 10.0);
      self.gl3D.shaders[MODEL].set_float(String::from("reflectivity"), 1.0);
      
      model.activate_texture(0, texture);
      model.draw_indexed(gl::TRIANGLES);
    } else {
      println!("Error: 3D model not found: {:?}", draw.get_texture());
    }
    
    unsafe {
      gl::Disable(gl::DEPTH_TEST);
    }
  }
  
  fn draw_instanced(&mut self, draw_calls: Vec<DrawCall>, offset: usize) -> usize {
    let mut num_instances = 0;
    
    if !self.gl2D.instanced_vao.contains_key(&draw_calls[offset].get_instance_reference()) {
      println!("Error: Instanced vao not found: {:?}", draw_calls[offset].get_instance_reference());
      return 0;
    }
    
    let texture = draw_calls[offset].get_texture();
    
    for i in offset..draw_calls.len() {
      if draw_calls[i].is_instanced() && draw_calls[i].get_texture() == texture {
        num_instances += 1;
      } else {
        break;
      }
    }
    
    let mut new_data: Vec<GLfloat> = Vec::new();
        
    let has_texture = {
      let mut value = 1.0;
      if draw_calls[offset].get_texture() == &String::from("") {
        value = 0.0;
      }
      value
    };
    
    for i in offset..offset+num_instances {
      let draw = draw_calls[i].clone();
      
      let colour: [f32; 4] = draw.get_colour().into();
      let mut bw: f32 = 0.0;
      if draw.is_back_and_white() {
        bw = 1.0;
      }
      
      let model = math::calculate_texture_model(draw.get_translation(), draw.get_size(), -(draw.get_x_rotation()));
      let model: [[f32; 4]; 4] = model.into();
      
      for row in model.iter() {
        for element in row {
          new_data.push(*element)
        }
      }
      
      for value in colour.iter() {
        new_data.push(*value)
      }
      new_data.push(has_texture);
      new_data.push(bw);
    }
   // println!("num: {}, len: {}", num_instances, new_data.len());
    
    let draw = draw_calls[offset].clone();
    if has_texture == 1.0 {
      //println!("{}", draw.get_texture());
      self.gl2D.instanced_vao[&draw.get_instance_reference()].activate_texture(0, *self.textures.get(draw.get_texture()).expect("Texture not found!"));
    }
    
    self.gl2D.shaders[INSTANCED].Use();
    self.gl2D.instanced_vao[&draw.get_instance_reference()].bind();
    self.gl2D.instanced_vao[&draw.get_instance_reference()].update_vbodata(num_instances, new_data);
    self.gl2D.instanced_vao[&draw.get_instance_reference()].draw_indexed_instanced(num_instances, gl::TRIANGLES);
    self.gl2D.instanced_vao[&draw.get_instance_reference()].unbind();
    
    num_instances
  }
  
  fn draw_square(&self, draw: &DrawCall) {
    let colour = draw.get_colour();
    let has_texture = {
      let mut value = 1.0;
      if draw.get_texture() == &String::from("") {
        value = 0.0;
      }
      value
    };
    
    let mut is_blackwhite = 0.0;
    if draw.is_back_and_white() {
      is_blackwhite = 1.0;
    }
    let textured_blackwhite = Vector2::new(has_texture, is_blackwhite);
    
    let model = math::calculate_texture_model(draw.get_translation(), draw.get_size(), -(draw.get_x_rotation()));
    
    self.gl2D.shaders[TEXTURE].Use();
    self.gl2D.shaders[TEXTURE].set_mat4(String::from("model"), model);
    self.gl2D.shaders[TEXTURE].set_vec4(String::from("new_colour"), colour);
    self.gl2D.shaders[TEXTURE].set_vec2(String::from("textured_blackwhite"), textured_blackwhite);
    if has_texture == 1.0 {
      if self.textures.contains_key(draw.get_texture()) {
        self.gl2D.vao.activate_texture(0, *self.textures.get(draw.get_texture()).unwrap());
      } else {
        println!("Error: Texture not found: {:?}", draw.get_texture());
      }
    }
    
    if draw.is_custom_vao() {
      self.gl2D.custom_vao.get(draw.get_text()).unwrap().draw_indexed(gl::TRIANGLES);
    } else {
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
    }
  }
  
  fn draw_text(&self, draw: &DrawCall) {
    if !self.textures.contains_key(draw.get_texture()) {
      println!("Error: Font texture not found: {:?}", draw.get_texture());
      return;
    }
    
    let mut translation = draw.get_translation();
    
    let wrapped_draw = drawcalls::setup_correct_wrapping(draw.clone(), self.fonts.clone());
    let size = draw.get_x_size();
    
    for letter in wrapped_draw {
      let char_letter = {
        letter.get_text().as_bytes()[0] 
      };
      
      let c = self.fonts.get(draw.get_texture()).unwrap().get_character(char_letter as i32);
      
      let model = drawcalls::calculate_text_model(letter.get_translation(), size, &c.clone(), char_letter);
      let letter_uv = drawcalls::calculate_text_uv(&c.clone());
      let colour = letter.get_colour();
      let outline = letter.get_outline_colour();
      let edge_width = letter.get_edge_width(); 
      
      self.gl2D.shaders[TEXT].Use();
      self.gl2D.shaders[TEXT].set_mat4(String::from("model"), model);
      self.gl2D.shaders[TEXT].set_vec4(String::from("colour"), colour);
      self.gl2D.shaders[TEXT].set_vec4(String::from("letter_uv"), letter_uv);
      self.gl2D.shaders[TEXT].set_vec3(String::from("outlineColour"), outline);
      self.gl2D.shaders[TEXT].set_vec4(String::from("edge_width"), edge_width);
      
      self.gl2D.vao.activate_texture(0, *self.textures.get(draw.get_texture()).unwrap());
      
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
      
      translation.x+=c.get_advance() as f32 * (size/640.0); 
    }
  }
  
  fn draw_framebuffer(&self, draw: DrawCall, texture: GLuint) {
    let colour = Vector4::new(1.0, 0.0, 0.0, 1.0);//draw.get_colour();
    let has_texture = 1.0;
    let mut is_blackwhite = 0.0;
    if draw.is_back_and_white() {
      is_blackwhite = 1.0;
    }
    let textured_blackwhite = Vector2::new(has_texture, is_blackwhite);
    
    let model = math::calculate_texture_model(draw.get_translation(), draw.get_size(), -(draw.get_x_rotation()));
    
    self.gl2D.shaders[TEXTURE].Use();
    self.gl2D.shaders[TEXTURE].set_mat4(String::from("model"), model);
    self.gl2D.shaders[TEXTURE].set_vec4(String::from("new_colour"), colour);
    self.gl2D.shaders[TEXTURE].set_vec2(String::from("textured_blackwhite"), textured_blackwhite);
    
    self.gl2D.vao.activate_texture(0, texture);
    
    if draw.is_custom_vao() {
      self.gl2D.custom_vao.get(draw.get_text()).unwrap().draw_indexed(gl::TRIANGLES);
    } else {
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
    }
  }
  
  fn draw_bloom(&self, draw: DrawCall, texture: GLuint) {
    let model = math::calculate_texture_model(draw.get_translation(), draw.get_size(), -(draw.get_x_rotation()));
    
    self.gl2D.shaders[BLOOM].Use();
    self.gl2D.shaders[BLOOM].set_mat4(String::from("model"), model);
    
    self.gl2D.vao.activate_texture(0, texture);
    
    if draw.is_custom_vao() {
      self.gl2D.custom_vao.get(draw.get_text()).unwrap().draw_indexed(gl::TRIANGLES);
    } else {
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
    }
  }
  
  fn draw_blur(&self, draw: DrawCall, texture: GLuint, direction: Vector2<f32>) {
    let model = math::calculate_texture_model(draw.get_translation(), draw.get_size(), -(draw.get_x_rotation()));
    
    self.gl2D.shaders[BLUR].Use();
    self.gl2D.shaders[BLUR].set_mat4(String::from("model"), model);
    self.gl2D.shaders[BLUR].set_vec2(String::from("direction"), direction);
    
    self.gl2D.vao.activate_texture(0, texture);
    
    if draw.is_custom_vao() {
      self.gl2D.custom_vao.get(draw.get_text()).unwrap().draw_indexed(gl::TRIANGLES);
    } else {
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
    }
  }
  
  fn draw_final_frame(&self, draw: DrawCall, base_texture: GLuint, bloom_texture: GLuint, bloom: bool) {
    let model = math::calculate_texture_model(draw.get_translation(), draw.get_size(), -(draw.get_x_rotation()));
    
    self.gl2D.shaders[FINAL].Use();
    self.gl2D.shaders[FINAL].set_mat4(String::from("model"), model);
    if bloom {
      self.gl2D.shaders[FINAL].set_float(String::from("bloom_enabled"), 2.0);
    } else {
      self.gl2D.shaders[FINAL].set_float(String::from("bloom_enabled"), 0.0);
    }
    
    self.gl2D.vao.activate_texture(0, base_texture);
    self.gl2D.vao.activate_texture(1, bloom_texture);
    
    if draw.is_custom_vao() {
      self.gl2D.custom_vao.get(draw.get_text()).unwrap().draw_indexed(gl::TRIANGLES);
    } else {
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
    }
  }
}

impl CoreRender for RawGl {
  fn load_instanced(&mut self, reference: String, max_instances: i32) {
    self.load_2d_instanced_vao(reference, max_instances);
  }
  
  fn load_static_geometry(&mut self, reference: String, vertices: Vec<graphics::Vertex2d>, indices: Vec<u32>) {
    let mut verts: Vec<GLfloat> = Vec::new();
    for v in vertices {
      verts.push(v.position[0] as GLfloat);
      verts.push(v.position[1] as GLfloat);
      verts.push(v.uv[0] as GLfloat);
      verts.push(v.uv[1] as GLfloat);
    };
    
    let index = indices.into_iter().map(|i| {
      i as GLuint
    }).collect::<Vec<GLuint>>();
    
    self.load_custom_2d_vao(reference, verts, index, false);
  }
  
  fn load_dynamic_geometry(&mut self, reference: String, vertices: Vec<graphics::Vertex2d>, indices: Vec<u32>) {
    let mut verts: Vec<GLfloat> = Vec::new();
    for v in vertices {
      verts.push(v.position[0] as GLfloat);
      verts.push(v.position[1] as GLfloat);
      verts.push(v.uv[0] as GLfloat);
      verts.push(v.uv[1] as GLfloat);
    };
    
    let index = indices.iter().map(|i| {
      *i as GLuint
    }).collect::<Vec<GLuint>>();
    
    self.load_custom_2d_vao(reference, verts, index, true);
  }
  
  fn preload_model(&mut self, reference: String, location: String, texture: String) {
    self.load_model(reference.clone(), location, texture.clone());
    self.load_texture(reference, texture);
  }
  
  fn add_model(&mut self, reference: String, location: String, texture: String) {
    self.model_paths.insert(reference.clone(), ModelInfo {location: location, texture: texture.clone()});
    self.add_texture(reference, texture);
  }
  
  fn load_model(&mut self, reference: String, location: String, texture: String) {
    let start_time = time::Instant::now();
    let model_data = OpengexPaser::new(location.clone());
    
    let vertex = model_data.get_vertex();
    let normal = model_data.get_normal();
    let uvs = model_data.get_uv();
    let index = model_data.get_index();
    
    let i = 0;
    
    let mut vertices: Vec<GLfloat> = Vec::with_capacity(vertex[i].len());
    for j in 0..vertex[i].len() {
      let mut uv = [0.0, 0.0];
      if uvs[i].len() > j {
        uv = uvs[i][j];
      }
      
      vertices.push(vertex[i][j][0]);
      vertices.push(vertex[i][j][1]);
      vertices.push(vertex[i][j][2]);
      
      vertices.push(normal[i][j][0]);
      vertices.push(normal[i][j][1]);
      vertices.push(normal[i][j][2]);
      
      vertices.push(uv[0]);
      vertices.push(uv[1]);
    }
    
    let indices = index[i].iter().map( |index| {
        *index as GLuint
      }
    ).collect::<Vec<GLuint>>();
    
    let mut vao: Vao = Vao::new();
    
    vao.bind();
    vao.create_ebo(indices, gl::STATIC_DRAW);
    vao.create_vbo(vertices, gl::STATIC_DRAW);
    
    vao.set_vertex_attrib(0, 3, 8, 0);
    vao.set_vertex_attrib(1, 3, 8, 3);
    vao.set_vertex_attrib(2, 2, 8, 6);
    
    self.gl3D.models.insert(reference, vao);
    
    /*
    let model = model_data::Loader::load_opengex(location.clone(), texture);
    
    let vertex = model.get_verticies();
    
    let mut vertices: Vec<GLfloat> = Vec::new();
    
    for vertex in model.get_verticies() {
      vertices.push(vertex.position[0]);
      vertices.push(vertex.position[1]);
      vertices.push(vertex.position[2]);
      
      vertices.push(vertex.normal[0]);
      vertices.push(vertex.normal[1]);
      vertices.push(vertex.normal[2]);
      
      vertices.push(vertex.uv[0]);
      vertices.push(vertex.uv[1]);
    }
    
    let indices = model.get_indices().iter().map( |index| {
        *index as GLuint
      }
    ).collect::<Vec<GLuint>>();
    
    let mut vao: Vao = Vao::new();
    
    vao.bind();
    vao.create_ebo(indices, gl::STATIC_DRAW);
    vao.create_vbo(vertices, gl::STATIC_DRAW);
    
    vao.set_vertex_attrib(0, 3, 8, 0);
    vao.set_vertex_attrib(1, 3, 8, 3);
    vao.set_vertex_attrib(2, 2, 8, 6);
    
    self.gl3D.models.insert(reference, vao);*/
    
    let total_time = start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (total_time*1000f64) as f32, location);
  }
 
  fn preload_texture(&mut self, reference: String, location: String) {
    self.load_texture(reference, location);
  }
  
  fn add_texture(&mut self, reference: String, location: String) {
    self.texture_paths.insert(reference, location);
  }
 
  fn load_texture(&mut self, reference: String, location: String) {
    if location == String::from("") {
      return;
    }
    
    let texture_start_time = time::Instant::now();
    let mut texture_id: GLuint = 0;
    
    unsafe {
      gl::GenTextures(1, &mut texture_id);
      
      gl::BindTexture(gl::TEXTURE_2D, texture_id);
      
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
      //gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
      
      let texture = location.clone();
      let image = image::open(&location).expect(&("No file or Directory at: ".to_string() + &texture)).to_rgba(); 
      let (width, height) = image.dimensions();
      let image_data = image.into_raw().clone();
     
      gl::TexImage2D(gl::TEXTURE_2D, 0,
                    gl::RGBA as GLint,
                    width as GLsizei,
                    height as GLsizei,
                    0, gl::RGBA, gl::UNSIGNED_BYTE,
                    mem::transmute(&image_data[0]));
      gl::GenerateMipmap(gl::TEXTURE_2D);
      
      gl::BindTexture(gl::TEXTURE_2D, 0);
    }
    
    self.textures.insert(reference, texture_id);
         
    let texture_time = texture_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (texture_time*1000f64) as f32, location);
  }
  
  fn preload_font(&mut self, reference: String, font: &[u8], font_texture: String) {
    self.load_font(reference.clone(), font);
    self.preload_texture(reference, font_texture);
  }
  
  fn add_font(&mut self, reference: String, location: &[u8], font_texture: String) {
    self.load_font(reference.clone(), location);
    self.add_texture(reference, font_texture);
  }
  
  fn load_font(&mut self, reference: String, font: &[u8]) {
    let mut new_font = GenericFont::new();
    new_font.load_font(font);
    self.fonts.insert(reference.clone(), new_font);
  }
  
  fn load_shaders(&mut self) {
    self.gl2D.shaders.push(Box::new(ShaderTexture::new()));
    self.gl2D.shaders.push(Box::new(ShaderText::new()));
    self.gl2D.shaders.push(Box::new(ShaderTextureInstanced::new()));
    self.gl2D.shaders.push(Box::new(ShaderBloom::new()));
    self.gl2D.shaders.push(Box::new(ShaderBlur::new()));
    self.gl2D.shaders.push(Box::new(ShaderFinal::new()));
    self.gl3D.shaders.push(Box::new(Shader3D::new()));
  }
  
  fn init(&mut self) {
    let dim = self.get_dimensions();
    
    self.framebuffer.init();
    self.framebuffer_bloom.init();
    self.framebuffer_blur_ping.init();
    self.framebuffer_blur_pong.init();
    self.set_viewport(dim[0], dim[1]);
    
    unsafe {
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    
    self.load_2d_vao();
    
    self.gl2D.shaders[TEXTURE].Use();
    self.gl2D.shaders[TEXTURE].set_int(String::from("tex"), 0);
    self.gl2D.shaders[TEXTURE].set_mat4(String::from("projection"), self.gl2D.projection);
    
    self.gl2D.shaders[INSTANCED].Use();
    self.gl2D.shaders[INSTANCED].set_int(String::from("tex"), 0);
    self.gl2D.shaders[INSTANCED].set_mat4(String::from("projection"), self.gl2D.projection);
    
    self.gl2D.shaders[TEXT].Use();
    self.gl2D.shaders[TEXT].set_int(String::from("tex"), 0);
    self.gl2D.shaders[TEXT].set_mat4(String::from("projection"), self.gl2D.projection);
    
    self.gl2D.shaders[BLOOM].Use();
    self.gl2D.shaders[BLOOM].set_int(String::from("tex"), 0);
    self.gl2D.shaders[BLOOM].set_mat4(String::from("projection"), self.gl2D.projection);
    
    self.gl2D.shaders[BLUR].Use();
    self.gl2D.shaders[BLUR].set_int(String::from("tex"), 0);
    self.gl2D.shaders[BLUR].set_mat4(String::from("projection"), self.gl2D.projection);
    
    self.gl2D.shaders[FINAL].Use();
    self.gl2D.shaders[FINAL].set_int(String::from("tex"), 0);
    self.gl2D.shaders[FINAL].set_int(String::from("bloom"), 1);
    self.gl2D.shaders[FINAL].set_mat4(String::from("projection"), self.gl2D.projection);
    
    self.gl3D.shaders[MODEL].Use();
    self.gl3D.shaders[MODEL].set_int(String::from("tex"), 0);
    self.gl3D.shaders[MODEL].set_mat4(String::from("projection"), self.gl3D.projection);
  }
  
  fn dynamic_load(&mut self) {
    let time_limit = 9.0;
    
    let mut delta_time;
    let frame_start_time = time::Instant::now();
  
    let mut still_loading = false;
    
    let texture_paths_clone = self.texture_paths.clone();
    
    for (reference, path) in &texture_paths_clone {
      self.load_texture(reference.clone(), path.clone());
      
      self.texture_paths.remove(reference);
      still_loading = true;
      
      delta_time = frame_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
      if (delta_time*1000f64) > time_limit {
        break;
      } 
    }
    
    delta_time = frame_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    if (delta_time*1000f64) > time_limit {
      return;
    }
    
    let model_paths_clone = self.model_paths.clone();
    
    for (reference, model) in &model_paths_clone {
      self.load_model(reference.clone(), model.location.clone(), model.texture.clone());
      
      self.model_paths.remove(reference);
      still_loading = true;
      
      delta_time = frame_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
      if (delta_time*1000f64) > time_limit {
        break;
      } 
    }
    
    if !still_loading {
      self.ready = true;
    }
  }
  
  fn clear_screen(&mut self) {
    unsafe {
      gl::DepthMask(gl::TRUE);
      gl::ClearColor(self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
      //gl::DepthMask(gl::FALSE);
    }
  }
  
  fn pre_draw(&mut self) {
    self.clear_screen();
  }
  
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
    let dimensions = self.get_dimensions();
    
    self.framebuffer.bind();
    
    self.clear_screen();
    
    let mut offset = 0;
    for i in 0..draw_calls.len() {
      if i+offset >= draw_calls.len() {
        break;
      }
      
      let draw = draw_calls[i+offset].clone();
      
      if draw.is_3d_model() {
        self.draw_3d(&draw);
      } else if draw.is_text() {
        self.draw_text(&draw);
      } else if draw.is_vao_update() {
        self.update_vao(&draw);
      } else if draw.is_instanced() {
        offset += self.draw_instanced(draw_calls.clone(), i+offset)-1;
      } else {
        self.draw_square(&draw);
      }
    }
    
    self.framebuffer.resolve_multisample();
    
    let x = dimensions[0] as f32*0.5;
    let y = dimensions[1] as f32*0.5;
    let width = dimensions[0] as f32;
    let height = dimensions[1] as f32;
    
    let blur_x = BLUR_DIM as f32*0.5;
    let blur_y = BLUR_DIM as f32*0.5;
    let blur_width = BLUR_DIM as f32;
    let blur_height = BLUR_DIM as f32;
    
    // Bloom Draw
    self.framebuffer_bloom.bind();
    
    self.clear_screen();
    let draw = self.framebuffer.draw_screen_texture(x, y, width, height);
    let texture = self.framebuffer.get_screen_texture();
    self.draw_bloom(draw, texture);
    //self.framebuffer_bloom.resolve_multisample();
    
    // Horizontal blur
    self.framebuffer_blur_ping.bind();
    
    self.clear_screen();
    let draw = self.framebuffer_bloom.draw_screen_texture(blur_x, blur_y, blur_width, blur_height);
    let texture = self.framebuffer_bloom.get_screen_texture();
    self.draw_blur(draw, texture, Vector2::new(1.0, 0.0));
    //self.framebuffer_blur_ping.resolve_multisample();
    
    // Verticle blur
    self.framebuffer_blur_pong.bind();
    
    self.clear_screen();
    let draw = self.framebuffer_blur_ping.draw_screen_texture(blur_x, blur_y, blur_width, blur_height);
    let texture = self.framebuffer_blur_ping.get_screen_texture();
    self.draw_blur(draw, texture, Vector2::new(0.0, 1.0));
    //self.framebuffer_blur_pong.resolve_multisample();
    
    // Final Draw
    self.framebuffer.bind_default();
    
    self.clear_screen();
    
    let draw = self.framebuffer.draw_screen_texture(x, y, width, height);
    let base_texture = self.framebuffer.get_screen_texture();
    let bloom_texture = self.framebuffer_blur_pong.get_screen_texture();
    self.draw_final_frame(draw, base_texture, bloom_texture, true);
  }
  
  fn post_draw(&self) {
    unsafe {
      gl::UseProgram(0);
      gl::BindVertexArray(0);
      gl::DisableVertexAttribArray(0 as GLuint);
      gl::DisableVertexAttribArray(1 as GLuint);
      gl::DisableVertexAttribArray(2 as GLuint);
      gl::DisableVertexAttribArray(3 as GLuint);
      gl::DisableVertexAttribArray(4 as GLuint);
      gl::DisableVertexAttribArray(5 as GLuint);
      gl::DisableVertexAttribArray(6 as GLuint);
      gl::DisableVertexAttribArray(7 as GLuint);
    }
  }
  
  fn clean(&self) {
  /*  unsafe {
      for shader in &self.shader_id {
        gl::DeleteProgram(*shader);
      }
      gl::DeleteVertexArrays(1, &self.vao_2d);
    }*/
  }
  
  fn swap_buffers(&mut self) {
    self.window.swap_buffers();
  }
  
  fn screen_resized(&mut self) {
    let mut dimensions = self.get_dimensions();
    if dimensions[0] <= 0 {
      dimensions[0] = self.min_dimensions[0];
    }
    if dimensions[1] <= 0 {
      dimensions[1] = self.min_dimensions[1];
    }
    
    let projection_2d = RawGl::load_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
    let projection_3d = RawGl::load_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
    self.window.resize_screen(dimensions);
    self.set_viewport(dimensions[0], dimensions[1]);
    self.gl2D.shaders[INSTANCED].Use();
    self.gl2D.shaders[INSTANCED].set_mat4(String::from("projection"), projection_2d);
    self.gl2D.shaders[TEXTURE].Use();
    self.gl2D.shaders[TEXTURE].set_mat4(String::from("projection"), projection_2d);
    self.gl2D.shaders[TEXT].Use();
    self.gl2D.shaders[TEXT].set_mat4(String::from("projection"), projection_2d);
    self.gl2D.shaders[BLOOM].Use();
    self.gl2D.shaders[BLOOM].set_mat4(String::from("projection"), projection_2d);
    self.gl2D.shaders[BLUR].Use();
    self.gl2D.shaders[BLUR].set_mat4(String::from("projection"), projection_2d);
    self.gl2D.shaders[FINAL].Use();
    self.gl2D.shaders[FINAL].set_mat4(String::from("projection"), projection_2d);
    self.gl3D.shaders[MODEL].Use();
    self.gl3D.shaders[MODEL].set_mat4(String::from("projection"), projection_3d);
    
    self.framebuffer.resize(dimensions[0] as f32, dimensions[1] as f32);
    
    unsafe {
      gl::UseProgram(0);
    }
  }
  
  fn get_dimensions(&self) -> [u32; 2] {
    let dimensions: [u32; 2] = self.window.get_dimensions();
    dimensions
  }
  
  fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn get_fonts(&self) -> HashMap<String, GenericFont> {
    self.fonts.clone()
  }
  
  fn get_dpi_scale(&self) -> f32 {
    self.window.get_dpi_scale()
  }
  
  fn is_ready(&self) -> bool {
    self.ready
  }
  
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
    self.clear_colour = Vector4::new(r,g,b,a);
  }
  
  fn show_cursor(&mut self) {
    self.window.show_cursor();
  }
  
  fn hide_cursor(&mut self) {
    self.window.hide_cursor();
  }
  
  fn set_camera(&mut self, camera: Camera){
    self.gl3D.camera = camera;
  }
  
  fn get_camera(&self) -> Camera { 
    self.gl3D.camera.to_owned() 
  }
}

