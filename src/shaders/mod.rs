use gl;
use gl::types::*;

use std::vec::Vec;
use std::ffi::CString;
use std::ptr;
use std::str;

pub mod traits;

use shaders::traits::ShaderData;
use shaders::traits::ShaderFunctions;

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

pub struct ShaderTextureInstanced {
  shader: ShaderData,
}

pub struct ShaderTexture {
  shader: ShaderData,
}

pub struct ShaderText {
  shader: ShaderData,
}

pub struct ShaderBloom {
  shader: ShaderData,
}

pub struct ShaderBlur {
  shader: ShaderData,
}

pub struct ShaderFinal {
  shader: ShaderData,
}

pub struct Shader3D {
  shader: ShaderData,
}

impl Shader3D {
  pub fn new() -> Shader3D {
    let v_string = String::from_utf8_lossy(include_bytes!("glsl/Gl3D.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("glsl/Gl3D.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    Shader3D {
      shader: ShaderData::new(shader_id),
    }
  }
}

impl ShaderFinal {
  pub fn new() -> ShaderFinal {
    let v_string = String::from_utf8_lossy(include_bytes!("glsl/GlPostFinal.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("glsl/GlPostFinal.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    ShaderFinal {
      shader: ShaderData::new(shader_id),
    }
  }
}

impl ShaderBlur {
  pub fn new() -> ShaderBlur {
    let v_string = String::from_utf8_lossy(include_bytes!("glsl/GlPostBlur.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("glsl/GlPostBlur.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    ShaderBlur {
      shader: ShaderData::new(shader_id),
    }
  }
}

impl ShaderBloom {
  pub fn new() -> ShaderBlur {
    let v_string = String::from_utf8_lossy(include_bytes!("glsl/GlPostBloom.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("glsl/GlPostBloom.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    ShaderBlur {
      shader: ShaderData::new(shader_id),
    }
  }
}

impl ShaderText {
  pub fn new() -> ShaderText {
    let v_string = String::from_utf8_lossy(include_bytes!("glsl/GlText.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("glsl/GlText.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    ShaderText {
      shader: ShaderData::new(shader_id),
    }
  }
}

impl ShaderTexture {
  pub fn new() -> ShaderTexture {
    let v_string = String::from_utf8_lossy(include_bytes!("glsl/GlTexture.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("glsl/GlTexture.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    ShaderTexture {
      shader: ShaderData::new(shader_id),
    }
  }
}

impl ShaderTextureInstanced {
  pub fn new() -> ShaderTexture {
    let v_string = String::from_utf8_lossy(include_bytes!("glsl/GlTextureInstanced.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("glsl/GlTextureInstanced.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    ShaderTexture {
      shader: ShaderData::new(shader_id),
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

impl ShaderFunctions for ShaderBloom {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}

impl ShaderFunctions for ShaderBlur {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}

impl ShaderFunctions for ShaderFinal {
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
