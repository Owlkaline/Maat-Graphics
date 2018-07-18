use gl;
use gl::types::*;

use drawcalls::DrawCall;

use opengl::opengl_helper;

use winit::dpi::LogicalSize;

use std::ffi::CString;
use std::mem;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix3;
use cgmath::Matrix4;

pub struct Fbo {
  ms_framebuffer: GLuint,
  framebuffer: GLuint,
  
  renderbuffer_depth: GLuint,
  renderbuffer_resolvedepth: GLuint,
  
  ms_colour_attachments: Vec<GLuint>,
  colour_attachments: Vec<GLuint>,
  
  hdr: bool,
  samples: i32,
  dimensions: [i32; 2],
}

impl Fbo {
  pub fn new(samples: i32, num_colour_attachments: usize, hdr: bool,  width: i32, height: i32) -> Fbo {
    debug_assert!(num_colour_attachments >= 1);
    debug_assert!(samples >= 1);
    debug_assert!(width > 0 || height > 0);
    
    let mut num_ms_colour_attachments = num_colour_attachments;
    if samples < 1 {
      num_ms_colour_attachments = 0;
    }
    
    let mut ms_colour_attachments = Vec::with_capacity(num_ms_colour_attachments);
    for i in 0..num_colour_attachments {
      ms_colour_attachments.push(0);
    }
    
    let mut colour_attachments = Vec::with_capacity(num_colour_attachments);
    for i in 0..num_colour_attachments {
      colour_attachments.push(0);
    }
    
    Fbo {
      ms_framebuffer: 0,
      framebuffer: 0,
      
      renderbuffer_depth: 0,
      renderbuffer_resolvedepth: 0,
      
      ms_colour_attachments: ms_colour_attachments,
      colour_attachments: colour_attachments,
      
      hdr: hdr,
      samples: samples,
      dimensions: [width, height],
    }
  }
  
  pub fn clean(&mut self) {
    unsafe {
      if self.samples > 1 {
        gl::DeleteFramebuffers(1, &mut self.ms_framebuffer);
        gl::DeleteRenderbuffers(1, &mut self.renderbuffer_depth);
        for i in 0..self.ms_colour_attachments.len() {
          gl::DeleteTextures(1, &mut self.ms_colour_attachments[i]);
        }
      }
      
      gl::DeleteFramebuffers(1, &mut self.framebuffer);
      gl::DeleteRenderbuffers(1, &mut self.renderbuffer_resolvedepth);
      for i in 0..self.colour_attachments.len() {
        gl::DeleteTextures(1, &mut self.colour_attachments[i]);
      }
    }
  }
  
  pub fn init(&mut self) {
    let mut colour_format = gl::RGBA;
    let mut byte_type = gl::UNSIGNED_BYTE;
    if self.hdr {
      colour_format = gl::RGBA16F;
      byte_type = gl::FLOAT;
    }
    
    unsafe {
      if self.samples > 1 {
        // Generate multisample objects
        gl::GenFramebuffers(1, &mut self.ms_framebuffer);
        gl::GenRenderbuffers(1, &mut self.renderbuffer_depth);
        for i in 0..self.ms_colour_attachments.len() {
          gl::GenTextures(1, &mut self.ms_colour_attachments[i]);
        }
        
        // Depth renderbuffer
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.renderbuffer_depth);
        gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, self.samples, gl::DEPTH_COMPONENT, self.dimensions[0], self.dimensions[1]);
        
        // Multisample texture attachment
        for i in 0..self.ms_colour_attachments.len() {
          gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, self.ms_colour_attachments[i]);
          gl::TexImage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE, self.samples, colour_format as GLuint, self.dimensions[0], self.dimensions[1], gl::TRUE);
        }
        gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);
        
        // Multisample Framebuffer
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.ms_framebuffer);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, self.renderbuffer_depth);
        
        for i in 0..self.ms_colour_attachments.len() {
          gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as u32, gl::TEXTURE_2D_MULTISAMPLE, self.ms_colour_attachments[i], 0);
        }
        
        opengl_helper::check_framebufferstatus(self.ms_framebuffer);
      }
      
      gl::GenFramebuffers(1, &mut self.framebuffer);
      gl::GenRenderbuffers(1, &mut self.renderbuffer_resolvedepth);
      
      // New colour attachment gen
      for i in 0..self.colour_attachments.len() {
        gl::GenTextures(1, &mut self.colour_attachments[i]);
      }
      
      // Resolve depth renderbuffer
      gl::BindRenderbuffer(gl::RENDERBUFFER, self.renderbuffer_resolvedepth);
      gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT as GLuint, self.dimensions[0], self.dimensions[1]);
      
      // Resolve Texture attachment
      for i in 0..self.colour_attachments.len() {
        gl::BindTexture(gl::TEXTURE_2D, self.colour_attachments[i]);
        gl::TexImage2D(gl::TEXTURE_2D, 0, colour_format as GLint, self.dimensions[0], self.dimensions[1], 0, gl::RGBA, byte_type, mem::transmute(0i64));
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
      }
      gl::BindTexture(gl::TEXTURE_2D, 0);
      
      // Standard Framebuffer
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
      gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, self.renderbuffer_resolvedepth);
      
      for i in 0..self.colour_attachments.len() {
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as u32, gl::TEXTURE_2D, self.colour_attachments[i], 0);
      }
      
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
      
      opengl_helper::check_framebufferstatus(self.framebuffer);
    }
  }
  
  pub fn bind(&self) {
    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, 0);
     // let mut bufs: Vec<u32> = Vec::new();
     // for i in 0..self.colour_attachments.len() {
     //   bufs.push(gl::COLOR_ATTACHMENT0 + i as u32);
     // }
      
      if self.samples > 1 {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.ms_framebuffer);
      } else {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
      }
      
     // gl::DrawBuffers(bufs.len() as i32, mem::transmute(&bufs[0]));
    }
  }
  
  pub fn resolve_multisample(&self) {
    if self.samples > 1 {
      unsafe {
       // let mut bufs: Vec<u32> = Vec::new();
      //  for i in 0..self.colour_attachments.len() {
      //    bufs.push(gl::COLOR_ATTACHMENT0 + i as u32);
     //   }
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.ms_framebuffer);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.framebuffer);
       // gl::DrawBuffers(bufs.len() as i32, mem::transmute(&bufs[0]));
        gl::BlitFramebuffer(0, 0, self.dimensions[0], self.dimensions[1], 0, 0, self.dimensions[0], self.dimensions[1], gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT, gl::NEAREST);
      }
    }
  }
  
  pub fn bind_default(&self) {
    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
  }
  
  pub fn draw_screen_texture(&self, x: f32, y: f32, width: f32, height: f32) -> DrawCall {
    DrawCall::new_draw(x, y, 0.0)
              .with_scale(-width, -height)
  }
  
  pub fn get_screen_texture(&self, attachment_index: usize) -> GLuint {
    self.colour_attachments[attachment_index]
  }
  
  pub fn resize(&mut self, dim: LogicalSize) {
    self.dimensions = [dim.width as i32, dim.height as i32];
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
  
  fn set_vec2(&self, name: String, value: Vector2<GLfloat>) {
    unsafe {
      gl::Uniform2f(gl::GetUniformLocation(self.data().id, CString::new(name).unwrap().as_ptr()), value.x, value.y);
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
