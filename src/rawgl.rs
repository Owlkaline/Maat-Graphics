use window::GlWindow;
use drawcalls::DrawCall;
use drawcalls::DrawMath;
use shaders::ShaderFunctions;
use shaders::ShaderProgram;
use shaders::ShaderTexture;
use shaders::ShaderText;
use shaders::Shader3D;
use graphics::CoreRender;
use settings::Settings;
use font::GenericFont;
use camera::Camera;
use model_data;

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
use std::time;
use std::f32::consts;
use std::ffi::CString;
use std::os::raw::c_void;
use std::collections::HashMap;

pub const TEXTURE: usize = 0;
pub const TEXT: usize = 1;

pub const MODEL: usize = 0;

pub struct Vao {
  vao: GLuint,
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
      ebo: 0,
      num_vertices: 0,
      num_indices: 0,
      attrib: Vec::new(),
    }
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
    self.bind_ebo();
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
  
  pub fn activate_texture0(&self, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE0);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
}

pub struct GL2D {
  shaders: Vec<Box<ShaderFunctions>>,
  vao: Vao,
  projection: Matrix4<f32>
}

pub struct GL3D {
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
  
  view: Matrix4<f32>,
  scale: Matrix4<f32>,
  
  pub window: GlWindow,
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
    
    let window = GlWindow::new(width, height, min_width, min_height, fullscreen);
    
    let proj_2d = RawGl::load_2d_projection(width as f32, height as f32);
    let proj_3d = RawGl::load_3d_projection(width as f32, height as f32);
    
    let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0.0, 0.0, -1.0), cgmath::Point3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, -1.0, 0.0));
    let scale = cgmath::Matrix4::from_scale(0.1);
    
    unsafe {
      gl::Viewport(0, 0, (width as i32 *2) as i32, (height as i32 *2) as i32);
    }
    
    RawGl {
      ready: false,
      shader_id: Vec::with_capacity(2),
      fonts: HashMap::with_capacity(10),
      textures: HashMap::with_capacity(10),
      texture_paths: HashMap::with_capacity(10),
      model_paths: HashMap::with_capacity(10),

      clear_colour: Vector4::new(0.0, 0.0, 0.0, 1.0),

      gl2D: GL2D {
        shaders: Vec::new(),
        vao: Vao::new(),
        projection: proj_2d,
      },
      
      gl3D: GL3D {
        shaders: Vec::new(),
        models: HashMap::new(),
        projection: proj_3d,
      },
      
      view: view,
      scale: scale,
      
      window: window,
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
  
  fn draw_3d(&mut self, draw: &DrawCall) {
    
    unsafe {
      gl::Enable(gl::DEPTH_TEST);
      gl::DepthFunc(gl::LESS);
    }
    
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
    
    let view = self.view /* Matrix4::from_angle_y(Deg(180.0)) */* self.scale;
    
    let mut texture = String::from("default");
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
    
    model.activate_texture0(texture);
    model.draw_indexed(gl::TRIANGLES);
    
    unsafe {
      gl::Disable(gl::DEPTH_TEST);
    }
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
    
    let model = DrawMath::calculate_texture_model(draw.get_translation(), draw.get_size(), -(draw.get_x_rotation()+180.0));
    
    self.gl2D.shaders[TEXTURE].Use();
    self.gl2D.shaders[TEXTURE].set_mat4(String::from("model"), model);
    self.gl2D.shaders[TEXTURE].set_vec4(String::from("new_colour"), colour);
    self.gl2D.shaders[TEXTURE].set_float(String::from("has_texture"), has_texture);
    if has_texture == 1.0 {
      self.gl2D.vao.activate_texture0(*self.textures.get(draw.get_texture()).unwrap());
    }
    
    self.gl2D.vao.draw_indexed(gl::TRIANGLES);
  }
  
  fn draw_text(&self, draw: &DrawCall) {
    let mut translation = draw.get_translation();
    
    let wrapped_draw = DrawMath::setup_correct_wrapping(draw.clone(), self.fonts.clone());
    let size = draw.get_x_size();
    
    for letter in wrapped_draw {
      let char_letter = {
        letter.get_text().as_bytes()[0] 
      };
      
      let c = self.fonts.get(draw.get_texture()).unwrap().get_character(char_letter as i32);
      
      let model = DrawMath::calculate_text_model(letter.get_translation(), size, &c.clone(), char_letter);
      let letter_uv = DrawMath::calculate_text_uv(&c.clone());
      let colour = letter.get_colour();
      let outline = letter.get_outline_colour();
      let edge_width = letter.get_edge_width(); 
      
      self.gl2D.shaders[TEXT].Use();
      self.gl2D.shaders[TEXT].set_mat4(String::from("model"), model);
      self.gl2D.shaders[TEXT].set_vec4(String::from("colour"), colour);
      self.gl2D.shaders[TEXT].set_vec4(String::from("letter_uv"), letter_uv);
      self.gl2D.shaders[TEXT].set_vec3(String::from("outlineColour"), outline);
      self.gl2D.shaders[TEXT].set_vec4(String::from("edge_width"), edge_width);
      
      self.gl2D.vao.activate_texture0(*self.textures.get(draw.get_texture()).unwrap());
      
      self.gl2D.vao.draw_indexed(gl::TRIANGLES);
      
      translation.x+=c.get_advance() as f32 * (size/640.0); 
    }
  }
}

impl CoreRender for RawGl {
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
    
    let indices = model.get_indicies().iter().map( |index| {
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
    self.gl3D.shaders.push(Box::new(Shader3D::new()));
  }
  
  fn init(&mut self) {
    unsafe {
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    
    self.load_2d_vao();
    
    self.gl2D.shaders[TEXTURE].Use();
    self.gl2D.shaders[TEXTURE].set_int(String::from("tex"), 0);
    self.gl2D.shaders[TEXTURE].set_mat4(String::from("projection"), self.gl2D.projection);
    
    self.gl2D.shaders[TEXT].Use();
    self.gl2D.shaders[TEXT].set_int(String::from("tex"), 0);
    self.gl2D.shaders[TEXT].set_mat4(String::from("projection"), self.gl2D.projection);
    
    self.gl3D.shaders[MODEL].Use();
    self.gl3D.shaders[MODEL].set_int(String::from("tex"), 0);
    self.gl3D.shaders[MODEL].set_mat4(String::from("projection"), self.gl3D.projection);
  }
  
  fn dynamic_load(&mut self) {
    let time_limit = 9.0;
    
    let mut delta_time;
    let frame_start_time = time::Instant::now();
  
    let mut still_loading = false;
    //let mut to_be_removed: Vec<String> = Vec::new();
    
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
      gl::ClearColor(self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
  }
  
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
    for draw in draw_calls {
      if draw.is_3d_model() {
        self.draw_3d(draw);
      } else if draw.get_text() != "" {
        self.draw_text(draw);
      } else {
        self.draw_square(draw);
      }
    }
  }
  
  fn post_draw(&self) {
    unsafe {
      gl::UseProgram(0);
      gl::BindVertexArray(0);
      gl::DisableVertexAttribArray(0 as GLuint);
      gl::DisableVertexAttribArray(1 as GLuint);
      gl::DisableVertexAttribArray(2 as GLuint);
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
    let dimensions = self.get_dimensions();
    let projection_2d = RawGl::load_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
    let projection_3d = RawGl::load_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
    self.window.resize_screen(dimensions);
    
    unsafe {
      gl::Viewport(0, 0, dimensions[0] as i32, dimensions[1] as i32);
      
      self.gl2D.shaders[TEXTURE].Use();
      self.gl2D.shaders[TEXTURE].set_mat4(String::from("projection"), projection_2d);
      self.gl2D.shaders[TEXT].Use();
      self.gl2D.shaders[TEXT].set_mat4(String::from("projection"), projection_2d);
      self.gl3D.shaders[MODEL].Use();
      self.gl3D.shaders[MODEL].set_mat4(String::from("projection"), projection_3d);
      
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
  
  fn set_camera(&mut self, camera: Camera){}
  fn pre_draw(&mut self) {}
    fn set_camera_location(&mut self, camera: Vector3<f32>, camera_rot: Vector2<f32>) {

    //let (x_rot, z_rot) = DrawMath::calculate_y_rotation(camera_rot.y);
    let (x_rot, z_rot) = DrawMath::rotate(camera_rot.y);
    
    self.view = cgmath::Matrix4::look_at(cgmath::Point3::new(camera.x, camera.y, camera.z), cgmath::Point3::new(camera.x-x_rot, camera.y, camera.z-z_rot), cgmath::Vector3::new(0.0, -1.0, 0.0));
  }
}





