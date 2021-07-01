use crate::modules::{Vulkan, DescriptorSet, DescriptorPoolBuilder, DescriptorWriter, Buffer, Sampler};
use crate::shader_handlers::{Camera, TextureHandler};

use crate::Image as vkimage;

use ash::vk;

use gltf;

#[derive(Copy, Clone)]
pub struct MeshVertex {
  pub pos: [f32; 3],
  pub normal: [f32; 3],
  pub uv: [f32; 2],
  pub colour: [f32; 3],
  pub joint_indices: [f32; 4],
  pub joint_weights: [f32; 4],
}

pub struct Skin {
  name: String,
  skeleton_root: Option<u32>,
  inverse_bind_matrices: Vec<[f32; 16]>,
  joints: Vec<u32>,
  inverse_bind_matrix_buffer: Buffer<f32>,
  pub descriptor_set: DescriptorSet,
}

pub struct Animation {
  name: String,
  samplers: Vec<AnimationSampler>,
  //channels: Vec<AnimationChannel>,
  start: f32,
  end: f32,
  current_time: f32,
}

pub struct AnimationChannel {
  path: String,
  node: Vec<Node>,
  sampler_index: u32,
}

pub enum AnimationInterpolation {
  Linear,
  Step,
  CubicSpline,
}

pub struct AnimationSampler {
  interpolation: AnimationInterpolation,
  inputs: Vec<f32>,
  outputs_vec4: Vec<[f32; 4]>,
}

#[derive(Debug)]
pub struct Primitive {
  pub first_index: u32,
  pub index_count: u32,
  pub material_index: i32,
}

#[derive(Debug)]
pub struct Mesh {
  pub primitives: Vec<Primitive>,
}

// idk if keep
#[derive(Debug)]
pub struct Node {
  pub idx: u32,
  pub children: Vec<Node>,
  pub mesh: Mesh,
  pub matrix: [f32; 16],
  pub skin: i32,
}

#[derive(Debug)]
pub struct Material {
  base_colour_factor: [f32; 4],
  pub base_colour_texture_index: u32,
}

pub struct MeshImage {
  pub texture: vkimage,
  pub descriptor_set: DescriptorSet,
}

#[derive(Debug)]
pub struct Texture {
  pub image_index: i32,
}

pub struct GltfModel {
  nodes: Vec<Node>,
  mesh_index_buffer: Buffer<u32>,
  mesh_vertex_buffer: Buffer<MeshVertex>,
  mesh_images: Vec<MeshImage>,
  mesh_skins: Vec<Skin>,
  mesh_animations: Vec<Animation>,
  textures: Vec<Texture>,
  materials: Vec<Material>,
  descriptor_pool: vk::DescriptorPool,
}

impl GltfModel {
  pub fn nodes(&self) -> &Vec<Node> {
    &self.nodes
  }
  
  pub fn images(&self) -> &Vec<MeshImage> {
    &self.mesh_images
  }
  
  pub fn index_buffer(&self) -> &Buffer<u32> {
    &self.mesh_index_buffer
  }
  
  pub fn vertex_buffer(&self) -> &Buffer<MeshVertex> {
    &self.mesh_vertex_buffer
  }
  
  pub fn materials(&self) -> &Vec<Material> {
    &self.materials
  }
  
  pub fn textures(&self) -> &Vec<Texture> {
    &self.textures
  }
  
  pub fn skins(&self) -> &Vec<Skin> {
    &self.mesh_skins
  }
}

fn load_animation(gltf: &gltf::Document, buffers: &Vec<gltf::buffer::Data>, animations: &mut Vec<Animation>) {
  let gltf_animations = gltf.animations();
  
  for animation in gltf_animations {
    let name = animation.name().unwrap_or("AnimationHasNoName").to_string();
    let mut start = 0.0;
    let mut end = 0.0;
    
    let mut samplers = Vec::new();
    for sampler in animation.samplers() {
      let interpolation = {
        match sampler.interpolation() {
          gltf::animation::Interpolation::Linear => {
            AnimationInterpolation::Linear
          },
          gltf::animation::Interpolation::Step => {
            AnimationInterpolation::Step
          },
          gltf::animation::Interpolation::CubicSpline => {
            AnimationInterpolation::CubicSpline
          },
        }
      };
      
      let mut inputs = Vec::new();
      let mut outputs = Vec::new();
      
      let input_accessor = sampler.input();
      
      start = input_accessor.min().unwrap().as_array().unwrap()[0].as_f64().unwrap() as f32;
      end = input_accessor.max().unwrap().as_array().unwrap()[0].as_f64().unwrap() as f32;
      
      let in_view = input_accessor.view().unwrap();
      
      let data = &buffers[in_view.buffer().index()].0;
      let begin = in_view.offset();
      let end = begin + in_view.length();
      let input_data_u8 = &data[begin..end];
      //let mut input_data = Vec::new();
      
      for bytes in input_data_u8.chunks(4) {
        inputs.push(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
      }
      
      let output_accessor = sampler.output();
      let out_view = output_accessor.view().unwrap();
      
      let data = &buffers[out_view.buffer().index()].0;
      let begin = out_view.offset();
      let end = begin + out_view.length();
      let output_data_u8 = &data[begin..end];
      
      let mut output_data = Vec::new();
      for bytes in output_data_u8.chunks(4) {
        output_data.push(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
      }
      
      match output_accessor.dimensions() {
        gltf::accessor::Dimensions::Vec3 => {
          for vec3 in output_data.chunks(3) {
            outputs.push([vec3[0], vec3[1], vec3[2], 0.0]);
          }
        },
        gltf::accessor::Dimensions::Vec4 => {
          for vec4 in output_data.chunks(4) {
            outputs.push([vec4[0], vec4[1], vec4[2], vec4[3]]);
          }
        },
        _ => {
          
        }
      }
      
      samplers.push(AnimationSampler {
        interpolation,
        inputs,
        outputs_vec4: outputs,
      });
    }
    
    //let mut channels = Vec::new();
    /*for channel in animation.channels() {
      let target = channel.target().node()
      AnimationChannel {
        path: String,
        node: Vec<Node>,
        sampler_index: u32,
      }
    }*/
    
    animations.push(
      Animation {
        name,
        samplers,
        //channels,
        start,
        end,
        current_time: 0.0,
      }
    );
  }
}

fn load_skins(vulkan: &mut Vulkan, gltf: &gltf::Document, buffers: &Vec<gltf::buffer::Data>, 
              descriptor_pool: &vk::DescriptorPool, nodes: &mut Vec<Node>, skins: &mut Vec<Skin>) {
  let gltf_skins = gltf.skins();
  
  let mut nodes_updated = Vec::new();
  
  for skin in gltf_skins {
    for i in 0..nodes.len() as usize {
      if !nodes_updated.contains(&i) {
        if nodes[i].skin != -1 {
          if skin.index() == nodes[i].skin as usize {
            nodes[i].skin = skins.len() as i32;
            nodes_updated.push(i);
          }
        }
      }
    }
    
    let name = skin.name().unwrap_or("").to_string();
    
    let skeleton_root = {
      if let Some(node) = skin.skeleton() {
        Some(node.index() as u32)
      } else {
        None
      }
    };
    
    let mut joints = Vec::new();
    for joint in skin.joints() {
      joints.push(joint.index() as u32);
    }
    
    let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
    
    let mut matrices = Vec::new();
    let mut raw_matrices = Vec::new();
    
    if let Some(inverse_bind_matrices) = reader.read_inverse_bind_matrices() {
      for matrix in inverse_bind_matrices {
        let mut new_matrix = Camera::mat4_identity();
        
        new_matrix[0] = matrix[0][0];
        new_matrix[1] = matrix[0][1];
        new_matrix[2] = matrix[0][2];
        new_matrix[3] = matrix[0][3];
        
        new_matrix[4] = matrix[1][0];
        new_matrix[5] = matrix[1][1];
        new_matrix[6] = matrix[1][2];
        new_matrix[7] = matrix[1][3];
        
        new_matrix[8] = matrix[2][0];
        new_matrix[9] = matrix[2][1];
        new_matrix[10] = matrix[2][2];
        new_matrix[11] = matrix[2][3];
        
        new_matrix[12] = matrix[3][0];
        new_matrix[13] = matrix[3][1];
        new_matrix[14] = matrix[3][2];
        new_matrix[15] = matrix[3][3];
        
        matrices.push(new_matrix);
        raw_matrices.append(&mut new_matrix.to_vec());
      }
    }
    
    let inverse_bind_matrix_buffer = Buffer::<f32>::new_storage_buffer(vulkan.device(), &raw_matrices);
    //inverse_bind_matrix.push(new_matrix);
    let descriptor_set = DescriptorSet::builder()
                                .storage_vertex()
                                .build(vulkan.device(), descriptor_pool);
    let descriptor_set_writer = DescriptorWriter::builder()
                                                 .update_storage_buffer(&inverse_bind_matrix_buffer, &descriptor_set);
    
    descriptor_set_writer.build(vulkan.device());
    
    skins.push(Skin {
      name,
      skeleton_root,
      inverse_bind_matrices: matrices,
      joints,
      inverse_bind_matrix_buffer,
      descriptor_set,
    });
  }
}

fn load_images(vulkan: &mut Vulkan, gltf: &gltf::Document, buffers: &Vec<gltf::buffer::Data>, 
               images: &mut Vec<vkimage>) {
  let gltf_images = gltf.images();
  
  for image in gltf_images {
    let image_data = image.source();
    let some_image = {
      match image_data {
        gltf::image::Source::View { ref view, mime_type: _ } => {
          let data = &buffers[view.buffer().index()].0;
          let begin = view.offset();
          let end = begin + view.length();
          let real_data = &data[begin..end];
          
          match image::load_from_memory(real_data) {
            Ok(i) => {
              Some(i.to_rgba8())
            },
            _ => {
              None
            }
          }
        },
        _ => { println!("Image stored in type that cant be loaded atm."); 
          None
        },
      }
    };
    
    if let Some(image) = some_image {
      let loaded_image = TextureHandler::create_device_local_texture_from_image(vulkan, image);
      images.push(loaded_image);
    }
  }
  
  
}


fn load_textures(gltf: &gltf::Document, textures: &mut Vec<Texture>) {
  let gltf_textures = gltf.textures();
  for texture in gltf_textures {
    textures.push(Texture {
      image_index: texture.source().index() as i32,
    });
  }
}

fn load_materials(gltf: &gltf::Document, materials: &mut Vec<Material>) {
  for material in gltf.materials() {
    let pbr = material.pbr_metallic_roughness();
    //println!("Colour: {:?}", pbr.base_color_factor());
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
              index_buffer: &mut Vec<u32>, vertex_buffer: &mut Vec<MeshVertex>, depth: i32) -> Option<Node> {
  let mut first_index = index_buffer.len();
  let mut vertex_start = vertex_buffer.len();
  let mut index_count = 0;
  //let mut vertex_count = 0;
  //println!("Depth: {}", depth);
  let mut node = Node {
    idx: gltf_node.index() as u32,
    children: Vec::new(),
    mesh: Mesh {
      primitives: Vec::new(),
    },
    matrix: Camera::mat4_identity(),
    skin: if let Some(skin) = gltf_node.skin() {
      skin.index() as i32
    } else {
      -1
    },
  };
  
  /*
  let (translation, rotation, scale) = gltf_node.transform().decomposed();
  let mut recomposed_matrix = Camera::mat4_identity();
  recomposed_matrix = Camera::translate(recomposed_matrix, translation);
  recomposed_matrix = Camera::mat4_mul(recomposed_matrix, Camera::quat_to_mat4(rotation));
  recomposed_matrix = Camera::mat4_scale(recomposed_matrix, scale);
  */
  let matrix = gltf_node.transform().matrix();
  
  node.matrix[0] = matrix[0][0];
  node.matrix[1] = matrix[0][1];
  node.matrix[2] = matrix[0][2];
  node.matrix[3] = matrix[0][3];
  
  node.matrix[4] = matrix[1][0];
  node.matrix[5] = matrix[1][1];
  node.matrix[6] = matrix[1][2];
  node.matrix[7] = matrix[1][3];
  
  node.matrix[8] = matrix[2][0];
  node.matrix[9] = matrix[2][1];
  node.matrix[10] = matrix[2][2];
  node.matrix[11] = matrix[2][3];
  
  node.matrix[12] = matrix[3][0];
  node.matrix[13] = matrix[3][1];
  node.matrix[14] = matrix[3][2];
  node.matrix[15] = matrix[3][3];
  
  for child in gltf_node.children() {
    if let Some(new_node) = load_node(&child, buffers, index_buffer, vertex_buffer, depth + 1) {
      node.children.push(new_node);
    }
  }
  
  if let Some(mesh) = gltf_node.mesh() {
    for primitive in mesh.primitives() {
      //println!("prim");
      let mut vertices = Vec::new();
      let mut normals = Vec::new();
      let mut uvs = Vec::new();
      let mut joint_indices = Vec::new();
      let mut joint_weights = Vec::new();
       
      let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
      
      if let Some(iter) = reader.read_positions() {
        //vertex_count = iter.len();
        for position in iter {
          vertices.push([position[0], position[1], position[2]]);
        }
      }
      
      if let Some(iter) = reader.read_normals() {
        for normal in iter {
          normals.push([normal[0], normal[1], normal[2]]);
        }
      }
      
      if let Some(read_tex_coords) = reader.read_tex_coords(0) {
        match read_tex_coords {
          gltf::mesh::util::ReadTexCoords::F32(iter) => {
            for texcoord in iter {
              uvs.push([texcoord[0], texcoord[1]]);
            }
          },
          _ => {
            println!("tex coords is other from f32");
          }
        }
      }
      
      if let Some(some_read_joints) = reader.read_joints(0) {
        match some_read_joints {
          gltf::mesh::util::ReadJoints::U8(read_joints) => {
            for joint in read_joints {
              joint_indices.push([joint[0] as f32, joint[1] as f32, joint[2] as f32, joint[3] as f32]);
            }
          },
          gltf::mesh::util::ReadJoints::U16(read_joints) => {
            for joint in read_joints {
              joint_indices.push([joint[0] as f32, joint[1] as f32, joint[2] as f32, joint[3] as f32]);
            }
          }
        }
        
      }
      
      if let Some(some_read_weights) = reader.read_weights(0) {
        match some_read_weights {
          gltf::mesh::util::ReadWeights::U8(read_weights) => {
            for weights in read_weights {
              joint_weights.push([weights[0] as f32, weights[1] as f32, weights[2] as f32, weights[3] as f32]);
            }
          },
          gltf::mesh::util::ReadWeights::U16(read_weights) => {
            for weights in read_weights {
              joint_weights.push([weights[0] as f32, weights[1] as f32, weights[2] as f32, weights[3] as f32]);
            }
          },
          gltf::mesh::util::ReadWeights::F32(read_weights) => {
            for weights in read_weights {
              joint_weights.push(weights);
            }
          },
        }
      }
      
      if let Some(indices) = reader.read_indices() {
        let indices = indices.into_u32();
        index_count = indices.len();
        
        for index in indices {
          index_buffer.push(index + vertex_start as u32);
        }
      }
      
      let pbr = primitive.material().pbr_metallic_roughness();
      let colour = pbr.base_color_factor();
      
      for i in 0..vertices.len() {
        vertex_buffer.push(
          MeshVertex {
            pos: vertices[i],
            normal: normals[i],
            uv: if uvs.len() <= i { [0.0, 0.0] } else { uvs[i] },
            colour: [colour[0], colour[1], colour[2]],
            joint_indices: if joint_indices.len() <= i { [0.0, 0.0, 0.0, 0.0] } else { joint_indices[i] },
            joint_weights: if joint_weights.len() <= i { [1.0, 1.0, 1.0, 1.0] } else { joint_weights[i] },
          }
        );
      }
      
      let mat_idx = {
        if let Some(idx) = primitive.material().index() {
          idx
        } else {
          0
        }
      };
      
      node.mesh.primitives.push(Primitive {
        first_index: first_index as u32,
        index_count: index_count as u32,
        material_index: mat_idx as i32,
      });
      first_index += index_count;
      
      vertex_start = vertex_buffer.len();
    }
    
    //println!("Primitives: {:?}", node.mesh.primitives);
  }
  
  Some(node)
}
/*
pub fn update_joints(node: &mut Node) {

  if node.skin != -1 {
    let inverse_transform = nodes[i].matrix;
    let skin = skins[nodes[i].skin];
    
    let num_joints = skin.joints.len();
    
    let mut joint_matrices = Vec::new();
    for j in 0..num_joints {
      joint_matrices.push();
    }
  }
  
  for i in 0..node.children().len() {
    gltf_loader::update_joints(&mut node.children()[i])
  }
}*/

pub fn load_gltf(vulkan: &mut Vulkan, sampler: &Sampler, location: &str) -> GltfModel {
  let mut images: Vec<vkimage> = Vec::new();
  let mut textures: Vec<Texture> = Vec::new();
  let mut materials: Vec<Material> = Vec::new();
  let mut nodes: Vec<Node> = Vec::new();
  let mut mesh_animations: Vec<Animation> = Vec::new();
  let mut mesh_skins: Vec<Skin> = Vec::new();
  
  let mut index_buffer = Vec::new();
  let mut vertex_buffer = Vec::new();
  
  let (gltf, buffers, _images) = gltf::import(location).unwrap();
  
  for scene in gltf.scenes() {
    for node in scene.nodes() {
      if let Some(new_node) = load_node(&node, &buffers, &mut index_buffer, &mut vertex_buffer, 1) {
        nodes.push(new_node);
      }
    }
  }
  
  load_materials(&gltf, &mut materials);
  
  load_textures(&gltf, &mut textures);
  
  load_images(vulkan, &gltf, &buffers, &mut images);
  
  let descriptor_pool = DescriptorPoolBuilder::new()
                                            .num_uniform_buffers((images.len() as u32).max(1))
                                            .num_storage((gltf.skins().len() as u32).max(1))
                                            .num_combined_image_samplers((images.len() as u32).max(1))
                                            .build(vulkan.device());
  
  load_skins(vulkan, &gltf, &buffers, &descriptor_pool, &mut nodes, &mut mesh_skins);
  
  load_animation(&gltf, &buffers, &mut mesh_animations); 
  
  let mut mesh_images: Vec<MeshImage> = Vec::new();
  
  for image in images {
    let descriptor_set = DescriptorSet::builder()
                                      .combined_image_sampler_fragment()
                                      .build(vulkan.device(), &descriptor_pool);
    let descriptor_set_writer = DescriptorWriter::builder()
                                                 .update_image(&image, sampler, &descriptor_set);
    
    descriptor_set_writer.build(vulkan.device());
    mesh_images.push(
      MeshImage {
        texture: image,
        descriptor_set,
      }
    );
  }
  
  let mesh_index_buffer = Buffer::<u32>::new_index(&vulkan.device(), index_buffer);
  let mesh_vertex_buffer = Buffer::<MeshVertex>::new_vertex(vulkan.device(), vertex_buffer);
  
   //  for i in 0..nodes.len() {
  
  
  GltfModel {
    nodes,
    mesh_index_buffer,
    mesh_vertex_buffer,
    mesh_images,
    mesh_skins,
    mesh_animations,
    textures,
    materials,
    descriptor_pool,
  }
}
