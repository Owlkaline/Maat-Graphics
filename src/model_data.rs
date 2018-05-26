use std::str;
use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;

use cgmath::Vector4;
use cgmath::Matrix4;

use opengex_parser::OpengexPaser;

#[derive(Copy, Clone)]
pub struct Vertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub uv: [f32; 2]
}

impl_vertex!(Vertex, position, normal, uv);

pub struct Loader {
  vertex: Vec<Vertex>,
  index: Vec<u32>,
}

impl Loader {
  pub fn load_opengex(location: String, texture: String) -> Loader {
    let model = OpengexPaser::new(location.clone());
    
    let vtx = model.get_vertex();
    let nrml = model.get_normal();
    let idx = model.get_index();
    let uvs = model.get_uv();
    
    let mut vertex: Vec<Vertex> = Vec::new();
    
    for i in 0..vtx.len() {
      let mut uv = [0.0, 0.0];
      if uvs.len() > i {
        uv = uvs[i];
      }
      vertex.push(Vertex { 
                    position: vtx[i], 
                    normal: nrml[i],
                    uv: uv });
    }
    
    let index = idx;//.iter().map(|i| *i as u16 ).collect::<Vec<u16>>();
    
    Loader {
      vertex: vertex,
      index: index,
    }
  }
  
  pub fn get_verticies(&self) -> Vec<Vertex> {
    self.vertex.clone()
  }
  
  pub fn get_indices(&self) -> Vec<u32> {
    self.index.clone()
  }
}


