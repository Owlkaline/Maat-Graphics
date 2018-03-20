use std::str;
use std::result;
use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;

use nom;
use nom::*;
use nom::{alpha,IResult,space};

use cgmath::Matrix4;
use cgmath::SquareMatrix;

fn to_utf8(text: &[u8]) -> String {
  str::from_utf8(text.clone()).expect("Error converting &[u8] to  String").to_string()
}

named!(take2, take!(2));
named!(take4, take!(4));
named!(take12, take!(12));
named!(double_tab, take_until!("\t\t"));

named!(open_bracket, take_until!("{"));
named!(next_number, take_while!(is_not_number));

named!(take_while_bracket, take_while!(is_bracket));

named!(next_float, take_until!("float["));
named!(take_float, take!(6));
named!(while_is_float, take_while!(is_float));
named!(while_isnt_char, take_while!(is_not_char));

named!(next_geometry, take_until!("GeometryNode $node"));
named!(end_geometry, take_until!("\n\t\t}\n\t}\n}"));
named!(geometry_name, take!(18));

named!(next_transform, take_until!("Transform\n\t{"));
named!(take_transform, take!(14));
named!(end_transform, take_until!("\t\t}\n\t}"));

named!(next_objectref, take_until!("ObjectRef {ref {"));
named!(take_objectref, take!(16));
named!(end_objectref, take_until!("}}"));

named!(next_materialref, complete!(take_until!(") {ref {")));
named!(take_materialref, take!(8));
named!(end_materialref, take_until!("}}"));

named!(next_name, take_until!("Name {string {\""));
named!(take_name, take!(15));
named!(end_name, take_until!("\"}}"));

named!(next_object, take_until!("GeometryObject "));
named!(end_object, take_until!("\t\t}\n\t}\n}"));
named!(object_name, take!(15));

named!(next_verticies, take_until!("VertexArray (attrib = \"position\")"));
named!(take_verticies, take!(33));
named!(end_verticies, take_until!("\n\t\t\t}\n\t\t}"));

named!(next_normals, take_until!("VertexArray (attrib = \"normal\")"));
named!(take_normals, take!(31));
named!(end_normals, take_until!("\n\t\t\t}\n\t\t}"));


named!(int32 <&str, Result<i32,ParseIntError>>,
    map!(digit, str::FromStr::from_str)
);

fn is_float(c: u8) -> bool {
  is_digit(c) || c as char == '.' || c as char == '-'
}

fn is_not_char(c: u8) -> bool {
  c as char == '\n' || c as char == '\t' || c as char == ',' || c as char == ' '
}

fn is_bracket(c: u8) -> bool {
  c as char == '}' || c as char == '{' || c as char == '[' || c as char == ']'
}

fn is_not_number(c: u8) -> bool {
  is_not_char(c) || is_alphabetic(c) || c as char == '{' || c as char == '}' || c as char == '[' || c as char == ']' || c as char == '/'
}

#[derive(Debug)]
pub struct Normals {
  normal: Vec<[f32; 3]>,
}

#[derive(Debug)]
pub struct Verticies {
  vertex: Vec<[f32; 3]>,
}

pub struct Indicies {
  index: Vec<u16>,
}

pub struct UVs {
  uv: Vec<[f32; 2]>,
}

pub struct OpengexData {
  num_nodes: i32,
  object_name: Vec<String>,
  object_ref: Vec<String>,
  material_ref: Vec<String>,
  node_transforms: Vec<Matrix4<f32>>,
  verticies: Vec<Verticies>,
  indicies: Vec<Indicies>,
  normal: Vec<Normals>,
  uv: Vec<UVs>,
}

impl OpengexData {
  pub fn new(location: String) -> OpengexData {
    let mut parser = OpengexParser::new(location);
    
    let mut num_nodes: i32 = 0;
    let mut object_name: Vec<String> = Vec::new();
    let mut object_ref: Vec<String> = Vec::new();
    let mut material_ref: Vec<String> = Vec::new();
    let mut node_transforms: Vec<Matrix4<f32>> = Vec::new();
    let mut verticies: Vec<Verticies> = Vec::new();
    let mut indicies: Vec<Indicies> = Vec::new();
    let mut normal: Vec<Normals> = Vec::new();
    let mut uv: Vec<UVs> = Vec::new();
    
    let mut index = 1;
    
    let mut found_all_nodes = false;
    while !found_all_nodes {
      let (node, success) = parser.get_geometry_node_n(index);
      if !success {
        found_all_nodes = true;
        continue;
      }
      let node_name = OpengexParser::get_name_from_node(node.clone());
      let objectref = OpengexParser::get_objectref_from_node(node.clone());
      let materialref = OpengexParser::get_materialref_from_node(node.clone());
      let transform = OpengexParser::get_transform_from_node(node.clone());
      println!("{:?}", node_name);
      println!("{:?}", objectref);
      println!("{:?}", materialref);
      println!("{:?}", transform);
      
      object_name.push(node_name);
      object_ref.push(objectref);
      material_ref.push(materialref);
      node_transforms.push(transform);
      
      index += 1;
    }
    
    let num_nodes = index;
    
    
    let (object, success) = parser.get_geometry_object(object_ref[0].clone());
    
    let object_verticies = OpengexParser::get_verticies_from_object(object.clone());
    let object_normals = OpengexParser::get_normals_from_object(object.clone());
    println!("{:?}", object_verticies);
    println!("{:?}", object_normals);
    //for 0..num_nodes {
      
   // }
   
    //let mut found_all_objects = false;
    //while !found_all_objects {
      
    //}
    
    OpengexData {
      num_nodes: num_nodes,
      object_name: object_name,
      object_ref: object_ref,
      material_ref: material_ref,
      node_transforms: node_transforms,
      verticies: verticies,
      indicies: indicies,
      normal: normal,
      uv: uv,
    }
  }
}


pub struct OpengexParser {
  text: String,
}

impl OpengexParser {
  pub fn new(location: String) -> OpengexParser {
    let mut file_h = File::open(location).unwrap();
    let mut source = String::new();
    file_h.read_to_string(&mut source).unwrap();
    
    OpengexParser {
      text: source,
    }
  }
  
  pub fn get_geometry_node_n(&self, index: i32) -> (String, bool) {
    let mut result: bool = false;
    let text = self.text.clone();
    let text = next_geometry(text.as_bytes());
    let text = geometry_name(text.unwrap().0);
    let value = int32(str::from_utf8(text.clone().unwrap().0).expect("failed str parse")).unwrap().1.unwrap();
    
    if value == index {
      result = true;
    }
    
    let text = end_geometry(text.unwrap().0);
    let text = str::from_utf8(text.unwrap().1).expect("failed str parse").to_string();
    
    (text, result)
  }
  
  pub fn get_geometry_object(&self, ref_name: String) -> (String, bool) {
    let mut success = false;
    
    let text = self.text.clone();
    let text = next_object(text.as_bytes());
    let text = object_name(text.unwrap().0);
    let text = double_tab(text.unwrap().0);
    
    if to_utf8(text.clone().unwrap().1) == ref_name {
      success = true;
    }
    
    let text = to_utf8(end_object(text.unwrap().0).unwrap().1);
    
    (text, success)
  }
  
  pub fn get_name_from_node(node: String) -> String {
    let text = next_name(node.as_bytes());
    let text = end_name(text.unwrap().0);
    let text = take_name(text.unwrap().1);
    
    str::from_utf8(text.unwrap().0).expect("failed str parse").to_string()
  }
  
  pub fn get_objectref_from_node(node: String) -> String {
    let text = next_objectref(node.as_bytes());
    let text = end_objectref(text.unwrap().0);
    let text = take_objectref(text.unwrap().1);
    
    str::from_utf8(text.unwrap().0).expect("failed str parse").to_string()
  }
  
  pub fn get_materialref_from_node(node: String) -> String {
   /* let text = next_materialref(node.as_bytes());
    match text {
      Ok(t) => (),
      Err(e) => return String::from(""),
    }
    
    let text = end_materialref(text.unwrap().0);
    let text = take_materialref(text.unwrap().1);
    
    str::from_utf8(text.unwrap().0).expect("failed str parse").to_string()*/
    "".to_string()
  }
  
  pub fn get_transform_from_node(node: String) -> Matrix4<f32> {
    let text = next_transform(node.as_bytes());
    let text = take_transform(text.unwrap().0);
    
    let floats = OpengexParser::parse_float(str::from_utf8(text.clone().unwrap().0).expect("").to_string());
    
    Matrix4::new(
      floats[0], floats[1], floats[2], floats[3],
      floats[4], floats[5], floats[6], floats[7],
      floats[8], floats[9], floats[10], floats[11],
      floats[12], floats[13], floats[14], floats[15])
    //Matrix4::identity()
  }
  
  pub fn parse_float(node: String) -> Vec<f32> {
   /* let text = next_float(node.as_bytes());
    let text = take_float(text.unwrap().0);
    
    let value = int32(str::from_utf8(text.clone().unwrap().0).expect("failed str parse")).unwrap().1.unwrap();
    
    let text = take2(text.unwrap().0);
    let text = open_bracket(text.unwrap().0);
    let text = take_while_bracket(text.unwrap().0);
    let text = open_bracket(text.unwrap().0);
    let mut text = take_while_bracket(text.unwrap().0);
    
    let mut values: Vec<f32> = Vec::new();
    
    let mut finished = false;
    while(!finished) {
      for i in 0..value {
        text = next_number(text.unwrap().0);
        println!("{:?}", to_utf8(text.clone().unwrap().0));
        let new_val = float(text.clone().unwrap().0).unwrap().1;
        values.push(new_val);
        
        text = while_is_float(text.unwrap().0);
      }
      if text.clone().unwrap().0.len() <= 1 {
        finished = true;
      }
    }
    
    values*/
    vec!(0.0, 0.0)
  }
  
  pub fn get_verticies_from_object(object: String) -> Verticies {
    let text = next_verticies(object.as_bytes());
    let text = take_verticies(text.unwrap().0);
    let text = end_verticies(text.unwrap().0);
    
    let position = OpengexParser::parse_float(to_utf8(text.unwrap().1));
    
    
    let mut vertices: Verticies = Verticies { vertex: Vec::new() };
    
    let mut vertex = [0.0,0.0,0.0];
    for i in 0..position.len() {
      vertex[i%3] = position[i];
      
      if i % 3 == 2 {
        vertices.vertex.push(vertex);
      }
    }
    
    vertices
  }
  
  pub fn get_normals_from_object(object: String) -> Normals {
    let text = next_verticies(object.as_bytes());
    let text = take_verticies(text.unwrap().0);
    let text = end_verticies(text.unwrap().0);
    println!("{:?}", to_utf8(text.clone().unwrap().0));
    let file_normal = OpengexParser::parse_float(to_utf8(text.unwrap().1));
    
    
    let mut normals: Normals = Normals { normal: Vec::new() };
    
    let mut normal = [0.0,0.0,0.0];
    for i in 0..file_normal.len() {
      normal[i%3] = file_normal[i];
      
      if i % 3 == 2 {
        normals.normal.push(normal);
      }
    }
    
    normals
  }
}
