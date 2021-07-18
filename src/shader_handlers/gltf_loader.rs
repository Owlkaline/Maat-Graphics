use crate::modules::{Vulkan, DescriptorSet, DescriptorPoolBuilder, DescriptorWriter, Buffer, Sampler};
use crate::shader_handlers::{Math, TextureHandler};

use crate::Image as vkimage;

use ash::vk;

use gltf;
use gltf::animation::Property;

#[derive(Clone)]
pub enum AnimationInterpolation {
  Linear,
  Step,
  CubicSpline,
}

#[derive(Copy, Clone, Debug)]
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
  skeleton_root: i32,
  inverse_bind_matrices: Vec<[f32; 16]>,
  joints: Vec<i32>,
  inverse_bind_matrix_buffer: Buffer<f32>,
  pub descriptor_set: DescriptorSet,
}

pub struct Animation {
  name: String,
  samplers: Vec<AnimationSampler>,
  channels: Vec<AnimationChannel>,
  start: f32,
  end: f32,
  current_time: f32,
}

pub struct AnimationChannel {
  property: Property,
  node: i32,
  sampler_index: i32,
}

#[derive(Clone)]
pub struct AnimationSampler {
  interpolation: AnimationInterpolation,
  inputs: Vec<f32>,
  outputs: Vec<[f32; 4]>,
}

#[derive(Debug)]
pub struct Primitive {
  pub first_index: u32,
  pub index_count: u32,
  pub material_index: i32,
  pub displacement: [f32; 3],
  pub bounding_box_min: [f32; 3],
  pub bounding_box_max: [f32; 3],
}

#[derive(Debug)]
pub struct Mesh {
  pub primitives: Vec<Primitive>,
}

// idk if keep
#[derive(Debug)]
pub struct Node {
  pub idx: u32,
  pub mesh: Mesh,
  pub skin: i32,
  pub parent: i32,
  pub children: Vec<usize>,
  
  pub translation: [f32; 3],
  pub rotation: [f32; 4], //quaternion
  pub scale: [f32; 3],
  pub matrix: Option<[f32; 16]>,
}

impl Node {
  pub fn calculate_local_matrix(&self) -> [f32; 16] {
    if let Some(matrix) = self.matrix {
      matrix
    } else {
      let scale = Math::mat4_scale(Math::mat4_identity(), self.scale);
      let rotation = Math::quat_to_mat4(self.rotation);
      let translation = Math::mat4_translate_vec3(Math::mat4_identity(), self.translation);
      
      let mut m = Math::mat4_mul(Math::mat4_identity(), translation);
      m = Math::mat4_mul(m, rotation);
      m = Math::mat4_mul(m, scale);
      //m = Math::mat4_mul(m, self.matrix);
      
      m
    }
  }
  
  pub fn get_node_matrix(nodes: &Vec<Node>, idx: usize) -> [f32; 16] {
    let mut matrix = nodes[idx].calculate_local_matrix();
    
    let mut last_parent = nodes[idx].parent;
    while last_parent != -1 {
      let p_matrix = nodes[last_parent as usize].calculate_local_matrix();
      matrix = Math::mat4_mul(p_matrix, matrix);
      //matrix = Math::mat4_mul(matrix, nodes[idx].matrix);
      
      last_parent = nodes[last_parent as usize].parent;
    }
    
    matrix
  }
  
  pub fn matrix(&self) -> [f32; 16] {
    if let Some(matrix) = self.matrix {
      matrix
    } else {
      Math::mat4_identity()
    }
  }
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

pub struct Texture {
  pub image_index: i32,
  pub sampler: Sampler,
}

pub struct GltfModel {
  nodes: Vec<Node>,
  mesh_index_buffer: Buffer<u32>,
  mesh_vertex_buffer: Buffer<MeshVertex>,
  mesh_images: Vec<MeshImage>,
  mesh_skins: Vec<Skin>,
  animations: Vec<Animation>,
  textures: Vec<Texture>,
  materials: Vec<Material>,
  descriptor_pool: vk::DescriptorPool,
  active_animation: i32,
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
  
  pub fn bounds(&self) -> Vec<([f32; 3], [f32; 3], [f32; 3])> {
    let mut bounds = Vec::new();
    for i in 0..self.nodes.len() {
      for j in 0..self.nodes[i].mesh.primitives.len() {
        let displacement = self.nodes[i].mesh.primitives[j].displacement;
        let bb_min = self.nodes[i].mesh.primitives[j].bounding_box_min;
        let bb_max = self.nodes[i].mesh.primitives[j].bounding_box_max;
        
        bounds.push((displacement, bb_min, bb_max));
      }
    }
    
    bounds
  }
  /*
  pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
    let mut min: [f32; 3] = [f32::MAX, f32::MAX, f32::MAX];
    let mut max: [f32; 3] = [f32::MIN, f32::MIN, f32::MIN];
    
    for i in 0..self.nodes.len() {
      for j in 0..self.nodes[i].mesh.primitives.len() {
        let bb_min = self.nodes[i].mesh.primitives[j].bounding_box_min;
        let bb_max = self.nodes[i].mesh.primitives[j].bounding_box_max;
        
        for k in 0..3 {
          if bb_min[k] < min[k] {
            min[k] = bb_min[k];
          }
          
          if bb_max[k] > max[k] {
            max[k] = bb_max[k];
          }
        }
      }
    }
    
    (min, max)
  }*/
  
  pub fn update_animation(&mut self, vulkan: &mut Vulkan, delta_time: f32) {
    if self.active_animation != -1 && self.active_animation < self.animations.len() as i32 {
      
      let anim_idx = self.active_animation as usize;
      
      self.animations[anim_idx].current_time += delta_time;
      if self.animations[anim_idx].current_time > self.animations[anim_idx].end {
        self.animations[anim_idx].current_time -= self.animations[anim_idx].end;
      }
      
      let current_time = self.animations[anim_idx].current_time;
      
      for i in 0..self.animations[anim_idx].channels.len() {
        let sampler = self.animations[anim_idx].samplers[self.animations[anim_idx].channels[i].sampler_index as usize].clone();
        
        for j in 0..sampler.inputs.len()-1 {
          match sampler.interpolation {
            AnimationInterpolation::Linear => {
              if current_time >= sampler.inputs[j] && current_time <= sampler.inputs[j + 1] {
                
                let a = (current_time - sampler.inputs[j]) / (sampler.inputs[j + 1] - sampler.inputs[j]);
                
                let node_idx = self.animations[anim_idx].channels[i].node as usize;
                
                let j_0 = j%sampler.outputs.len();
                let j_1 = (j+1)%sampler.outputs.len();
                
                match self.animations[anim_idx].channels[i].property {
                  Property::Translation => {
                    let translation = Math::vec4_mix(sampler.outputs[j_0], sampler.outputs[j_1], a);
                    self.nodes[node_idx].translation = [translation[0], translation[1], translation[2]];
                  },
                  Property::Rotation => {
                    
                    let q1 = sampler.outputs[j_0];
                    let q2 = sampler.outputs[j_1];
                    
                    self.nodes[node_idx].rotation = Math::vec4_normalise(Math::quat_slerp(q1, q2, a));
                    //self.nodes[node_idx].rotation = Math::vec4_normalise(Math::quat_short_mix(q1, q2, a));
                  },
                  Property::Scale => {
                    let scale = Math::vec4_mix(sampler.outputs[j_0], sampler.outputs[j_1], a);
                    self.nodes[node_idx].scale = [scale[0], scale[1], scale[2]];
                  },
                  _ => {
                    // weights
                  }
                }
              }
            },
            _ => { println!("Warning (model): Only linear interpolation is implemented"); },
          }
        }
      }
      
      for i in 0..self.nodes.len() {
        update_joints(vulkan, &mut self.mesh_skins, &mut self.nodes, i);
      }
    }
  }
}

fn load_animation(gltf: &gltf::Document, buffers: &Vec<gltf::buffer::Data>, 
                  nodes: &Vec<Node>, animations: &mut Vec<Animation>) {
  let gltf_animations = gltf.animations();
  
  for animation in gltf_animations {
    let name = animation.name().unwrap_or("DefaultAnim").to_string();
    let mut animation_start = 10000000000000000000000.0;
    let mut animation_end: f32 = 0.0;
    
    let mut samplers = Vec::new();
    let mut channels = Vec::new();
    
    for channel in animation.channels() {
      let target = channel.target();
      
      let node = {
        let mut node_idx: i32 = -1;
        let target_idx = target.node().index() as u32;
        for i in 0..nodes.len() {
          if nodes[i].idx == target_idx {
            node_idx = i as i32;
            break;
          }
        }
        
        node_idx
      };
      
      let sampler = channel.sampler();
      
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
      
      let in_view = input_accessor.view().unwrap();
      
      let data = &buffers[in_view.buffer().index()].0;
      let begin = in_view.offset();
      let end = begin + in_view.length();
      let input_data_u8 = &data[begin..end];
      
      for bytes in input_data_u8.chunks(4) {
        inputs.push(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
      }
      
      for i in 0..inputs.len() {
        if inputs[i] < animation_start {
          animation_start = inputs[i];
        }
        if inputs[i] > animation_end {
          animation_end = inputs[i];
        }
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
          let mut remaing = Vec::new();
          let left_over = output_data.len() % 3;
          if left_over != 0 {
            for i in (output_data.len()-left_over)..output_data.len() {
              remaing.push(outputs[i]);
            }
          }
          
          for vec3 in output_data.chunks(3) {
            outputs.push([vec3[0], vec3[1], vec3[2], 0.0]);
          }
          
          for remain in remaing {
            outputs.push(remain);
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
      
      let sampler_index = samplers.len() as i32;
      samplers.push(AnimationSampler {
        interpolation,
        inputs,
        outputs,
      });
      
      channels.push(
        AnimationChannel {
          property: target.property(),
          node,
          sampler_index,
        }
      );
    }
    
    animations.push(
      Animation {
        name,
        samplers,
        channels,
        start: animation_start,
        end: animation_end,
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
      let mut root_idx = -1;
      if let Some(root) = skin.skeleton() {
        for i in 0..nodes.len() {
          if nodes[i].idx == root.index() as u32 {
            root_idx = i as i32;
            break;
          }
        }
      }
      
      root_idx
    };
    
    let mut joints = Vec::new();
    for joint in skin.joints() {
      for i in 0..nodes.len() {
        if nodes[i].idx == joint.index() as u32 {
          joints.push(i as i32);
          break;
        }
      }
    }
    
    let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
    
    let mut matrices = Vec::new();
    let mut raw_matrices = Vec::new();
    
    if let Some(inverse_bind_matrices) = reader.read_inverse_bind_matrices() {
      for matrix in inverse_bind_matrices {
        let mut new_matrix = Math::mat4_identity();
        
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


fn load_textures(vulkan: &mut Vulkan, gltf: &gltf::Document, textures: &mut Vec<Texture>) {
  let gltf_textures = gltf.textures();
  for texture in gltf_textures {
    let t_sampler = texture.sampler();
    
    let mut sampler = Sampler::builder().mipmap_mode_linear()
                                        .border_colour_float_opaque_white()
                                        .compare_op_never();
    
    if let Some(min) = t_sampler.min_filter() {
      sampler = {
        match min {
          gltf::texture::MinFilter::Nearest => {
            sampler.min_filter_nearest()
          },
          gltf::texture::MinFilter::Linear => {
            sampler.min_filter_linear()
          },
          _ => {
            sampler.min_filter_linear()
          }
        }
      };
    } else {
      sampler = sampler.min_filter_linear();
    }
    
    if let Some(mag) = t_sampler.mag_filter() {
      sampler = {
        match mag {
          gltf::texture::MagFilter::Nearest => {
            sampler.mag_filter_nearest()
          },
          gltf::texture::MagFilter::Linear => {
            sampler.mag_filter_linear()
          }
        }
      };
    } else {
      sampler = sampler.mag_filter_linear();
    }
    
    match t_sampler.wrap_s() {
      gltf::texture::WrappingMode::ClampToEdge => {
        sampler = sampler.address_mode_clamp_to_edge();
      },
      gltf::texture::WrappingMode::MirroredRepeat => {
        sampler = sampler.address_mode_mirrored_repeat();
      },
      gltf::texture::WrappingMode::Repeat => {
        sampler = sampler.address_mode_repeat();
      }
    }
    
    let sampler = sampler.build(vulkan.device());
    
    textures.push(Texture {
      image_index: texture.source().index() as i32,
      sampler,
    });
  }
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

fn load_node(nodes: &mut Vec<Node>, parent: i32,
             gltf_node: &gltf::Node, buffers: &Vec<gltf::buffer::Data>, 
             index_buffer: &mut Vec<u32>, vertex_buffer: &mut Vec<MeshVertex>) {
  
  let mut first_index = index_buffer.len();
  let mut vertex_start = vertex_buffer.len();
  let mut index_count = 0;
  
  let node_idx = nodes.len();
  
  nodes.push(Node {
    idx: gltf_node.index() as u32,
    mesh: Mesh {
      primitives: Vec::new(),
    },
    skin: if let Some(skin) = gltf_node.skin() {
      skin.index() as i32
    } else {
      -1
    },
    parent,
    children: Vec::new(),
    
    translation: [0.0; 3],
    rotation: [0.0; 4],
    scale: [1.0; 3],
    matrix: None,//Math::mat4_identity(),
  });
  /*
  let matrix = gltf_node.transform().matrix();
  println!("{:?}", matrix);
  if matrix[0][3] != 0.0  ||
     matrix[1][3] != 0.0  ||
     matrix[2][3] != 0.0 ||
     matrix[3][3] != 1.0 {
     
    nodes[node_idx].matrix = Some(Math::mat4_identity());
    if let Some(nmatrix) = &mut nodes[node_idx].matrix {
      /*nodes[node_idx].*/nmatrix[0] = matrix[0][0];
      /*nodes[node_idx].*/nmatrix[1] = matrix[0][1];
      /*nodes[node_idx].*/nmatrix[2] = matrix[0][2];
      /*nodes[node_idx].*/nmatrix[3] = matrix[0][3];
      
      /*nodes[node_idx].*/nmatrix[4] = matrix[1][0];
      /*nodes[node_idx].*/nmatrix[5] = matrix[1][1];
      /*nodes[node_idx].*/nmatrix[6] = matrix[1][2];
      /*nodes[node_idx].*/nmatrix[7] = matrix[1][3];
      
      /*nodes[node_idx].*/nmatrix[8] = matrix[2][0];
      /*nodes[node_idx].*/nmatrix[9] = matrix[2][1];
      /*nodes[node_idx].*/nmatrix[10] = matrix[2][2];
      /*nodes[node_idx].*/nmatrix[11] = matrix[2][3];
      
      /*nodes[node_idx].*/nmatrix[12] = matrix[3][0];
      /*nodes[node_idx].*/nmatrix[13] = matrix[3][1];
      /*nodes[node_idx].*/nmatrix[14] = matrix[3][2];
      /*nodes[node_idx].*/nmatrix[15] = matrix[3][3];
    }
  } else {*/
    let (translation, rotation, scale) = gltf_node.transform().decomposed();
    
    nodes[node_idx].translation = translation;
    nodes[node_idx].rotation = rotation;
    nodes[node_idx].scale = scale;
 // }
  
  for child in gltf_node.children() {
    let child_idx = nodes.len();
    nodes[node_idx].children.push(child_idx);
    load_node(nodes, node_idx as i32, &child, buffers, index_buffer, vertex_buffer);
  }
  
  if let Some(mesh) = gltf_node.mesh() {
    for primitive in mesh.primitives() {
      let mut displacement = [0.0; 3];
      
      let mut vertices = Vec::new();
      let mut normals = Vec::new();
      let mut uvs = Vec::new();
      let mut joint_indices = Vec::new();
      let mut joint_weights = Vec::new();
      
      let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
      
      if let Some(iter) = reader.read_positions() {
        for vertex in iter {
          displacement = Math::vec3_add(displacement, vertex);
          vertices.push(vertex);
        }
      }
      
      if let Some(iter) = reader.read_normals() {
        normals.extend(iter);
      }
      
      if let Some(read_tex_coords) = reader.read_tex_coords(0) {
        match read_tex_coords {
          gltf::mesh::util::ReadTexCoords::F32(iter) => {
            uvs.extend(iter);
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
          gltf::mesh::util::ReadWeights::F32(iter) => {
            joint_weights.extend(iter);
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
      
      let mut b_box_min: [f32; 3] = [0.0; 3];
      let mut b_box_max: [f32; 3] = [0.0; 3];
      
      match primitive.bounding_box() {
        gltf::mesh::BoundingBox { min, max } => {
          b_box_min[0] = min[0]*nodes[node_idx].scale[0];
          b_box_max[0] = max[0]*nodes[node_idx].scale[0];
          b_box_min[1] = min[1]*nodes[node_idx].scale[1];
          b_box_max[1] = max[1]*nodes[node_idx].scale[1];
          b_box_min[2] = min[2]*nodes[node_idx].scale[2];
          b_box_max[2] = max[2]*nodes[node_idx].scale[2];
        }
      }
      
      displacement = Math::vec3_div_f32(displacement, vertices.len() as f32);
      
      nodes[node_idx].mesh.primitives.push(Primitive {
        first_index: first_index as u32,
        index_count: index_count as u32,
        material_index: mat_idx as i32,
        displacement,
        bounding_box_min: b_box_min,
        bounding_box_max: b_box_max,
      });
      first_index += index_count;
      
      vertex_start = vertex_buffer.len();
    }
  }
}
/*
pub fn load_collision_data(gltf: &gltf::Node) {
  for mesh in gltf.meshes() {
    for primitive in mesh.primitives() {
      let mut reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
      
      let mut displacement = [0.0; 3];
      
      let mut verticies = Vec::new();
      let mut indices = Vec::new();

      if let Some(iter) = reader.read_positions() {
        for vertex in iter {
          displacement = Math::vec3_add(displacement, vertex);
          vertices.push(vertex);
        }
      }
      
      if let Some(indices) = reader.read_indices() {
        let indices = indices.into_u32();
        index_count = indices.len();
        
        for index in indices {
          index_buffer.push(index + vertex_start as u32);
        }
      }
      

    }
  }
}
*/
pub fn update_joints(vulkan: &mut Vulkan, skins: &mut Vec<Skin>, nodes: &mut Vec<Node>, idx: usize) {
  if nodes[idx].skin != -1 {
    let matrix = Node::get_node_matrix(nodes, idx);
    
    let inverse_transform = Math::mat4_inverse(matrix);
    let skin_idx = nodes[idx].skin as usize;
    
    let num_joints = skins[skin_idx].joints.len();
    
    let mut joint_matrices = Vec::new();
    for _ in 0..num_joints {
      joint_matrices.push([0.0; 16]);
    }
    
    let mut joint_data: Vec<f32> = Vec::new();
    for i in 0..num_joints {
      let joint_idx = skins[skin_idx].joints[i] as usize;
      let joint_matrix = Node::get_node_matrix(nodes, joint_idx);
      
      joint_matrices[i] = Math::mat4_mul(joint_matrix, skins[skin_idx].inverse_bind_matrices[i]);
      joint_matrices[i] = Math::mat4_mul(inverse_transform, joint_matrices[i]);
      joint_data.append(&mut joint_matrices[i].to_vec());
    }
    
    skins[nodes[idx].skin as usize].inverse_bind_matrix_buffer.update_data(vulkan.device(), joint_data);
  }
}

pub fn load_gltf<T: Into<String>>(vulkan: &mut Vulkan, sampler: &Sampler, location: T) -> GltfModel {
  let mut location = location.into();
  
  let mut images: Vec<vkimage> = Vec::new();
  let mut textures: Vec<Texture> = Vec::new();
  let mut materials: Vec<Material> = Vec::new();
  let mut nodes: Vec<Node> = Vec::new();
  let mut mesh_animations: Vec<Animation> = Vec::new();
  let mut mesh_skins: Vec<Skin> = Vec::new();
  
  let mut index_buffer = Vec::new();
  let mut vertex_buffer = Vec::new();
  
  let (gltf, buffers, _images) = gltf::import(&location.to_string()).unwrap();
  
  for scene in gltf.scenes() {
    for node in scene.nodes() {
      load_node(&mut nodes, -1, &node, &buffers, &mut index_buffer, &mut vertex_buffer);
    }
  }
  
  load_materials(&gltf, &mut materials);
  
  load_textures(vulkan, &gltf, &mut textures);
  
  load_images(vulkan, &gltf, &buffers, &mut images);
  
  let descriptor_pool = DescriptorPoolBuilder::new()
                                            .num_uniform_buffers((images.len() as u32).max(1))
                                            .num_storage((gltf.skins().len() as u32).max(1))
                                            .num_combined_image_samplers((images.len() as u32).max(1))
                                            .build(vulkan.device());
  
  load_skins(vulkan, &gltf, &buffers, &descriptor_pool, &mut nodes, &mut mesh_skins);
  
  load_animation(&gltf, &buffers, &nodes, &mut mesh_animations); 
  
  let mut mesh_images: Vec<MeshImage> = Vec::new();
  
  let mut i = 0;
  for image in images {
    let sampler = {
      let mut s = sampler;
      for j in 0..textures.len() {
        if textures[j].image_index == i as i32 {
          s = &textures[j].sampler;
        }
      }
      
      s
    };
    
    let descriptor_set = DescriptorSet::builder()
                                      .combined_image_sampler_fragment()
                                      .build(vulkan.device(), &descriptor_pool);
    let descriptor_set_writer = DescriptorWriter::builder()
                                                 .update_image(&image, &sampler, &descriptor_set);
    
    descriptor_set_writer.build(vulkan.device());
    mesh_images.push(
      MeshImage {
        texture: image,
        descriptor_set,
      }
    );
    
    i += 1;
  }
  
  let mesh_index_buffer = Buffer::<u32>::new_index(&vulkan.device(), index_buffer);
  let mesh_vertex_buffer = Buffer::<MeshVertex>::new_vertex(vulkan.device(), vertex_buffer);
  
  for i in 0..nodes.len() {
    update_joints(vulkan, &mut mesh_skins, &mut nodes, i);
  }
  
  println!("Animations: {:?}", mesh_animations.len());
  for animation in &mesh_animations {
    println!("    Name: {:?}", animation.name);
  }
  /* 
  location.remove(location.len()-1);
  location.remove(location.len()-1);
  location.remove(location.len()-1);
  location.remove(location.len()-1);
  location = format!("{}_collision.glb", location);
  let (collision_gltf, collision_buffers, _) = gltf::import(location).unwrap();
   
  load_collision_data(collision_gltf);
  
  let collision_vertices: Vec<[f32; 3]> = collision_gltf
    .meshes()
    .flat_map(|mesh| mesh.primitives())
    .flat_map(|primitive| primitive.reader(|buffer| Some(&buffers[buffer.index()])).read_positions())
    .flat_map(|positions| positions)
    .collect();
  
  let collision_indices: Vec<[u32; 3]> = collision_gltf
    .meshes()
    .flat_map(|mesh| mesh.primitives())
    .flat_map(|primitive| primitive.reader(|buffer| Some(&buffers[buffer.index()])).read_indices().unwrap().into_u32())
    .collect::<Vec<u32>>()
    .chunks(3)
    .map(|x| [x[0], x[1], x[2]])
    .collect();
  */
  GltfModel {
    nodes,
    mesh_index_buffer,
    mesh_vertex_buffer,
    mesh_images,
    mesh_skins,
    animations: mesh_animations,
    textures,
    materials,
    descriptor_pool,
    active_animation: 0,
  }
}
