use crate::DescriptorSet;

use gltf;

struct Vertex {
  pos: [f32; 3],
  normal: [f32; 3],
  uv: [f32; 2],
  colour: [f32; 3],
}

#[derive(Debug)]
struct Primitive {
  first_index: u32,
  index_count: u32,
  material_index: i32,
}

struct Mesh {
  pub primitives: Vec<Primitive>,
}

// idk if keep
struct Node {
  pub children: Vec<Option<Node>>,
  pub mesh: Mesh,
  matrix: [f32; 16],
}

#[derive(Debug)]
struct Material {
  base_colour_factor: [f32; 4],
  base_colour_texture_index: u32,
}

struct Image {
  texture: crate::Image,
  descriptor_set: DescriptorSet,
}

struct Texture {
  image_index: i32,
}

fn load_materials(gltf: &gltf::Document, materials: &mut Vec<Material>) {
  for material in gltf.materials() {
    let pbr = material.pbr_metallic_roughness();
    
    materials.push(Material {
      base_colour_factor: pbr.base_color_factor(),
      base_colour_texture_index: if let Some(texture_info) = pbr.base_color_texture() {
        texture_info.texture().index() as u32
      } else {
        0
      },
    });
  }
}

fn load_node(gltf_node: &gltf::Node, buffers: &Vec<gltf::buffer::Data>, 
              index_buffer: &mut Vec<u32>, vertex_buffer: &mut Vec<Vertex>, depth: i32) -> Option<Node> {
  let first_index = index_buffer.len();
  let vertex_start = vertex_buffer.len();
  let mut index_count = 0;
  //let mut vertex_count = 0;
  
  let mut node = Node {
    children: Vec::new(),
    mesh: Mesh {
      primitives: Vec::new(),
    },
    matrix: [1.0, 0.0, 0.0, 0.0,
             0.0, 1.0, 0.0, 0.0,
             0.0, 0.0, 1.0, 0.0,
             0.0, 0.0, 0.0, 1.0],
  };
  
  for child in gltf_node.children() {
    node.children.push(load_node(&child, buffers, index_buffer, vertex_buffer, depth + 1));
  }
  
  for primitive in gltf_node.mesh().unwrap().primitives() {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
     
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    
    if let Some(iter) = reader.read_positions() {
      //vertex_count = iter.len();
      for position in iter {
        //println!("{:?}", positions);
        vertices.push(position);
      }
    }
    
    if let Some(iter) = reader.read_normals() {
      for normal in iter {
        //println!("{:?}", normals);
        normals.push(normal);
      }
    }
    
    if let Some(read_tex_coords) = reader.read_tex_coords(0) {
      match read_tex_coords {
        gltf::mesh::util::ReadTexCoords::F32(iter) => {
          for texcoord in iter {
            //println!("{:?}", texcoords);
            uvs.push(texcoord);
          }
        },
        _ => {
          println!("tex coords is other from f32");
        }
      }
    }
    
    if let Some(indices) = reader.read_indices() {
      let indices = indices.into_u32();
      index_count = indices.len();
      
      for index in indices {
        index_buffer.push(index + vertex_start as u32);
      }
    }
    
    for i in 0..vertices.len() {
      vertex_buffer.push(
        Vertex {
          pos: vertices[i],
          normal: normals[i],
          uv: uvs[i],
          colour: [0.0, 0.0, 0.0],
        }
      );
    }
    
    let mat_idx = primitive.material().index().unwrap_or(0);
    
    node.mesh.primitives.push(Primitive {
      first_index: first_index as u32,
      index_count: index_count as u32,
      material_index: mat_idx as i32,
    });
  }
  
  Some(node)
}

pub fn load_gltf() {
  let _images: Vec<Image> = Vec::new();
  let _textures: Vec<Texture> = Vec::new();
  let mut materials: Vec<Material> = Vec::new();
  let mut nodes: Vec<Node> = Vec::new();
  
  let mut index_buffer = Vec::new();
  let mut vertex_buffer = Vec::new();
  
  let (gltf, buffers, _images) = gltf::import("./models/DamagedHelmet.glb").unwrap();
  
  for scene in gltf.scenes() {
    for node in scene.nodes() {
      nodes.push(load_node(&node, &buffers, &mut index_buffer, &mut vertex_buffer, 1).unwrap());
    }
  }
  
  load_materials(&gltf, &mut materials);
  
  println!("{:?}", materials);
}
