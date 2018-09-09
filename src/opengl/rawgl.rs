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
use opengl::opengl_3d;
use opengl::vao::Vao;
use opengl::vao::Vao3D;
use opengl::vao::InstancedVao;

use gltf_interpreter::ModelDetails;
use gltf::material::AlphaMode;

use cgmath;
use cgmath::Deg;
use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;

use image;
use winit;
use winit::dpi::LogicalSize;

use gl;
use gl::types::*;

use std::env;
use std::mem;
use std::cmp;
use std::time;
use std::ffi::CStr;
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

struct Uniform3D {
  alpha_cutoff: f32,
  base_colour_factor: Vector4<f32>,
  metallic_roughness_factor: Vector2<f32>,
  normal_scale: f32,
  occlusion_strength: f32,
  emissive_factor: Vector3<f32>,
  forced_alpha: i32,
  has_normals: i32,
  has_tangents: i32,
  has_colour_texture: i32,
  has_metallic_roughness_texture: i32,
  has_normal_texture: i32,
  has_occlusion_texture: i32,
  has_emissive_texture: i32,
}

struct Model3D {
  vao: Vao3D,
  uniforms: Uniform3D,
  base_texture: Option<GLuint>,
  metallic_roughness_texture: Option<GLuint>,
  normal_texture: Option<GLuint>,
  occlusion_texture: Option<GLuint>,
  emissive_texture: Option<GLuint>,
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
  models: HashMap<String, Vec<Model3D>>,
  projection: Matrix4<f32>,
}

#[derive(Clone)]
pub struct ModelInfo {
  location: String,
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

impl Uniform3D {
  pub fn new() -> Uniform3D {
    Uniform3D {
      alpha_cutoff: 0.5,
      base_colour_factor: Vector4::new(0.0, 0.0, 0.0, 1.0),
      metallic_roughness_factor: Vector2::new(0.0, 0.0),
      normal_scale: 1.0,
      occlusion_strength: 1.0,
      emissive_factor: Vector3::new(1.0, 1.0, 1.0),
      forced_alpha: -1,
      has_normals: -1,
      has_tangents: -1,
      has_colour_texture: -1,
      has_metallic_roughness_texture: -1,
      has_normal_texture: -1,
      has_occlusion_texture: -1,
      has_emissive_texture: -1,
    }
  }
}

impl Model3D {
  pub fn new() -> Model3D {
    Model3D {
      vao: Vao3D::new(),
      uniforms: Uniform3D::new(),
      base_texture: None,
      metallic_roughness_texture: None,
      normal_texture: None,
      occlusion_texture: None,
      emissive_texture: None,
    }
  }
  
  pub fn with_vao(mut self, vao: Vao3D) -> Self {
    self.vao = vao;
    self
  }
}

extern "system" fn opengl_debug(source: GLenum, _type: GLenum, id: GLuint, severity: GLenum, 
                    _length: GLsizei, messages: *const GLchar, _user: *mut c_void) {
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
    let width = settings.get_resolution()[0] as f64;
    let height = settings.get_resolution()[1] as f64;
    let min_width = settings.get_minimum_resolution()[0];
    let min_height = settings.get_minimum_resolution()[1];
    let fullscreen = settings.is_fullscreen();
    let mut msaa_samples = settings.get_msaa();
    let vsync = settings.vsync_enabled();
    
    let window = GlWindow::new(width, height, fullscreen, vsync);
    
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
      
      framebuffer: Fbo::new(msaa_samples, 1, true, width as i32, height as i32),
      framebuffer_bloom: Fbo::new(1, 1, true, width as i32, height as i32),
      framebuffer_blur_ping: Fbo::new(1, 1, false, BLUR_DIM as i32, BLUR_DIM as i32),
      framebuffer_blur_pong: Fbo::new(1, 1, false, BLUR_DIM as i32, BLUR_DIM as i32),
      
      view: view,
      scale: scale,
      
      min_dimensions: [min_width, min_height],
      window: window,
    }
  }
  
  pub fn set_viewport(&self, dim: LogicalSize) {
    unsafe {
      gl::Viewport(0, 0, (dim.width as f64 * self.get_dpi_scale()) as i32, (dim.height as f64 * self.get_dpi_scale()) as i32);
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
      vao.create_ebo(indicies, gl::DYNAMIC_DRAW);
      vao.create_vbo(verts, gl::DYNAMIC_DRAW);
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
    
    if let Some((new_vertices, new_indices)) = draw.new_shape_details() {
      for v in new_vertices {
        verts.push(v.position[0] as GLfloat);
        verts.push(v.position[1] as GLfloat);
        verts.push(v.uv[0] as GLfloat);
        verts.push(v.uv[1] as GLfloat);
      };
      
      let index = new_indices.iter().map(|i| {
        *i as GLuint
      }).collect::<Vec<GLuint>>();
      
      if let Some(reference) = draw.display_text() {
        if let Some(custom_vao) = self.gl2D.custom_vao.get_mut(&reference) {
          custom_vao.update_vbo(verts);
          custom_vao.update_ebo(index);
        } else {
          println!("Error: custom vao doesnt exist: {:?}", reference);
        }
      }
    }
  }
  
  fn draw_3d(&mut self, draw: &DrawCall) {
    unsafe {
      gl::Enable(gl::DEPTH_TEST);
      gl::DepthFunc(gl::LESS);
    }
    
    if let Some(texture_name) = draw.texture_name() {
      if self.gl3D.models.contains_key(&texture_name) {
        let rotation_x: Matrix4<f32> = Matrix4::from_angle_x(Deg(draw.rotation().x));
        let rotation_y: Matrix4<f32> = Matrix4::from_angle_y(Deg(draw.rotation().y));
        let rotation_z: Matrix4<f32> = Matrix4::from_angle_z(Deg(draw.rotation().z));
          
        let transformation: Matrix4<f32> = (cgmath::Matrix4::from_translation(draw.position()) * cgmath::Matrix4::from_scale(draw.scale().x)) * (rotation_x*rotation_y*rotation_z);
        
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
        
        self.gl3D.shaders[MODEL].Use();
        self.gl3D.shaders[MODEL].set_mat4(String::from("u_transformation"), transformation);
        self.gl3D.shaders[MODEL].set_mat4(String::from("u_view"), view);
        self.gl3D.shaders[MODEL].set_mat4(String::from("u_projection"), self.gl3D.projection);
        
        if let Some(model) = self.gl3D.models.get(&texture_name) {
          for i in 0..model.len() {
            self.gl3D.shaders[MODEL].set_vec4(String::from("u_base_colour_factor"),
                                               model[i].uniforms.base_colour_factor);
            self.gl3D.shaders[MODEL].set_vec2(String::from("u_metallic_roughness_factor"),
                                               model[i].uniforms.metallic_roughness_factor);
            
            self.gl3D.shaders[MODEL].set_float(String::from("u_alpha_cutoff"), 
                                               model[i].uniforms.alpha_cutoff);
            self.gl3D.shaders[MODEL].set_float(String::from("u_normal_scale"), 
                                               model[i].uniforms.normal_scale);
            self.gl3D.shaders[MODEL].set_float(String::from("u_occlusion_strength"), 
                                               model[i].uniforms.occlusion_strength);
            self.gl3D.shaders[MODEL].set_vec3(String::from("u_emissive_factor"), 
                                               model[i].uniforms.emissive_factor);
            
            self.gl3D.shaders[MODEL].set_int(String::from("u_forced_alpha"), 
                                             model[i].uniforms.forced_alpha);
            self.gl3D.shaders[MODEL].set_int(String::from("u_has_normals"), 
                                             model[i].uniforms.has_normals);
            self.gl3D.shaders[MODEL].set_int(String::from("u_has_tangents"), 
                                             model[i].uniforms.has_tangents);
            
            self.gl3D.shaders[MODEL].set_int(String::from("u_has_colour_texture"), 
                                             model[i].uniforms.has_colour_texture);
            self.gl3D.shaders[MODEL].set_int(String::from("u_has_metallic_roughness_texture"), 
                                             model[i].uniforms.has_metallic_roughness_texture);
            self.gl3D.shaders[MODEL].set_int(String::from("u_has_normal_texture"), 
                                             model[i].uniforms.has_normal_texture);
            self.gl3D.shaders[MODEL].set_int(String::from("u_has_occlusion_texture"), 
                                             model[i].uniforms.has_occlusion_texture);
            self.gl3D.shaders[MODEL].set_int(String::from("u_has_emissive_texture"), 
                                             model[i].uniforms.has_emissive_texture);
            
            if let Some(texture) = model[i].base_texture {
              model[i].vao.activate_texture(0, texture);
            }
            if let Some(texture) = model[i].metallic_roughness_texture {
              //model[i].vao.activate_texture(1, texture);
            }
            if let Some(texture) = model[i].normal_texture {
              //model[i].vao.activate_texture(2, texture);
            }
            if let Some(texture) = model[i].occlusion_texture {
             // model[i].vao.activate_texture(3, texture);
            }
            if let Some(texture) = model[i].emissive_texture {
           //   model[i].vao.activate_texture(4, texture);
            }
            
            model[i].vao.draw_indexed(gl::TRIANGLES);
            
            model[i].vao.activate_texture(0, 0);
            model[i].vao.activate_texture(1, 0);
            model[i].vao.activate_texture(2, 0);
            model[i].vao.activate_texture(3, 0);
            model[i].vao.activate_texture(4, 0);
          }
        }
      } else {
        println!("Error: 3D model not found: {:?}", texture_name);
      }
    }
    
    unsafe {
      gl::Disable(gl::DEPTH_TEST);
    }
  }
  
  fn draw_instanced(&mut self, draw_calls: Vec<DrawCall>, offset: usize) -> usize {
    let mut num_instances = 0;
    
    if let Some(instance_name) = draw_calls[offset].instance_name() {
      if !self.gl2D.instanced_vao.contains_key(&instance_name) {
        println!("Error: Instanced vao not found: {:?}", instance_name);
        return 0;
      }
      
      if let Some(texture_name) = draw_calls[offset].texture_name() {
        for i in offset..draw_calls.len() {
          if let Some(test_texture_name) = draw_calls[i].texture_name() {
            if draw_calls[i].is_instanced_texture() && test_texture_name == texture_name {
              num_instances += 1;
            } else {
              break;
            }
          }
        }
        
        let mut new_data: Vec<GLfloat> = Vec::new();
            
        let has_texture = {
          let mut value = 1.0;
          if texture_name == String::from("") {
            value = 0.0;
          }
          value
        };
        
        for i in offset..offset+num_instances {
          let draw = draw_calls[i].clone();
          
          let colour: [f32; 4] = draw.colour().into();
          let mut bw: f32 = 0.0;
          if draw.black_and_white_enabled() {
            bw = 1.0;
          }
          
          let model = math::calculate_texture_model(draw.position(), Vector2::new(draw.scale().x, draw.scale().y), -(draw.rotation().x));
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
          if let Some(instance_name) = draw.instance_name() {
            self.gl2D.instanced_vao[&instance_name].activate_texture(0, *self.textures.get(&texture_name).expect("Texture not found!"));
          }
        }
        
        if let Some(instance_name) = draw.instance_name() {
          self.gl2D.shaders[INSTANCED].Use();
          self.gl2D.instanced_vao[&instance_name].bind();
          self.gl2D.instanced_vao[&instance_name].update_vbodata(num_instances, new_data);
          self.gl2D.instanced_vao[&instance_name].draw_indexed_instanced(num_instances, gl::TRIANGLES);
          self.gl2D.instanced_vao[&instance_name].unbind();
        }
      }
    }
    
    num_instances
  }
  
  fn draw_square(&self, draw: &DrawCall) {
    let colour = draw.colour();
    let has_texture = {
      let mut value = 1.0;
      if let Some(texture_name) = draw.texture_name() {
        if texture_name == String::from("") {
          value = 0.0;
        }
      }
      value
    };
    
    let mut is_blackwhite = 0.0;
    if draw.black_and_white_enabled() {
      is_blackwhite = 1.0;
    }
    let textured_blackwhite = Vector2::new(has_texture, is_blackwhite);
    
    let model = math::calculate_texture_model(draw.position(), Vector2::new(draw.scale().x, draw.scale().y), -(draw.rotation().x));
    
    self.gl2D.shaders[TEXTURE].Use();
    self.gl2D.shaders[TEXTURE].set_mat4(String::from("model"), model);
    self.gl2D.shaders[TEXTURE].set_vec4(String::from("new_colour"), colour);
    self.gl2D.shaders[TEXTURE].set_vec2(String::from("textured_blackwhite"), textured_blackwhite);
    if has_texture == 1.0 {
      if let Some(texture_name) = draw.texture_name() {
        if self.textures.contains_key(&texture_name) {
          self.gl2D.vao.activate_texture(0, *self.textures.get(&texture_name).unwrap());
        } else {
          println!("Error: Texture not found: {:?}", texture_name);
        }
      }
    }
    
    if draw.is_custom_shape() {
      if let Some(display_text) = draw.display_text() {
        self.gl2D.custom_vao.get(&display_text).unwrap().draw_indexed(gl::TRIANGLES);
      }
    } else {
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
    }
  }
  
  fn draw_text(&self, draw: &DrawCall) {
    if let Some(texture_name) = draw.texture_name() {
      if !self.textures.contains_key(&texture_name) {
        println!("Error: Font texture not found: {:?}", texture_name);
        return;
      }
      
      let mut translation = draw.position();
      
      let wrapped_draw = drawcalls::setup_correct_wrapping(draw.clone(), self.fonts.clone());
      let size = draw.scale().x;
      
      for letter in wrapped_draw {
        let char_letter = {
          letter.display_text().unwrap().as_bytes()[0] 
        };
        
        let c = self.fonts.get(&texture_name).unwrap().get_character(char_letter as i32);
        
        let model = drawcalls::calculate_text_model(letter.position(), size, &c.clone(), char_letter);
        let letter_uv = drawcalls::calculate_text_uv(&c.clone());
        let colour = letter.colour();
        let outline = letter.text_outline_colour();
        let edge_width = letter.text_edge_width(); 
        
        self.gl2D.shaders[TEXT].Use();
        self.gl2D.shaders[TEXT].set_mat4(String::from("model"), model);
        self.gl2D.shaders[TEXT].set_vec4(String::from("colour"), colour);
        self.gl2D.shaders[TEXT].set_vec4(String::from("letter_uv"), letter_uv);
        self.gl2D.shaders[TEXT].set_vec3(String::from("outlineColour"), outline);
        self.gl2D.shaders[TEXT].set_vec4(String::from("edge_width"), edge_width);
        
        self.gl2D.vao.activate_texture(0, *self.textures.get(&texture_name).unwrap());
        
        self.gl2D.vao.draw_indexed(gl::TRIANGLES);
        
        translation.x+=c.get_advance() as f32 * (size/640.0); 
      }
    }
  }
  
  fn draw_framebuffer(&self, draw: DrawCall, texture: GLuint) {
    let colour = Vector4::new(1.0, 0.0, 0.0, 1.0);
    let has_texture = 1.0;
    let mut is_blackwhite = 0.0;
    if draw.black_and_white_enabled() {
      is_blackwhite = 1.0;
    }
    let textured_blackwhite = Vector2::new(has_texture, is_blackwhite);
    
    let model = math::calculate_texture_model(draw.position(), Vector2::new(draw.scale().x, draw.scale().y), -(draw.rotation().x));
    
    self.gl2D.shaders[TEXTURE].Use();
    self.gl2D.shaders[TEXTURE].set_mat4(String::from("model"), model);
    self.gl2D.shaders[TEXTURE].set_vec4(String::from("new_colour"), colour);
    self.gl2D.shaders[TEXTURE].set_vec2(String::from("textured_blackwhite"), textured_blackwhite);
    
    self.gl2D.vao.activate_texture(0, texture);
    
    if draw.is_custom_shape() {
      if let Some(dispaly_text) = draw.display_text() {
        self.gl2D.custom_vao.get(&dispaly_text).unwrap().draw_indexed(gl::TRIANGLES);
      }
    } else {
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
    }
  }
  
  fn draw_bloom(&self, draw: DrawCall, texture: GLuint) {
    let model = math::calculate_texture_model(draw.position(), Vector2::new(draw.scale().x, draw.scale().y), -(draw.rotation().x));
    
    self.gl2D.shaders[BLOOM].Use();
    self.gl2D.shaders[BLOOM].set_mat4(String::from("model"), model);
    
    self.gl2D.vao.activate_texture(0, texture);
    
    if draw.is_custom_shape() {
      if let Some(dispaly_text) = draw.display_text() {
        self.gl2D.custom_vao.get(&dispaly_text).unwrap().draw_indexed(gl::TRIANGLES);
      }
    } else {
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
    }
  }
  
  fn draw_blur(&self, draw: DrawCall, texture: GLuint, direction: Vector2<f32>) {
    let model = math::calculate_texture_model(draw.position(), Vector2::new(draw.scale().x, draw.scale().y), -(draw.rotation().x));
    
    self.gl2D.shaders[BLUR].Use();
    self.gl2D.shaders[BLUR].set_mat4(String::from("model"), model);
    self.gl2D.shaders[BLUR].set_vec2(String::from("direction"), direction);
    
    self.gl2D.vao.activate_texture(0, texture);
    
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
  }
  
  fn draw_final_frame(&self, draw: DrawCall, base_texture: GLuint, bloom_texture: GLuint, bloom: bool) {
    let model = math::calculate_texture_model(draw.position(), Vector2::new(draw.scale().x, draw.scale().y), -(draw.rotation().x));
    
    self.gl2D.shaders[FINAL].Use();
    self.gl2D.shaders[FINAL].set_mat4(String::from("model"), model);
    if bloom {
      self.gl2D.shaders[FINAL].set_float(String::from("bloom_enabled"), 2.0);
    } else {
      self.gl2D.shaders[FINAL].set_float(String::from("bloom_enabled"), 0.0);
    }
    
    self.gl2D.vao.activate_texture(0, base_texture);
    self.gl2D.vao.activate_texture(1, bloom_texture);
    
    if draw.is_custom_shape() {
      if let Some(display_text) = draw.display_text() {
        self.gl2D.custom_vao.get(&display_text).unwrap().draw_indexed(gl::TRIANGLES);
      }
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
  
  fn preload_model(&mut self, reference: String, directory: String) {
    self.load_model(reference.clone(), directory);
    //self.load_texture(reference, texture);
  }
  
  fn add_model(&mut self, reference: String, directory: String) {
    self.model_paths.insert(reference.clone(), ModelInfo {location: directory});
    //self.add_texture(reference, texture);
  }
  
  fn load_model(&mut self, reference: String, directory: String) {
    let start_time = time::Instant::now();
    
    let mesh_data = ModelDetails::new(directory.clone());
    
    let mut model: Vec<Model3D> = Vec::new();
    
    for i in 0..mesh_data.num_models() {
      
      let vertex = mesh_data.vertex(i);
      let texcoord = mesh_data.texcoords(i);
      let normal = mesh_data.normal(i);
      let tangent = mesh_data.tangent(i);
      let index = mesh_data.index(i);
      let colour = mesh_data.colours(i);
      //let alpha_mode = mesh_data.alphamode(i);
    
      let mut vertices: Vec<GLfloat> = Vec::with_capacity(vertex.len());
      for j in 0..vertex.len() {
        let mut uv = [0.0, 0.0];
        if texcoord.len() > j {
          uv = texcoord[j];
        }
        let mut n_normal = [1.0, 1.0, 1.0];
        if normal.len() > j {
          n_normal = normal[j];
        }
        let mut t_tangent = [1.0, 1.0, 1.0, 0.0];
        if tangent.len() > j {
          t_tangent = tangent[j];
        }
        let mut c_colour = [1.0, 1.0, 1.0, 1.0];
        if colour.len() >  j {
          c_colour = colour[j];
        }
        
        vertices.push(vertex[j][0]);
        vertices.push(vertex[j][1]);
        vertices.push(vertex[j][2]);
        
        vertices.push(n_normal[0]);
        vertices.push(n_normal[1]);
        vertices.push(n_normal[2]);
        
        vertices.push(t_tangent[0]);
        vertices.push(t_tangent[1]);
        vertices.push(t_tangent[2]);
        vertices.push(t_tangent[3]);
        
        vertices.push(uv[0]);
        vertices.push(uv[1]);
        
        vertices.push(c_colour[0]);
        vertices.push(c_colour[1]);
        vertices.push(c_colour[2]);
        vertices.push(c_colour[3]);
      }
      
      let indices = index.iter().map( |index| {
          *index as GLuint
        }
      ).collect::<Vec<GLuint>>();
      
      let mut vao: Vao3D = Vao3D::new();
      
      vao.bind();
      vao.create_ebo(indices, gl::STATIC_DRAW);
      vao.create_vbo(vertices, gl::STATIC_DRAW);
      
      vao.set_vertex_attrib(0, 3, 16, 0);
      vao.set_vertex_attrib(1, 3, 16, 3);
      vao.set_vertex_attrib(2, 4, 16, 6);
      vao.set_vertex_attrib(3, 2, 16, 10);
      vao.set_vertex_attrib(4, 4, 16, 12);
      
      let mut new_model = Model3D::new().with_vao(vao);
      
      new_model.base_texture = opengl_3d::create_texture_from_dynamicimage(mesh_data.base_colour_texture(i));
      new_model.metallic_roughness_texture = opengl_3d::create_texture_from_dynamicimage(mesh_data.metallic_roughness_texture(i));
      new_model.normal_texture = opengl_3d::create_texture_from_dynamicimage(mesh_data.normal_texture(i));
      new_model.occlusion_texture = opengl_3d::create_texture_from_dynamicimage(mesh_data.occlusion_texture(i));
      new_model.emissive_texture = opengl_3d::create_texture_from_dynamicimage(mesh_data.emissive_texture(i));
      
      let base_colour_factor = { 
        let colour = mesh_data.base_colour(i);
        Vector4::new(colour[0], colour[1], colour[2], colour[3])
      };
      let metallic_roughness_factor = Vector2::new(mesh_data.metallic_factor(i), mesh_data.roughness_factor(i));
      let normal_scale = mesh_data.normal_texture_scale(i);
      let occlusion_strength = mesh_data.occlusion_texture_strength(i);
      let emissive_factor = {
        let factor = mesh_data.emissive_factor(i);
        Vector3::new(factor[0], factor[1], factor[2])
      };
      
      let has_colour_texture = {
        if new_model.base_texture.is_some() { 0 } else { -1 }
      };
      let has_metallic_roughness_texture = {
        if new_model.metallic_roughness_texture.is_some() { 0 } else { -1 }
      };
      let has_normal_texture = {
        if new_model.normal_texture.is_some() { 0 } else { -1 }
      };
      let has_occlusion_texture = {
        if new_model.occlusion_texture.is_some() { 0 } else { -1 }
      };
      let has_emissive_texture = {
        if new_model.emissive_texture.is_some() { 0 } else { -1 }
      };
      
      let alpha_mode = mesh_data.alphamode(i);
      let mut alpha_cutoff = mesh_data.alphacutoff(i);
      let has_normals = if mesh_data.has_normals(i) { 1 } else { 0 };
      let has_tangents = if mesh_data.has_tangents(i) { 1 } else { 0 };
      
      let mut forced_alpha = 0;
      match alpha_mode {
        AlphaMode::Opaque => {
          forced_alpha = 1;
        },
        AlphaMode::Mask => {
          forced_alpha = 2;
        },
        AlphaMode::Blend => {
          forced_alpha = 0;
        },
      }
      
      let uniform_var = Uniform3D {
        alpha_cutoff: alpha_cutoff,
        base_colour_factor: base_colour_factor,
        metallic_roughness_factor: metallic_roughness_factor,
        normal_scale: normal_scale,
        occlusion_strength: occlusion_strength,
        emissive_factor: emissive_factor,
        forced_alpha: forced_alpha,
        has_normals: has_normals,
        has_tangents: has_tangents,
        has_colour_texture: has_colour_texture,
        has_metallic_roughness_texture: has_metallic_roughness_texture,
        has_normal_texture: has_normal_texture,
        has_occlusion_texture: has_occlusion_texture,
        has_emissive_texture: has_emissive_texture,
      };
      new_model.uniforms = uniform_var;
      model.push(new_model);
    }
    
    self.gl3D.models.insert(reference, model);
    
    let total_time = start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (total_time*1000f64) as f32, directory);
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
    self.set_viewport(dim);
    
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
    self.gl3D.shaders[MODEL].set_int(String::from("u_base_colour"), 0);
    self.gl3D.shaders[MODEL].set_int(String::from("u_metallic_roughness"), 1);
    self.gl3D.shaders[MODEL].set_int(String::from("u_normal_texture"), 2);
    self.gl3D.shaders[MODEL].set_int(String::from("u_occlusion_texture"), 3);
    self.gl3D.shaders[MODEL].set_int(String::from("u_emissive_texture"), 4);
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
      self.load_model(reference.clone(), model.location.clone());
      
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
     
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
      gl::ClearColor(self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w);
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
      
      if draw.is_model() {
        self.draw_3d(&draw);
      } else if draw.is_text() {
        self.draw_text(&draw);
      } else if draw.is_shape_update() {
        self.update_vao(&draw);
      } else if draw.is_instanced_texture() {
        offset += self.draw_instanced(draw_calls.clone(), i+offset)-1;
      } else {
        self.draw_square(&draw);
      }
    }
    
    self.framebuffer.resolve_multisample();
    
    let x = dimensions.width as f32*0.5;
    let y = dimensions.height as f32*0.5;
    let width = dimensions.width as f32;
    let height = dimensions.height as f32;
    
    let blur_x = BLUR_DIM as f32*0.5;
    let blur_y = BLUR_DIM as f32*0.5;
    let blur_width = BLUR_DIM as f32;
    let blur_height = BLUR_DIM as f32;
    
    // Bloom Draw
    self.framebuffer_bloom.bind();
    
    self.clear_screen();
    let draw = self.framebuffer.draw_screen_texture(x, y, width, height);
    let texture = self.framebuffer.get_screen_texture(0);
    self.draw_bloom(draw, texture);
    /*
    // Horizontal blur
    self.framebuffer_blur_ping.bind();
    
    self.clear_screen();
    let draw = self.framebuffer_bloom.draw_screen_texture(blur_x, blur_y, blur_width, blur_height);
    let texture = self.framebuffer_bloom.get_screen_texture(0);
    self.draw_blur(draw, texture, Vector2::new(1.0, 0.0));
    
    // Verticle blur
    self.framebuffer_blur_pong.bind();
    
    self.clear_screen();
    let draw = self.framebuffer_blur_ping.draw_screen_texture(blur_x, blur_y, blur_width, blur_height);
    let texture = self.framebuffer_blur_ping.get_screen_texture(0);
    self.draw_blur(draw, texture, Vector2::new(0.0, 1.0));
    */
    // Final Draw
    self.framebuffer.bind_default();
    
    self.clear_screen();
    
    let draw = self.framebuffer.draw_screen_texture(x, y, width, height);
    let base_texture = self.framebuffer.get_screen_texture(0);
    let bloom_texture = self.framebuffer_bloom.get_screen_texture(0);
    self.draw_final_frame(draw, base_texture, bloom_texture, false);
  }
  
  fn post_draw(&self) {
    unsafe {
      gl::UseProgram(0);
      gl::BindVertexArray(0);
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
  
  fn screen_resized(&mut self, window_size: LogicalSize) {
    let mut window_size = window_size;
    if window_size.width <= 0.0 {
      window_size.width = self.min_dimensions[0] as f64;
    }
    if window_size.height <= 0.0 {
      window_size.height = self.min_dimensions[1] as f64;
    }
    
    let projection_2d = RawGl::load_2d_projection(window_size.width as f32, window_size.height as f32);
    let projection_3d = RawGl::load_3d_projection(window_size.width as f32, window_size.height as f32);
    self.window.resize_screen(window_size);
    self.set_viewport(window_size);
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
    
    self.framebuffer.resize(window_size);
    
    unsafe {
      gl::UseProgram(0);
    }
  }
  
  fn get_dimensions(&self) -> LogicalSize {
    self.window.get_dimensions()
  }
  
  fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn get_fonts(&self) -> HashMap<String, GenericFont> {
    self.fonts.clone()
  }
  
  fn get_dpi_scale(&self) -> f64 {
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
  fn num_drawcalls(&self) -> u32 {0}
  fn set_cursor_position(&mut self, x: f32, y: f32) {}
}

