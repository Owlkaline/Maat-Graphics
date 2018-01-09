use window::GlWindow;
use drawcalls::DrawCall;
use drawcalls::DrawMath;
use shaders::ShaderProgram;
use graphics::CoreRender;
use settings::Settings;
use font::GenericFont;

use cgmath;
use cgmath::Matrix4;

use image;
use winit;

use gl;
use gl::types::*;

use std::ptr;
use std::mem;
use std::time;
use std::ffi::CString;
use std::collections::HashMap;

pub struct RawGl {
  ready: bool,
  shader_id: Vec<GLuint>,
  fonts: HashMap<String, GenericFont>,
  textures: HashMap<String, GLuint>,
  texture_paths: HashMap<String, String>,
  projection: Matrix4<f32>,
  vao: GLuint,
  pub window: GlWindow,
}

impl RawGl {
  pub fn new() -> RawGl {
    let mut settings = Settings::load();
    let width = settings.get_resolution()[0];
    let height = settings.get_resolution()[1];
    let min_width = settings.get_minimum_resolution()[0];
    let min_height = settings.get_minimum_resolution()[1];
    let fullscreen = settings.is_fullscreen();
    
    let window = GlWindow::new(width, height, min_width, min_height, fullscreen);
    
    let proj = cgmath::ortho(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
    
    unsafe {
      gl::Viewport(0, 0, (width as i32 *2) as i32, (height as i32 *2) as i32);
    }
    
    RawGl {
      ready: false,
      shader_id: Vec::with_capacity(2),
      fonts: HashMap::with_capacity(10),
      textures: HashMap::with_capacity(10),
      texture_paths: HashMap::with_capacity(10),
      projection: proj,
      vao: 0,
      window: window,
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
    
  }
  
  fn add_model(&mut self, reference: String, location: String, texture: String) {
    
  }
  
  fn load_model(&mut self, reference: String, location: String, texture: String) {
    
  }
 
  fn pre_load_texture(&mut self, reference: String, location: String) {
    self.load_texture(reference, location);
  }
  
  fn add_texture(&mut self, reference: String, location: String) {
    self.texture_paths.insert(reference, location);
  }
 
  fn load_texture(&mut self, reference: String, location: String) {
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
  
  fn pre_load_font(&mut self, reference: String, font: &[u8], font_texture: String) {
    self.load_font(reference.clone(), font);
    self.pre_load_texture(reference, font_texture);
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
  }
  
  fn init(&mut self) {
    let square: [GLfloat; 24] = [ 0.5,  0.5, 1.0, 0.0,
                                 -0.5,  0.5, 0.0, 0.0,
                                 -0.5, -0.5, 0.0, 1.0,
                                 
                                 -0.5, -0.5, 0.0, 1.0,
                                  0.5, -0.5, 1.0, 1.0,
                                  0.5,  0.5, 1.0, 0.0];
    
    let mut vbo = 0;
    
    unsafe {
      
      gl::UseProgram(self.shader_id[0]);
      // texture shader
      gl::Uniform1i(gl::GetUniformLocation(self.shader_id[0], CString::new("image").unwrap().as_ptr()), 0);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[0], CString::new("projection").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection[0]));
      
      // Create Vertex Array Object
      gl::GenVertexArrays(1, &mut self.vao);
      gl::BindVertexArray(self.vao);
      
      // Create a Vertex Buffer Object and copy the vertex data to it
      gl::GenBuffers(1, &mut vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::BufferData(gl::ARRAY_BUFFER,
                     (square.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                     mem::transmute(&square[0]),
                     gl::STATIC_DRAW);
      
      gl::VertexAttribPointer(0 as GLuint,
                              4,
                              gl::FLOAT,
                              gl::FALSE as GLboolean,
                              0,
                              ptr::null());
      gl::EnableVertexAttribArray(0 as GLuint);
      
      // Text shader
      gl::UseProgram(self.shader_id[1]);
      
      gl::Uniform1i(gl::GetUniformLocation(self.shader_id[1], CString::new("image").unwrap().as_ptr()), 0);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[1], CString::new("projection").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection[0]));
      
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
      
      gl::UseProgram(0);
    }
  }
  
  fn dynamic_load(&mut self) {
    let mut delta_time;
    let frame_start_time = time::Instant::now();
  
    let mut loaded_a_image = false;
    let cloned_paths = self.texture_paths.clone();
    
    for (reference, path) in &cloned_paths {
      self.load_texture(reference.clone(), path.clone());
      
      self.texture_paths.remove(reference);
      loaded_a_image = true;
      
      delta_time = frame_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
      if (delta_time*1000f64) > 9.0 {
        break;
      } 
    }
    
    if !loaded_a_image {
      self.ready = true;
    }
  }
  
  fn clear_screen(&mut self) {
    unsafe {
      gl::ClearColor(0.2, 0.3, 0.3, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);
    }
  }
  
  fn pre_draw(&mut self) {
    unsafe {
      gl::BindVertexArray(self.vao);
    }
  }
  
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
    for draw in draw_calls {
      if draw.get_text() != "" {
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
    }
  }
  
  fn clean(&self) {
    unsafe {
      for shader in &self.shader_id {
        gl::DeleteProgram(*shader);
      }
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
  
  fn swap_buffers(&mut self) {
    self.window.swap_buffers();
  }
  
  fn screen_resized(&mut self) {
    let dimensions = self.get_dimensions();
    self.projection = cgmath::ortho(0.0, dimensions[0] as f32, 0.0, dimensions[1] as f32, -1.0, 1.0);
    self.window.resize_screen(dimensions);
    
    unsafe {
      gl::Viewport(0, 0, dimensions[0] as i32, dimensions[1] as i32); 
      // texture shader
      gl::UseProgram(self.shader_id[0]);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[0], CString::new("projection").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection[0]));
      
      // Text shader
      gl::UseProgram(self.shader_id[1]);
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.shader_id[1], CString::new("projection").unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&self.projection[0]));
      
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
  
  fn show_cursor(&mut self){}
  fn hide_cursor(&mut self){}
}





