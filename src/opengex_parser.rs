use std::str;
//use std::result;
//use std::fs::File;
//use std::io::Read;
//use std::num::ParseIntError;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use cgmath::Matrix4;
use cgmath::SquareMatrix;

const X: &str = "\"x\"";
const Y: &str = "\"y\"";
const Z: &str = "\"z\"";

const STRING: &str = "{string";

const FLOAT: &str = "{float";
//const FLOAT3
//const FLOAT16


const METRIC: &str = "Metric";
const KEY: &str = "(key";
const EQUALS: &str = "=";

const DISTANCE: &str = "\"distance\")";
const ANGLE: &str = "\"angle\")";
const TIME: &str = "\"time\")";
const UP: &str = "\"up\")";

const GEOMETRY_NODE: &str = "GeometryNode";
const OPEN_BRACKET: &str = "{";

const TAB: &str = "\t";
const NAME: &str = "Name";

fn get_float(v: Vec<&str>) -> Option<f32> {
  let mut result = None;
  
  let v: Vec<&str> = v[0].split("{").collect();
  let v: Vec<&str> = v[1].split("}").collect();
  if let Ok(float) = v[0].parse::<f32>() {
    result = Some(float);
  }
  
  result
}

fn get_string_value(v: Vec<&str>) -> Option<&str> {
  let mut result = None;
  
  let v: Vec<&str> = v[0].split("{").collect();
  let v: Vec<&str> = v[1].split("}").collect();
  result = Some(v[0]);

  
  result
}

fn to_utf8(text: &[u8]) -> String {
  str::from_utf8(text.clone()).expect("Error converting &[u8] to  String").to_string()
}

pub struct Normal {
  normal: [f32; 3],
}

pub struct Vertex {
  vertex: [f32; 3],
}

pub struct Index {
  index: u16,
}

pub struct UV {
  uv: [f32; 2],
}

struct Material {
  name: String,
  material_ref: String,
  texture: String,
  
  diffuse_colour: [f32; 3],
  specular_colour: [f32; 3],
  specular_power: f32,
}

impl Material {
  pub fn new() -> Material {
    Material {
      name: "".to_string(),
      material_ref: "".to_string(),
      texture: "".to_string(),
      
      diffuse_colour: [0.0, 0.0, 0.0],
      specular_colour: [0.0, 0.0, 0.0],
      specular_power: 0.0,
    }
  }
}

struct GeometryObject {
  object_ref: String,
  vertex: Vec<Vertex>,
  index: Vec<Index>,
  normal: Vec<Normal>,
  uv: Vec<UV>,
}

impl GeometryObject {
  pub fn new() -> GeometryObject {
    GeometryObject {
      object_ref: "".to_string(),
      vertex: Vec::new(),
      index: Vec::new(),
      normal: Vec::new(),
      uv: Vec::new(),
    }
  }
}

struct GeometryNode {
  name: String,
  transform: [f32; 16],
  
  object_ref: String,
  geometry_object: GeometryObject,
  
  material_ref: (i32, String),
  materiel: Material, 
}

impl GeometryNode {
  pub fn new(name: String) -> GeometryNode {
    GeometryNode {
      name: name,
      transform: [1.0, 0.0, 0.0, 0.0, 
                  0.0, 1.0, 0.0, 0.0, 
                  0.0, 0.0, 1.0, 0.0, 
                  0.0, 0.0, 0.0, 1.0],
      
      object_ref: "".to_string(),
      geometry_object: GeometryObject::new(),
      
      material_ref: (0, "".to_string()),
      materiel: Material::new(),
    }
  }
}

pub struct OpengexPaser {
  metric_dist: f32,
  metric_angle: f32,
  metric_time: f32,
  metric_up: String,
  
  num_nodes: i32,
  geometry: Vec<GeometryNode>,
}

impl OpengexPaser {
  pub fn new(location: String) -> OpengexPaser {
    let mut metric_dist = 1.0;
    let mut metric_angle = 1.0;
    let mut metric_time = 1.0;
    let mut metric_up: String = Y.to_string();
    
    let mut num_nodes: i32 = 0;
    let mut geometry: Vec<GeometryNode> = Vec::new();
    
    let mut still_in_node = false;
    let mut num_brackets_open = 0;
    
    if let Ok(file) = File::open(location.clone()) {
      let file = BufReader::new(file);
      
      for line in file.lines() {
        let line = line.expect("Unable to read line");
        let v: Vec<&str> = line.split(" ").collect();
        println!("{:?}", v);
        
        
        
        match v[0] {
          METRIC => {
            if v[1] == KEY && v[2] == EQUALS {
              match v[3] {
                DISTANCE => {
                  if v[4] == FLOAT {
                    if let Some(float) = get_float(vec!(v[5])) {
                      println!("Metric Distance found!");
                      metric_dist = float;
                    }
                  }
                },
                ANGLE => {
                  println!("Metric Angle found!");
                  if v[4] == FLOAT {
                    if let Some(float) = get_float(vec!(v[5])) {
                      metric_angle = float;
                    }
                  }
                },
                TIME => {
                  println!("Metric Time found!");
                  if v[4] == FLOAT {
                    if let Some(float) = get_float(vec!(v[5])) {
                      metric_time = float;
                    }
                  }
                },
                UP => {
                  println!("Metric Up found!");
                   if v[4] == STRING {
                     if let Some(dir) = get_string_value(vec!(v[5])) {
                       metric_up = dir.to_string();
                     }
                   }
                },
                _ => {
                  
                }
              }
            }
          },
          GEOMETRY_NODE => {
            still_in_node = true;
            geometry.push(GeometryNode::new(v[1].to_string()));
            println!("GeometryNode Found!");
          },
          NAME => {
            println!("Name found!");
          },
          
          OPEN_BRACKET => {
            if num_brackets_open < 2 {
              num_brackets_open += 1;
            } else {
              
            }
          },
          _ => {
            
          }
        }
      }
    } else {
      println!("Error: Model file at location {:?} does not exist!", location);
    }
    
    OpengexPaser {
      metric_dist: metric_dist,
      metric_angle: metric_angle,
      metric_time: metric_time,
      metric_up: metric_up.to_string(),
      
      num_nodes: num_nodes,
      geometry: geometry,
    }
  }
}
