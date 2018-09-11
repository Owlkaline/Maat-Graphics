use gl;
use gl::types::*;

use cgmath;
use cgmath::Matrix4;

use std::ffi::CString;

use opengl::fbo::ShaderData;
use opengl::fbo::ShaderFunctions;
use opengl::shader::link_program;
use opengl::shader::compile_shader;

pub struct TextureShader {
  shader: ShaderData,
}

impl TextureShader {
  pub fn new() -> TextureShader {
    let v_string = String::from_utf8_lossy(include_bytes!("../shaders/glsl/GlTexture.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("../shaders/glsl/GlTexture.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    TextureShader {
      shader: ShaderData::new(shader_id),
    }
  }
  
  pub fn create_projection(width: f32, height: f32) -> Matrix4<f32> {
    cgmath::ortho(0.0, width, 0.0, height, -1.0, 1.0)
  }
}

impl ShaderFunctions for TextureShader {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}

pub struct TextShader {
  shader: ShaderData,
}

impl TextShader {
  pub fn new() -> TextShader {
    let v_string = String::from_utf8_lossy(include_bytes!("../shaders/glsl/GlText.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("../shaders/glsl/GlText.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    TextShader {
      shader: ShaderData::new(shader_id),
    }
  }
}

impl ShaderFunctions for TextShader {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}


