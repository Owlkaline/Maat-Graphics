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
const REF: &str = "{ref";
const FLOAT: &str = "{float";
const INDEX: &str = "(index";
const PRIMITIVE: &str = "(primitive";
const ATTRIB: &str = "(attrib";
const POSITION: &str = "\"position\")";
const NORMAL: &str = "\"normal\")";

const FLOAT3: &str = "float[3]";
const FLOAT16: &str = "float[16]";
const UNSIGNED_INT3: &str = "unsigned_int32[3]";

const METRIC: &str = "Metric";
const KEY: &str = "(key";
const EQUALS: &str = "=";

const DISTANCE: &str = "\"distance\")";
const ANGLE: &str = "\"angle\")";
const TIME: &str = "\"time\")";
const UP: &str = "\"up\")";

const GEOMETRY_NODE: &str = "GeometryNode";
const GEOMETRY_OBJECT: &str = "GeometryObject";
const OPEN_BRACKET: &str = "{";
const CLOSE_BRACKET: &str = "}";

const ZERO_BRACKET: &str = "0)";

const TAB: &str = "\t";
const NAME: &str = "Name";
const OBJECT_REF: &str = "ObjectRef";
const MATERIAL_REF: &str = "MaterialRef";
const TRANSFORM: &str = "Transform";

const MESH: &str = "Mesh";
const VERTEXARRAY: &str = "VertexArray";
const INDEXARRAY: &str = "IndexArray";

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

fn remove_brackets(v: &str) -> &str {
  let v = v.trim_matches('{');
  let v = v.trim_matches('}');
  let v = v.trim_matches(')');
  let v = v.trim_matches('(');
  let v = v.trim_matches('\"');
  let v = v.trim_matches('/');
  let v = v.trim_matches(',');
  let v = v.trim_matches('}');
  let v = v.trim();
  v
}

fn to_utf8(text: &[u8]) -> String {
  str::from_utf8(text.clone()).expect("Error converting &[u8] to  String").to_string()
}

#[derive(Debug)]
pub struct Normal {
  normal: [f32; 3],
}

#[derive(Debug)]
pub struct Vertex {
  vertex: [f32; 3],
}

#[derive(Debug)]
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
  mesh: String,
  vertex: Vec<Vertex>,
  index: Vec<Index>,
  normal: Vec<Normal>,
  uv: Vec<UV>,
}

impl GeometryObject {
  pub fn new() -> GeometryObject {
    GeometryObject {
      mesh: "".to_string(),
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
    
    let mut in_geometrynode = (-1, false);
    let mut in_transform = (-1, false);
    let mut in_float3 = (-1, false);
    let mut in_float16 = (-1, false, 0);
    let mut in_unsigned_int3 = (-1, false);
    let mut in_values = (-1, false);
    
    let mut in_geometryobject = (-1, false, 0);
    let mut in_vertexposition = (-1, false);
    let mut in_vertexnormal = (-1, false);
    let mut in_index = (-1, false);
    
    let mut num_brackets_open = 0;
    
    if let Ok(file) = File::open(location.clone()) {
      let file = BufReader::new(file);
      
      for line in file.lines() {
        let line = line.expect("Unable to read line");
        let line = line.trim();
        let line = line.trim_left();
        let line = line.trim_matches('\t');
        let mut v: Vec<&str> = line.split(" ").collect();
        
        //println!("{:?}", v);
        if v[0].contains(FLOAT3) {
          v[0] = remove_brackets(v[0]);
        }
        if v[0].contains(UNSIGNED_INT3) {
          v[0] = remove_brackets(v[0]);
        }
        
        match v[0] {
          METRIC => {
            if v[1] == KEY && v[2] == EQUALS {
              match v[3] {
                DISTANCE => {
                  if v[4] == FLOAT {
                    if let Some(float) = get_float(vec!(v[5])) {
                      //println!("Metric Distance found!");
                      metric_dist = float;
                    }
                  }
                },
                ANGLE => {
                  //println!("Metric Angle found!");
                  if v[4] == FLOAT {
                    if let Some(float) = get_float(vec!(v[5])) {
                      metric_angle = float;
                    }
                  }
                },
                TIME => {
                  //println!("Metric Time found!");
                  if v[4] == FLOAT {
                    if let Some(float) = get_float(vec!(v[5])) {
                      metric_time = float;
                    }
                  }
                },
                UP => {
                  //println!("Metric Up found!");
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
            in_geometrynode = (num_brackets_open, true);
            num_nodes += 1;
            geometry.push(GeometryNode::new(v[1].to_string()));
            //println!("GeometryNode Found!");
          },
          NAME => {
            //println!("Name found!");
            if v[1] == STRING {
              let name = remove_brackets(v[2]);
              if in_geometrynode.1 {
                geometry[(num_nodes-1) as usize].name = name.to_string();
              }
            }
          },
          OBJECT_REF => {
            //println!("Object ref found!");
            if v[1] == REF {
              let objectref = remove_brackets(v[2]);
              if in_geometrynode.1 {
                geometry[(num_nodes-1) as usize].object_ref = objectref.to_string();
              }
            }
          },
          MATERIAL_REF => {
            if v[1] == INDEX {
              if v[2] == EQUALS {
                if v[3] == ZERO_BRACKET {
                  if v[4] == REF {
                    if in_geometrynode.1 {
                      let materialref = remove_brackets(v[5]);
                      geometry[(num_nodes-1) as usize].material_ref = (0, materialref.to_string());
                    }
                  }
                }
              }
            }
          },
          TRANSFORM => {
            in_transform = (num_brackets_open, true);
          },
          FLOAT3 => {
            in_float3 = (num_brackets_open, true);
          },
          FLOAT16 => {
            in_float16 = (num_brackets_open, true, 0);
          },
          UNSIGNED_INT3 => {
            in_unsigned_int3 = (num_brackets_open, true);
          },
          GEOMETRY_OBJECT => {
            println!("geomtry object found!");
            let name = remove_brackets(v[1]);
            println!("{}", name);
            for i in 0..geometry.len() {
              if name == geometry[i].object_ref {
                in_geometryobject = (num_brackets_open, true, i);
                break;
              }
            }
          },
          MESH => {
            if in_geometryobject.1 {
              if v[1] == PRIMITIVE {
                if v[2] == EQUALS {
                  let mesh_name = remove_brackets(v[3]);
                  geometry[in_geometryobject.2].geometry_object.mesh = mesh_name.to_string();
                }
              }
            }
          },
          VERTEXARRAY => {
            if in_geometryobject.1 {
              if v[1] == ATTRIB {
                if v[2] == EQUALS {
                  if v[3] == POSITION {
                    println!("Vertex array found!");
                    in_vertexposition = (num_brackets_open, true);
                  } else
                  if v[3] == NORMAL {
                    println!("Normal array found!");
                    in_vertexnormal = (num_brackets_open, true);
                  }
                }
              }
            }
          },
          INDEXARRAY => {
            if in_geometryobject.1 {
              println!("Index array found!");
              in_index = (num_brackets_open, true);
            }
          },
          OPEN_BRACKET => {
            num_brackets_open += 1;
            //println!("open bracket");
          },
          CLOSE_BRACKET => {
            num_brackets_open -= 1;
            if in_geometrynode.1 {
              if in_geometrynode.0 == num_brackets_open {
                in_geometrynode = (-1, false);
              }
            }
            if in_transform.1 {
              if in_transform.0 == num_brackets_open {
                in_transform = (-1, false);
              }
            }
            if in_float3.1 {
              if in_float3.0 == num_brackets_open {
                in_float3 = (-1, false);
              }
            }
            if in_float16.1 {
              if in_float16.0 == num_brackets_open {
                in_float16 = (-1, false, 0);
              }
            }
            if in_unsigned_int3.1 {
              if in_unsigned_int3.0 == num_brackets_open {
                in_unsigned_int3 = (-1, false);
              }
            }
            if in_values.1 {
              if in_values.0 == num_brackets_open {
                in_values = (-1, false);
              }
            }
            if in_geometryobject.1 {
              if in_geometryobject.0 == num_brackets_open {
                in_geometryobject = (-1, false, 0);
              }
            }
            if in_vertexposition.1 {
              if in_vertexposition.0 == num_brackets_open {
                in_vertexposition = (-1, false);
              }
            }
            if in_vertexnormal.1 {
              if in_vertexnormal.0 == num_brackets_open {
                in_vertexnormal = (-1, false);
              }
            }
            if in_index.1 {
              if in_index.0 == num_brackets_open {
                in_index = (-1, false);
              }
            }
            
            //println!("close bracket");
          },
          _ => {
            if v[0].len() > 1 && v[0].contains(char::is_numeric) {
              if in_geometrynode.1 {
                if in_float16.1 {
                  println!("numbers");
                  for i in 0..v.len() {
                    let value = remove_brackets(v[i]);
                    if let Ok(float) = value.parse::<f32>() {
                      geometry[(num_nodes-1) as usize].transform[in_float16.2] = float;
                      in_float16.2 += 1;
                    }
                  }
                }
              }
              if in_geometryobject.1 {
                if in_vertexposition.1 {
                  if in_float3.1 {
                    let mut vtx: Vertex = Vertex { vertex: [0.0,0.0,0.0] };
                    let mut idx = 0;
                    for i in 0..v.len() {
                      let value = remove_brackets(v[i]);
                      if let Ok(float) = value.parse::<f32>() {
                        vtx.vertex[idx] = float;
                        idx += 1;
                        if idx == 3 {
                          let temp_vtx = vtx;
                          geometry[(in_geometryobject.2) as usize].geometry_object.vertex.push(temp_vtx);
                          idx = 0;
                          vtx = Vertex { vertex: [0.0, 0.0, 0.0] };
                        }
                      }
                    }
                  }
                }
                if in_vertexnormal.1 {
                  if in_float3.1 {
                    let mut nrml: Normal = Normal { normal: [0.0,0.0,0.0] };
                    let mut idx = 0;
                    for i in 0..v.len() {
                      let value = remove_brackets(v[i]);
                      if let Ok(float) = value.parse::<f32>() {
                        nrml.normal[idx] = float;
                        idx += 1;
                        if idx == 3 {
                          let temp_nrml = nrml;
                          geometry[(in_geometryobject.2) as usize].geometry_object.normal.push(temp_nrml);
                          idx = 0;
                          nrml = Normal { normal: [0.0, 0.0, 0.0] };
                        }
                      }
                    }
                  }
                }
                if in_index.1 {
                  if in_unsigned_int3.1 {
                    for i in 0..v.len() {
                      let value = remove_brackets(v[i]);
                      if let Ok(unsigned) = value.parse::<u16>() {
                        geometry[(in_geometryobject.2) as usize].geometry_object.index.push(Index { index: unsigned });
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    } else {
      println!("Error: Model file at location {:?} does not exist!", location);
    }
    
    for i in 0..geometry.len() {
      println!("{}", geometry[i].name);
      println!("{}", geometry[i].object_ref);
      println!("{:?}", geometry[i].material_ref);
      println!("{:?}", geometry[i].transform);
      println!("{:?}", geometry[i].geometry_object.mesh);
    }
    //println!("{:?}", geometry[0].geometry_object.vertex);
    //println!("{:?}", geometry[0].geometry_object.normal);
    //println!("{:?}", geometry[0].geometry_object.index);
    
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
