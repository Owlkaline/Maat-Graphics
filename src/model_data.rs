use opengex;

use piston_meta::parse;
use piston_meta::syntax;
use piston_meta::stderr_unwrap;
use piston_meta_search::Search;
use piston_meta::ParseError;
use range::Range;

use std::fs::File;
use std::io::Read;

#[derive(Copy, Clone)]
pub struct Vertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub uv: [f32; 2]
}

impl_vertex!(Vertex, position, normal, uv);

pub struct Loader {
  vertex: Vec<Vertex>,
  index: Vec<u16>,
}

impl Loader {
  pub fn load_opengex(location: String, texture: String) -> Loader {
    let mut file_h = File::open("resources/models/opengex-syntax.txt").unwrap();
    let mut source = String::new();
    file_h.read_to_string(&mut source).unwrap();
    let rules = stderr_unwrap(&source, syntax(&source));
    
    let mut file_h = File::open(location).unwrap();
    let mut source = String::new();
    file_h.read_to_string(&mut source).unwrap();
    let mut data = vec![];
    stderr_unwrap(&source, parse(&rules, &source, &mut data));
    
    let s = Search::new(&data);
    
    let verticies: Vec<Vertex> = {
      let vertex_data: Vec<[f32; 3]> = stderr_unwrap(&source, s.for_bool("position", true,
        |ref mut s| {
            let mut vs = Vec::with_capacity(24);
            loop {
                vs.push([
                   match s.f64("x") {
                     Ok(t)  => t,
                     Err(e) => break,
                   } as f32,
                    try!(s.f64("y")) as f32,
                    try!(s.f64("z")) as f32
                ]);
            }
            Ok(vs)
        }));
      let normal_data: Vec<[f32; 3]> = stderr_unwrap(&source, s.for_bool("normal", true,
        |ref mut s| {
            let mut vs = Vec::with_capacity(24);
            loop {
                vs.push([
                   match s.f64("x") {
                     Ok(t)  => t,
                     Err(e) => break,
                   } as f32,
                   // try!(s.f64("x")) as f32,
                    try!(s.f64("y")) as f32,
                    try!(s.f64("z")) as f32
                ]);
            }
            Ok(vs)
        }));
      
      
      let uv_data: Vec<[f32; 2]> = {
        let mut tc: Vec<[f32; 2]> = Vec::with_capacity(24);
        if texture != String::from("") {
          tc = stderr_unwrap(&source, s.for_bool("texcoord", true,
            |ref mut s| {
              let mut vs = tc.clone();                 
                loop {
                    vs.push(
                      [
                        match s.f64("x") {
                          Ok(t)  => t,
                          Err(e) => break,
                        } as f32,
                       try!(s.f64("y")) as f32
                      ]);
                }
                Ok(vs)
            }));
          } else {
            for _ in 0..vertex_data.len() {
              tc.push([1.0, 0.0]);
            }
          }
        tc
      };
      println!("vertex: {:?}", vertex_data.len());
      println!("normal: {:?}", normal_data.len());
      println!("    uv: {:?}", uv_data.len());   
  
      let mut vert: Vec<Vertex> = Vec::with_capacity(24);
      for i in 0..vertex_data.len() {
        vert.push(Vertex {
            position: vertex_data[i],
            normal: normal_data[i],
            uv: uv_data[i],
          }
        );
      }
      
      vert
    };
    
    let indicies: Vec<u16> = stderr_unwrap(&source, s.for_node("IndexArray",
        |ref mut s| {
            let mut is = Vec::with_capacity(36);
            loop {
                is.push(
                   match s.f64("a") {
                     Ok(t)  => t,
                     Err(e) => break,
                   } as u16);
                is.push(try!(s.f64("b")) as u16);
                is.push(try!(s.f64("c")) as u16);
            }
            Ok(is)
    }));
            
    Loader {
      vertex: verticies,
      index: indicies,
    }
  }
  
  pub fn get_verticies(&self) -> Vec<Vertex> {
    self.vertex.clone()
  }
  
  pub fn get_indicies(&self) -> Vec<u16> {
    self.index.clone()
  }
}


