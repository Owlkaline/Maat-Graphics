use gl;
use gl::types::*;

use std::ptr;
use std::mem;

pub const INSTANCE_DATA_LENGTH: usize = 22;

#[derive(Clone)]
pub struct Vao {
  vao: GLuint,
  vbo: GLuint,
  ebo: GLuint,
  num_vertices: GLint,
  num_indices: GLint,
  attrib: Vec<GLuint>,
}

pub struct Vao3D {
  vao: GLuint,
  vbo: GLuint,
  ebo: GLuint,
  num_vertices: GLint,
  num_indices: GLint,
  attrib: Vec<GLuint>,
}

pub struct InstancedVao {
  vao: GLuint,
  vbo: [GLuint; 2],
  ebo: GLuint,
  num_vertices: GLint,
  num_indices: GLint,
  attrib: Vec<GLuint>,
  vbo_data: Vec<GLfloat>,
  max_instances: GLuint,
}

impl Vao3D {
  pub fn new() -> Vao3D {
    let mut vao: GLuint = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
    }
    
    Vao3D {
      vao: vao,
      vbo: 0,
      ebo: 0,
      num_vertices: 0,
      num_indices: 0,
      attrib: Vec::new(),
    }
  }
  
  pub fn cleanup(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &mut self.vbo);
      gl::DeleteBuffers(1, &mut self.ebo);
      gl::DeleteVertexArrays(1, &mut self.vao);
      gl::GenVertexArrays(1, &mut self.vao);
    }
    
    self.attrib.clear();
    self.num_vertices = 0;
    self.num_indices = 0;
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
    self.vbo = vbo;
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
      gl::EnableVertexAttribArray(location);
    }
  }
  
  pub fn activate_texture(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE0 + i);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
  
  pub fn update_vbo(&mut self, vertices: Vec<GLfloat>) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
      gl::BufferSubData(gl::ARRAY_BUFFER, 0, 
                        (mem::size_of::<GLuint>()*vertices.len()) as isize,
                        mem::transmute(&vertices[0]));
    }
  }
  
  pub fn update_ebo(&mut self, indices: Vec<GLuint>) {
    unsafe {
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
      gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, 0,
                     (mem::size_of::<GLuint>()*indices.len()) as isize,
                     mem::transmute(&indices[0]));
    }
  }
}

impl Vao {
  pub fn new() -> Vao {
    let mut vao: GLuint = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
    }
    
    Vao {
      vao: vao,
      vbo: 0,
      ebo: 0,
      num_vertices: 0,
      num_indices: 0,
      attrib: Vec::new(),
    }
  }
  
  pub fn cleanup(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &mut self.vbo);
      gl::DeleteBuffers(1, &mut self.ebo);
      gl::DeleteVertexArrays(1, &mut self.vao);
      gl::GenVertexArrays(1, &mut self.vao);
    }
    
    self.attrib.clear();
    self.num_vertices = 0;
    self.num_indices = 0;
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
    self.vbo = vbo;
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
      gl::EnableVertexAttribArray(location);
    }
  }
  
  pub fn activate_texture(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE0 + i);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
  
  pub fn activate_texture1(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE1);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
  
  pub fn update_vbo(&mut self, vertices: Vec<GLfloat>) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
      gl::BufferSubData(gl::ARRAY_BUFFER, 0, 
                        (mem::size_of::<GLuint>()*vertices.len()) as isize,
                        mem::transmute(&vertices[0]));
    }
  }
  
  pub fn update_ebo(&mut self, indices: Vec<GLuint>) {
    unsafe {
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
      gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, 0,
                     (mem::size_of::<GLuint>()*indices.len()) as isize,
                     mem::transmute(&indices[0]));
    }
  }
}

impl InstancedVao {
  pub fn new(max_instances: i32) -> InstancedVao {
    let mut vao: GLuint = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
    }
    
    InstancedVao {
      vao: vao,
      vbo: [0, 0],
      ebo: 0,
      num_vertices: 0,
      num_indices: 0,
      attrib: Vec::new(),
      vbo_data: Vec::new(),
      max_instances: max_instances as GLuint,
    }
  }
  
  pub fn cleanup(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &mut self.vbo[0]);
      gl::DeleteBuffers(1, &mut self.vbo[1]);
      gl::DeleteBuffers(1, &mut self.ebo);
      gl::DeleteVertexArrays(1, &mut self.vao);
      gl::GenVertexArrays(1, &mut self.vao);
    }
    
    self.attrib.clear();
    self.num_vertices = 0;
    self.num_indices = 0;
  }
  
  pub fn update_vbodata(&self, size: usize, new_data: Vec<GLfloat>) {
    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo[1]);
      gl::BufferSubData(gl::ARRAY_BUFFER, 0, (mem::size_of::<GLfloat>()*size*INSTANCE_DATA_LENGTH) as isize, mem::transmute(&new_data[0]));
    }
  }
  
  pub fn draw_indexed_instanced(&self, size: usize, draw_type: GLuint) {
    self.bind();
    self.bind_ebo();
    unsafe {
      gl::DrawElementsInstanced(draw_type, self.num_indices, gl::UNSIGNED_INT, ptr::null(), size as GLint);
    }
    self.unbind();
  }
  
  pub fn unbind(&self) {
    unsafe {
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
      self.set_vertex_attrib(0, 2, 4, 0);
      self.set_vertex_attrib(1, 2, 4, 2);
      self.num_vertices = (vertices.len()/3) as GLint;
    }
    
    let mut vbo_data: GLuint = 0;
    unsafe {
      gl::GenBuffers(1, &mut vbo_data);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo_data);
      gl::BufferData(gl::ARRAY_BUFFER,
                     (mem::size_of::<GLfloat>()*self.max_instances as usize*INSTANCE_DATA_LENGTH as usize) as isize,
                     ptr::null(),
                     draw_type);
    }
    
    self.set_vertex_instanced_attrib(2, 4, INSTANCE_DATA_LENGTH, 0);
    self.set_vertex_instanced_attrib(3, 4, INSTANCE_DATA_LENGTH, 4);
    self.set_vertex_instanced_attrib(4, 4, INSTANCE_DATA_LENGTH, 8);
    self.set_vertex_instanced_attrib(5, 4, INSTANCE_DATA_LENGTH, 12);
    self.set_vertex_instanced_attrib(6, 4, INSTANCE_DATA_LENGTH, 16);
    self.set_vertex_instanced_attrib(7, 2, INSTANCE_DATA_LENGTH, 20);
    
    self.vbo = [vbo, vbo_data];
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
  
  pub fn set_vertex_instanced_attrib(&mut self, location: GLuint, size: GLint, total_size: usize, offset: usize) {
    unsafe {
      gl::VertexAttribPointer(location, size, gl::FLOAT, gl::FALSE, 
                              (total_size * mem::size_of::<GLfloat>()) as i32,
                              ptr::null().offset((offset * mem::size_of::<GLfloat>()) as isize));
      gl::VertexAttribDivisor(location, 1);
      self.attrib.push(location);
      gl::EnableVertexAttribArray(location);
    }
  }
  
  pub fn activate_texture(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE0 + i);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
  
  pub fn activate_texture1(&self, i: u32, texture: GLuint) {
    unsafe {
      gl::ActiveTexture(gl::TEXTURE1);
      gl::BindTexture(gl::TEXTURE_2D, texture);
    }
  }
}

