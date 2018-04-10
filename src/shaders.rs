use gl;
use gl::types::*;

use std::vec::Vec;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;

use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix3;
use cgmath::Matrix4;

pub struct ShaderTextureInstanced {
  shader: ShaderData,
}

pub struct ShaderTexture {
  shader: ShaderData,
}

pub struct ShaderText {
  shader: ShaderData,
}

pub struct Shader3D {
  shader: ShaderData,
}

impl Shader3D {
  pub fn new() -> Shader3D {
    let v_string = String::from_utf8_lossy(include_bytes!("shaders/Gl3D.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("shaders/Gl3D.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = ShaderProgram::compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = ShaderProgram::compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = ShaderProgram::link_program(vs, fs);
    
    Shader3D {
      shader: 
        ShaderData {
          id: shader_id,
        }
    }
  }
}

impl ShaderText {
  pub fn new() -> ShaderText {
    let v_string = String::from_utf8_lossy(include_bytes!("shaders/GlText.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("shaders/GlText.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = ShaderProgram::compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = ShaderProgram::compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = ShaderProgram::link_program(vs, fs);
    
    ShaderText {
      shader: 
        ShaderData {
          id: shader_id,
        }
    }
  }
}

impl ShaderTexture {
  pub fn new() -> ShaderTexture {
    let v_string = String::from_utf8_lossy(include_bytes!("shaders/GlTexture.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("shaders/GlTexture.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = ShaderProgram::compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = ShaderProgram::compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = ShaderProgram::link_program(vs, fs);
    
    ShaderTexture {
      shader: 
        ShaderData {
          id: shader_id,
        }
    }
  }
}

impl ShaderTextureInstanced {
  pub fn new() -> ShaderTexture {
    let v_string = String::from_utf8_lossy(include_bytes!("shaders/GlTextureInstanced.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("shaders/GlTextureInstanced.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = ShaderProgram::compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = ShaderProgram::compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = ShaderProgram::link_program(vs, fs);
    
    ShaderTexture {
      shader: 
        ShaderData {
          id: shader_id,
        }
    }
  }
}

impl ShaderFunctions for ShaderTexture {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}

impl ShaderFunctions for ShaderTextureInstanced {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}

impl ShaderFunctions for ShaderText {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}

impl ShaderFunctions for Shader3D {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}

pub struct ShaderData {
  id: GLuint,
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

pub struct ShaderProgram {}

impl ShaderProgram {
  pub fn compile_shader(c_str: CString, ty: GLenum) -> GLuint {
    let shader;
    
    println!("Compiling shader");
    
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader,
                                 len,
                                 ptr::null_mut(),
                                 buf.as_mut_ptr() as *mut GLchar);
            panic!("{}",
                   str::from_utf8(&buf)
                       .ok()
                       .expect("ShaderInfoLog not valid utf8"));
        }
    }
    shader
  }

  pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    println!("Linking program\n");
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program,
                                  len,
                                  ptr::null_mut(),
                                  buf.as_mut_ptr() as *mut GLchar);
            panic!("{}",
                   str::from_utf8(&buf)
                       .ok()
                       .expect("ProgramInfoLog not valid utf8"));
        }
        program
    }
  }
}

// TODO in future versions
/*
pub struct Fbo {
  frame_buffer_id: GLuint,
  texture: GLuint,
  width: i32,
  height: i32,
}

impl Fbo {
  pub fn new(width: i32, height: i32) -> Fbo {
    let mut frame_buffer: GLuint = 0;
    let mut texture: GLuint = 0;
    unsafe {
      gl::GenFramebuffers(1, &mut frame_buffer);
      gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer);
      gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
      gl::GenTextures(1, &mut texture);
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
    
    Fbo {
      frame_buffer_id: frame_buffer,
      texture: texture,
      width: width,
      height: height,
    }
  }
  
  pub fn bind(&mut self) {
    unsafe {
      self.cleanup();
      gl::BindTexture(gl::TEXTURE_2D, 0);
      gl::GenFramebuffers(1, &mut self.frame_buffer_id);
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer_id);
      gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
      gl::Viewport(0, 0, self.width, self.height);
      
      gl::GenTextures(1, &mut self.texture);
      gl::BindTexture(gl::TEXTURE_2D, self.texture);
      gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, self.width, self.height, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
      gl::TexImage2D(gl::TEXTURE_2D, 0,
                     gl::RGB as GLint,
                     self.width as GLsizei,
                     self.height as GLsizei,
                     0, gl::RGB, gl::UNSIGNED_BYTE,
                     ptr::null());
      gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.texture, 0);
    }
  }
  
  pub fn unbind(&self) {
    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
  }
  
  pub fn get_texture(&self) -> GLuint {
    self.texture
  }
  
  pub fn cleanup(&mut self) {
    unsafe {
      gl::DeleteFramebuffers(1, &mut self.frame_buffer_id);
      gl::DeleteTextures(1, &mut self.texture);
    }
  }
}*/
