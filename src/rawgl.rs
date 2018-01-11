use window::GlWindow;
use drawcalls::DrawCall;
use drawcalls::DrawMath;
use shaders::ShaderProgram;
use graphics::CoreRender;
use settings::Settings;
use font::GenericFont;
use model_data;

use cgmath;
use cgmath::Deg;
use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
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

#[derive(Clone)]
pub struct ModelInfo {
  location: String,
  texture: String,
}

#[derive(Clone, Debug)]
pub struct Model {
  vao: GLuint,
  ebo: GLuint,
  index_len: i32,
}

pub struct RawGl {
  ready: bool,
  shader_id: Vec<GLuint>,
  fonts: HashMap<String, GenericFont>,
  textures: HashMap<String, GLuint>,
  texture_paths: HashMap<String, String>,
  model_paths: HashMap<String, ModelInfo>,
  
  clear_colour: Vector4<f32>,
  
  projection_2d: Matrix4<f32>,
  vao_2d: GLuint,
  
  projection_3d: Matrix4<f32>,
  view: Matrix4<f32>,
  scale: Matrix4<f32>,
  models: HashMap<String, Model>,
  
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
    
    let proj_2d = cgmath::ortho(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
    let proj_3d = cgmath::perspective(cgmath::Deg(45.0)/*cgmath::Rad(consts::FRAC_PI_4)*/, { width as f32 / height as f32 }, 0.01, 100.0);
    
    let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0.0, 0.0, -1.0), cgmath::Point3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, 1.0, 0.0));
    let scale = cgmath::Matrix4::from_scale(0.01);
    
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

      projection_2d: proj_2d,
      vao_2d: 0,
      
      projection_3d: proj_3d,
      view: view,
      scale: scale,
      models: HashMap::with_capacity(10), 
      
      window: window,
    }
  }
  
  pub fn with_title(mut self, title: String) -> RawGl {
    self.window.set_title(title);
    self
  }
  
  fn store_f32_in_attribute_list(&mut self, attribute_number: i32, length: i32, data: Vec<GLfloat>) {
    let mut vbo: GLuint = 0;
    unsafe {
      // Create a Vertex Buffer Object and copy the vertex data to it
      gl::GenBuffers(1, &mut vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::BufferData(gl::ARRAY_BUFFER,
                     (data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                     mem::transmute(&data[0]),
                     gl::STATIC_DRAW);
      
      gl::EnableVertexAttribArray(attribute_number as u32);
      gl::VertexAttribPointer(attribute_number as GLuint,
                              length,
                              gl::FLOAT,
                              gl::FALSE as GLboolean,
                              0,
                              ptr::null());
    }
  }
  
  fn draw_3d(&mut self, draw: &DrawCall) {
    let model = self.models.get(draw.get_texture()).expect("Invalid model name").clone();

    unsafe {
      gl::UseProgram(self.shader_id[2]);
      
      gl::Enable(gl::DEPTH_TEST);
      // Accept fragment if it closer to the camera than the former one
      gl::DepthFunc(gl::LESS);
     
      let axis_x = Vector3::new(1.0, 0.0, 0.0).normalize();
      let axis_y = Vector3::new(0.0, 1.0, 0.0).normalize();
      let axis_z = Vector3::new(0.0, 0.0, 1.0).normalize();
      
      let rotation_x: Matrix4<f32> = Matrix4::from_axis_angle(axis_x, Deg(draw.get_x_rotation()));
      let rotation_y: Matrix4<f32> = Matrix4::from_axis_angle(axis_y, Deg(draw.get_y_rotation()));
      let rotation_z: Matrix4<f32> = Matrix4::from_axis_angle(axis_z, Deg(draw.get_z_rotation()));
      
      let world = cgmath::Matrix4::from_translation(draw.get_translation()) * (rotation_x*rotation_y*rotation_z);
      let view: Matrix4<f32> = (self.view * cgmath::Matrix4::from_scale(draw.get_size().x)).into();
      
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[2], CString::new("world").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&world[0]));
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[2], CString::new("view").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&view[0]));
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[2], CString::new("proj").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection_3d[0]));
      
      gl::BindVertexArray(model.vao);
      //gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, model.index_buffer);
      
      gl::DrawElements(
        gl::TRIANGLES,
        model.index_len,
        gl::UNSIGNED_INT,
        ptr::null()
      );
      
      gl::Disable(gl::DEPTH_TEST);
      
      gl::BindVertexArray(self.vao_2d);
      gl::UseProgram(0);
    }
  }
  
  fn draw_square(&self, object: &DrawCall) {
    let colour = object.get_colour();
    
    let model = DrawMath::calculate_texture_model(object.get_translation(), object.get_size());
    
    unsafe {
      gl::UseProgram(self.shader_id[0]);
      
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[0], CString::new("model").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&model[0]));
      
      gl::Uniform4f(gl::GetUniformLocation(self.shader_id[0], CString::new("colour").unwrap().as_ptr()), colour.x, colour.y, colour.z, colour.w);
      
      if object.get_colour().w == -1.0 {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, *self.textures.get(object.get_texture()).unwrap());
      }
      
      gl::DrawArrays(gl::TRIANGLES, 0, 6);
    }
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
      
      unsafe {
        gl::UseProgram(self.shader_id[1]);
        
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        
        gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[1], CString::new("model").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&model[0]));
        gl::Uniform4f(gl::GetUniformLocation(self.shader_id[1], CString::new("colour").unwrap().as_ptr()), colour.x, colour.y, colour.z, colour.w);
        gl::Uniform4f(gl::GetUniformLocation(self.shader_id[1], CString::new("letter_uv").unwrap().as_ptr()), letter_uv.x, letter_uv.y, letter_uv.z, letter_uv.w);
        gl::Uniform3f(gl::GetUniformLocation(self.shader_id[1], CString::new("outlineColour").unwrap().as_ptr()), outline.x, outline.y, outline.z);
        gl::Uniform4f(gl::GetUniformLocation(self.shader_id[1], CString::new("edge_width").unwrap().as_ptr()), edge_width.x, edge_width.y, edge_width.z, edge_width.w);
        
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, *self.textures.get(draw.get_texture()).unwrap());
        
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
      }
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
    let index: Vec<u16> = model.get_indicies();
/*    
    let mut verticies: Vec<f32> = Vec::with_capacity(10);
    let mut normals: Vec<f32> = Vec::with_capacity(10);
    let mut uv: Vec<f32> = Vec::with_capacity(10);
    //let mut indicies: Vec<u16> = Vec::with_capacity(10);
    
    for vert in vertex {
      for pos in vert.position.iter() {
        verticies.push(*pos);
      }
      
      for norm in vert.normal.iter() {
        normals.push(*norm);
      }
      
      for u_uv in vert.uv.iter() {
        uv.push(*u_uv);
      }
    }*/
    
    let mut vao: GLuint = 0;
    let mut vbo: GLuint = 0;
    let mut ebo: GLuint = 0;    
    
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
      gl::GenBuffers(1, &mut vbo);
      gl::GenBuffers(1, &mut ebo);
      
      gl::BindVertexArray(vao);
      
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      
      gl::BufferData(gl::ARRAY_BUFFER,
                     (vertex.len() * mem::size_of::<model_data::Vertex>()) as GLsizeiptr,
                     mem::transmute(&vertex[0]),
                     gl::STATIC_DRAW);
                     
      gl::UseProgram(self.shader_id[2]);
      
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
      gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                     (index.len() * mem::size_of::<u16>()) as GLsizeiptr,
                     mem::transmute(&index[0]),
                     gl::STATIC_DRAW);
      
      gl::EnableVertexAttribArray(0);
      gl::VertexAttribPointer(0,
                              3,
                              gl::FLOAT,
                              gl::FALSE as GLboolean,
                              8*mem::size_of::<model_data::Vertex>() as i32,
                              ptr::null());
      
      gl::EnableVertexAttribArray(1);
      gl::VertexAttribPointer(1,
                              3,
                              gl::FLOAT,
                              gl::FALSE as GLboolean,
                              8*mem::size_of::<model_data::Vertex>() as i32, 
                              ptr::null().offset(3*mem::size_of::<GLfloat>() as isize));
      
      gl::EnableVertexAttribArray(2);
      gl::VertexAttribPointer(2, 
                              2, 
                              gl::FLOAT, 
                              gl::FALSE as GLboolean, 
                              8*mem::size_of::<model_data::Vertex>() as i32, 
                              ptr::null().offset(6*mem::size_of::<GLfloat>() as isize));
      
      gl::BindVertexArray(0);
    }
   /* let mut vao: GLuint = 0;
    // Create Vertex Array Object
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);
    }
    
    self.store_f32_in_attribute_list(0, 3, verticies);
    self.store_f32_in_attribute_list(1, 3, normals);
    self.store_f32_in_attribute_list(2, 2, uv); 
    
    unsafe {
      // Create a Vertex Buffer Object and copy the vertex data to it
      gl::GenBuffers(1, &mut ebo);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
      gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                     (index.len() * mem::size_of::<u16>()) as GLsizeiptr,
                     mem::transmute(&index[0]),
                     gl::STATIC_DRAW);*/
      
     /* gl::VertexAttribPointer(3 as GLuint,
                              index.len(),
                              gl::FLOAT,
                              gl::FALSE as GLboolean,
                              0,
                              ptr::null());*/
    //}
    /*
    unsafe {
      gl::BindVertexArray(0);
    }*/
    
    let model = Model {
      vao: vao,
      ebo: ebo,
      index_len: index.len() as i32,
    };
    
    self.models.insert(reference, model);
    
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
      
      let image = image::open(&location).unwrap().to_rgba(); 
      let (width, height) = image.dimensions();
      let image_data = image.into_raw().clone();
     
      gl::TexImage2D(gl::TEXTURE_2D, 0,
                    gl::RGBA as GLint,
                    width as GLsizei,
                    height as GLsizei,
                    0, gl::RGBA, gl::UNSIGNED_BYTE,
                    mem::transmute(&image_data[0]));
      
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
    let v_string = String::from_utf8_lossy(include_bytes!("shaders/GlTexture.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("shaders/GlTexture.frag"));
    
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
    
    let vs = ShaderProgram::compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = ShaderProgram::compile_shader(f_src, gl::FRAGMENT_SHADER);
    let program = ShaderProgram::link_program(vs, fs);
    
    self.shader_id.push(program);
    
    unsafe {
      gl::DeleteShader(fs);
      gl::DeleteShader(vs);
    }
    
    let v_string = String::from_utf8_lossy(include_bytes!("shaders/GlText.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("shaders/GlText.frag"));
    
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
    
    let vs = ShaderProgram::compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = ShaderProgram::compile_shader(f_src, gl::FRAGMENT_SHADER);
    let program = ShaderProgram::link_program(vs, fs);
    
    self.shader_id.push(program);
    
    unsafe {
      gl::DeleteShader(fs);
      gl::DeleteShader(vs);
    }
    
    let v_string = String::from_utf8_lossy(include_bytes!("shaders/Gl3D.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("shaders/Gl3D.frag"));
    
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
    
    let vs = ShaderProgram::compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = ShaderProgram::compile_shader(f_src, gl::FRAGMENT_SHADER);
    let program = ShaderProgram::link_program(vs, fs);
    
    self.shader_id.push(program);
    
    unsafe {
      gl::DeleteShader(fs);
      gl::DeleteShader(vs);
    }
  }
  
  fn init(&mut self) {
    let square = vec!( 0.5,  0.5, 1.0, 0.0,
                                 -0.5,  0.5, 0.0, 0.0,
                                 -0.5, -0.5, 0.0, 1.0,
                                 
                                 -0.5, -0.5, 0.0, 1.0,
                                  0.5, -0.5, 1.0, 1.0,
                                  0.5,  0.5, 1.0, 0.0);
    
    unsafe {
      
      gl::UseProgram(self.shader_id[0]);
      // texture shader
      gl::Uniform1i(gl::GetUniformLocation(self.shader_id[0], CString::new("image").unwrap().as_ptr()), 0);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[0], CString::new("projection").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection_2d[0]));
      
      // Create Vertex Array Object
      gl::GenVertexArrays(1, &mut self.vao_2d);
      gl::BindVertexArray(self.vao_2d);
      
      // Create a Vertex Buffer Object and copy the vertex data to it      
      self.store_f32_in_attribute_list(0, 4, square);
      
      // Text shader
      gl::UseProgram(self.shader_id[1]);
      
      gl::Uniform1i(gl::GetUniformLocation(self.shader_id[1], CString::new("image").unwrap().as_ptr()), 0);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[1], CString::new("projection").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection_2d[0]));
 
      // 3D shader     
      gl::UseProgram(self.shader_id[2]);
 
      gl::Uniform1i(gl::GetUniformLocation(self.shader_id[2], CString::new("tex").unwrap().as_ptr()), 0);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[2], CString::new("proj").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection_3d[0]));
           
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
      
      gl::UseProgram(0);
    }
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
      //gl::ClearColor(0.2, 0.3, 0.3, 1.0);
      gl::ClearColor(self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
  }
  
  fn pre_draw(&mut self) {
    unsafe {
      gl::BindVertexArray(self.vao_2d);
      gl::EnableVertexAttribArray(0 as GLuint);
      gl::EnableVertexAttribArray(1 as GLuint);
      gl::EnableVertexAttribArray(2 as GLuint);
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
    unsafe {
      for shader in &self.shader_id {
        gl::DeleteProgram(*shader);
      }
      gl::DeleteVertexArrays(1, &self.vao_2d);
    }
  }
  
  fn swap_buffers(&mut self) {
    self.window.swap_buffers();
  }
  
  fn screen_resized(&mut self) {
    let dimensions = self.get_dimensions();
    self.projection_2d = cgmath::ortho(0.0, dimensions[0] as f32, 0.0, dimensions[1] as f32, -1.0, 1.0);
    self.projection_3d = cgmath::perspective(cgmath::Deg(45.0)/*cgmath::Rad(consts::FRAC_PI_4)*/, { dimensions[0] as f32 / dimensions[1] as f32 }, 0.01, 100.0);
    self.window.resize_screen(dimensions);
    
    unsafe {
      gl::Viewport(0, 0, dimensions[0] as i32, dimensions[1] as i32); 
      // texture shader
      gl::UseProgram(self.shader_id[0]);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[0], CString::new("projection").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection_2d[0]));
      
      // Text shader
      gl::UseProgram(self.shader_id[1]);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[1], CString::new("projection").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection_2d[0]));
      
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
  
  fn set_camera_location(&mut self, camera: Vector3<f32>, camera_rot: Vector2<f32>){}
}





