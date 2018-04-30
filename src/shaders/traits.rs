use gl;
use gl::types::*;

use drawcalls::DrawCall;

use std::ffi::CString;
use std::mem;
use std::ptr;

use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix3;
use cgmath::Matrix4;

pub struct Fbo {
  ms_framebuffer: GLuint,
  framebuffer: GLuint,
  renderbuffer: GLuint,
  
  multisampled_colour_texture: GLuint,
  screen_texture: GLuint,
  
  samples: i32,
  depth_enabled: bool,
  dimensions: [i32; 2],
}

impl Fbo {
  pub fn new(samples: i32, width: i32, height: i32) -> Fbo {
    Fbo {
      ms_framebuffer: 0,
      framebuffer: 0,
      renderbuffer: 0,
      
      multisampled_colour_texture: 0,
      screen_texture: 0,
      
      samples: samples,
      depth_enabled: false,
      dimensions: [width, height],
    }
  }
  
  pub fn clean(&mut self) {
    unsafe {
      gl::DeleteFramebuffers(1, &mut self.ms_framebuffer);
      gl::DeleteFramebuffers(1, &mut self.framebuffer);
      gl::DeleteRenderbuffers(1, &mut self.renderbuffer);
      gl::DeleteTextures(1, &mut self.screen_texture);
    }
  }
  
  pub fn is_3d(mut self) -> Self {
    self.depth_enabled = true;
    self
  }
  
  pub fn init(&mut self) {
    unsafe {
      gl::GenFramebuffers(1, &mut self.ms_framebuffer);
      gl::GenFramebuffers(1, &mut self.framebuffer);
      gl::GenRenderbuffers(1, &mut self.renderbuffer);
      
      gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.ms_framebuffer);
      gl::BindRenderbuffer(gl::RENDERBUFFER, self.renderbuffer);
      gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::RGB, self.dimensions[0], self.dimensions[1]);
      gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, self.renderbuffer);
      
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
      
      gl::GenTextures(1, &mut self.screen_texture);
      gl::BindTexture(gl::TEXTURE_2D, self.screen_texture);
      gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, self.dimensions[0], self.dimensions[1], 0, gl::RGB, gl::UNSIGNED_BYTE, mem::transmute(0i64));
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
      gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.screen_texture, 0);
      
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
  }
  
  pub fn bind(&self) {
    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, 0);
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.ms_framebuffer);
    }
  }
  
  pub fn blit_to_post(&self) {
    unsafe {
      gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.ms_framebuffer);
      gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.framebuffer);
      gl::BlitFramebuffer(self.dimensions[0], 0, 0, self.dimensions[1], self.dimensions[0], self.dimensions[1], 0, 0, gl::COLOR_BUFFER_BIT, gl::NEAREST);
    }
  }
  
  pub fn bind_default(&self) {
    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
  }
  
  pub fn draw_screen_texture(&self, x: f32, y: f32, width: f32, height: f32) -> DrawCall {
    DrawCall::new_draw(x, y, 0.0)
              .with_scale(width, height)
  }
  
  pub fn get_screen_texture(&self) -> GLuint {
    self.screen_texture
  }
  
  pub fn resize(&mut self, width: f32, height: f32) {
    println!("resized: {}, {}", width, height);
    self.dimensions = [width as i32, height as i32];
    self.clean();
    self.init();
  }
}

pub struct ShaderData {
  id: GLuint,
}

impl ShaderData {
  pub fn new(shader_id: GLuint) -> ShaderData {
    ShaderData {
      id: shader_id
    }
  }
}

pub trait ShaderFunctions {
  fn data(&self) -> &ShaderData;
  fn mut_data(&mut self) ->&mut ShaderData;
  
  fn get_id(&self) -> GLuint {
    self.data().id
  }
  
  fn Use(&self) {
    unsafe {
      gl::UseProgram(self.data().id);
    }
  }
  
  fn set_bool(&self, name: String, value: GLboolean) {
    unsafe {
      gl::Uniform1i(gl::GetUniformLocation(self.data().id, CString::new(name).unwrap().as_ptr()), value as GLint);
    }
  }
  
  fn set_int(&self, name: String, value: GLint) {
    unsafe {
      gl::Uniform1i(gl::GetUniformLocation(self.data().id, CString::new(name).unwrap().as_ptr()), value);
    }
  }
  
  fn set_float(&self, name: String, value: GLfloat) {
    unsafe {
      gl::Uniform1f(gl::GetUniformLocation(self.data().id, CString::new(name).unwrap().as_ptr()), value);
    }
  }
  
  fn set_vec3(&self, name: String, value: Vector3<GLfloat>) {
    unsafe {
      gl::Uniform3f(gl::GetUniformLocation(self.data().id, CString::new(name).unwrap().as_ptr()), value.x, value.y, value.z);
    }
  }
  
  fn set_vec4(&self, name: String, value: Vector4<GLfloat>) {
    unsafe {
      gl::Uniform4f(gl::GetUniformLocation(self.data().id, CString::new(name).unwrap().as_ptr()), value.x, value.y, value.z, value.w);
    }
  }
  
  fn set_mat3(&self, name: String, value: Matrix3<GLfloat>) {
    unsafe {
      gl::UniformMatrix3fv(gl::GetUniformLocation(self.data().id, CString::new(name).unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&value[0]));
    }
  }
  
  fn set_mat4(&self, name: String, value: Matrix4<GLfloat>) {
    unsafe {
      gl::UniformMatrix4fv(gl::GetUniformLocation(self.data().id, CString::new(name).unwrap().as_ptr()), 1, gl::FALSE, mem::transmute(&value[0]));
    }
  }
}
