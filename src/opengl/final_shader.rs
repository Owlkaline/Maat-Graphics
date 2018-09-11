use gl;
use gl::types::*;

use std::ffi::CString;

use opengl::fbo::ShaderData;
use opengl::fbo::ShaderFunctions;
use opengl::shader::link_program;
use opengl::shader::compile_shader;

pub struct FinalShader {
  shader: ShaderData,
}

impl FinalShader {
  pub fn new() -> FinalShader {
    let v_string = String::from_utf8_lossy(include_bytes!("../shaders/glsl/GlFinal.vert"));
    let f_string = String::from_utf8_lossy(include_bytes!("../shaders/glsl/GlFinal.frag"));
  
    let v_src = CString::new(v_string.as_bytes()).unwrap();
    let f_src = CString::new(f_string.as_bytes()).unwrap();
  
    let vs = compile_shader(v_src, gl::VERTEX_SHADER);
    let fs = compile_shader(f_src, gl::FRAGMENT_SHADER);
    let shader_id = link_program(vs, fs);
    
    FinalShader {
      shader: ShaderData::new(shader_id),
    }
  }
}

impl ShaderFunctions for FinalShader {
  fn data(&self) -> &ShaderData {
    &self.shader
  }
  
  fn mut_data(&mut self) ->&mut ShaderData {
    &mut self.shader
  }
}
