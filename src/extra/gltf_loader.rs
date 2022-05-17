use std::ops::Div;

use ash::vk;
use gltf;
use gltf::animation::Property;

use crate::extra::Math;
use crate::glam::{Mat4, Quat, Vec3};
use crate::shader_handlers::TextureHandler;
use crate::vkwrapper::{
  Buffer, DescriptorPoolBuilder, DescriptorSet, DescriptorWriter, Sampler, VkDevice, Vulkan,
};
use crate::Image as vkimage;

#[derive(Clone)]
pub struct CollisionObject {
  name: String,
  displacement: [f32; 3],
  indices: Vec<u32>,
  positions: Vec<[f32; 3]>,
  min_bounds: [f32; 3],
  max_bounds: [f32; 3],
}

#[derive(Clone)]
pub struct CollisionInformation {
  name: String,
  objects: Vec<CollisionObject>,
  displacement: [f32; 3],
  min_bounds: [f32; 3],
  max_bounds: [f32; 3],
}

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
  inverse_bind_matrices: Vec<Mat4>,
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

//#[derive(Debug)]
pub struct Primitive {
  pub first_index: u32,
  pub index_count: u32,
  pub material_index: i32,
  pub displacement: [f32; 3],
  pub bounding_box_min: [f32; 3],
  pub bounding_box_max: [f32; 3],
}

pub struct Mesh {
  pub primitives: Vec<Primitive>,
}

// idk if keep
pub struct Node {
  pub idx: u32,
  pub mesh: Mesh,
  pub skin: i32,
  pub parent: i32,
  pub children: Vec<usize>,

  // local transform
  pub translation: Vec3,
  pub rotation: Quat, //quaternion
  pub scale: Vec3,

  // global transform
  pub global_translation: Vec3,
  pub global_rotation: Quat,
  pub global_scale: Vec3,
}

#[derive(Clone, Copy)]
pub struct MaterialUbo {
  base_colour_factor: [f32; 4],
  emissive: [f32; 4],
  roughness: f32,
  metallic: f32,
  double_sided: f32,
  pad: f32,
}

pub struct Material {
  descriptor_set: DescriptorSet,
  material_ubo: MaterialUbo,
  base_colour_texture: Option<usize>,
  metallic_roughness_texture: Option<usize>,
  normal_map: Option<usize>,
  occlusion_texture: Option<usize>,
  emissive_texture: Option<usize>,
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
  collision_info: CollisionInformation,

  mesh_index_buffer: Buffer<u32>,
  mesh_vertex_buffer: Buffer<MeshVertex>,
  mesh_skins: Vec<Skin>,

  animations: Vec<Animation>,

  textures: Vec<Texture>,
  materials: Vec<Material>,

  descriptor_pool: vk::DescriptorPool,
  active_animation: i32,
}

impl MaterialUbo {
  pub fn default() -> MaterialUbo {
    MaterialUbo {
      base_colour_factor: [1.0; 4],
      roughness: 0.6,
      metallic: 0.4,
      double_sided: -1.0,
      emissive: [1.0; 4],
      pad: 0.0,
    }
  }
}
impl Material {
  pub fn descriptor(&self) -> &DescriptorSet {
    &self.descriptor_set
  }
}

impl CollisionObject {
  pub fn new<T: Into<String>>(
    name: T,
    displacement: [f32; 3],
    indices: Vec<u32>,
    positions: Vec<[f32; 3]>,
    min_bounds: [f32; 3],
    max_bounds: [f32; 3],
  ) -> CollisionObject {
    CollisionObject {
      name: name.into(),
      displacement,
      indices,
      positions,
      min_bounds,
      max_bounds,
    }
  }

  pub fn name(&self) -> String {
    self.name.to_string()
  }

  pub fn displacement(&self) -> &[f32; 3] {
    &self.displacement
  }

  pub fn indices(&self) -> &Vec<u32> {
    &self.indices
  }

  pub fn vertices(&self) -> &Vec<[f32; 3]> {
    &self.positions
  }

  pub fn min_bounds(&self) -> &[f32; 3] {
    &self.min_bounds
  }

  pub fn max_bounds(&self) -> &[f32; 3] {
    &self.max_bounds
  }
}

impl CollisionInformation {
  pub fn default() -> CollisionInformation {
    CollisionInformation {
      name: format!(""),
      objects: Vec::new(),
      displacement: [0.0; 3],
      min_bounds: [0.0; 3],
      max_bounds: [0.0; 3],
    }
  }

  pub fn new(
    reference: String,
    location: String,
    collision_objects: Vec<CollisionObject>,
  ) -> CollisionInformation {
    let mut location = location;

    location.pop();
    location.pop();
    location.pop();
    location.pop();

    //location = format!("{}_collision.glb", location);

    let mut object_displacement = [0.0; 3];
    let object_min_bounds = [f32::MAX; 3];
    let object_max_bounds = [f32::MIN; 3];

    //if std::path::Path::new(&location).exists() {
    //  collision_objects.clear();

    //  let (gltf, buffers, _images) = gltf::import(&location.to_string()).unwrap();
    //  for scene in gltf.scenes() {
    //    for node in scene.nodes() {
    //      if let Some(mesh) = node.mesh() {
    //        for primitive in mesh.primitives() {
    //          let mut displacement = [0.0; 3];

    //          let mut vertices = Vec::new();
    //          let mut indexs = Vec::new();

    //          let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    //          let (_translation, _rotation, scale) = node.transform().decomposed();

    //          if let Some(iter) = reader.read_positions() {
    //            for vertex in iter {
    //              let scaled_vertex = Math::vec3_mul(vertex, scale);
    //              displacement = Math::vec3_add(displacement, scaled_vertex);
    //              vertices.push(scaled_vertex);
    //            }
    //          }

    //          if let Some(indices) = reader.read_indices() {
    //            let indices_u32 = indices.into_u32();

    //            for index in indices_u32 {
    //              indexs.push(index);
    //            }
    //          }

    //          let mut min_bounds: [f32; 3] = [0.0; 3];
    //          let mut max_bounds: [f32; 3] = [0.0; 3];

    //          match primitive.bounding_box() {
    //            gltf::mesh::BoundingBox { min, max } => {
    //              min_bounds[0] = min[0] * scale[0];
    //              max_bounds[0] = max[0] * scale[0];
    //              min_bounds[1] = min[1] * scale[1];
    //              max_bounds[1] = max[1] * scale[1];
    //              min_bounds[2] = min[2] * scale[2];
    //              max_bounds[2] = max[2] * scale[2];
    //            }
    //          }

    //          for k in 0..3 {
    //            if min_bounds[k] < object_min_bounds[k] {
    //              object_min_bounds[k] = min_bounds[k];
    //            }
    //            if max_bounds[k] < object_max_bounds[k] {
    //              object_max_bounds[k] = max_bounds[k];
    //            }
    //          }

    //          let name = mesh.name().unwrap();

    //          displacement = Math::vec3_div_f32(displacement, vertices.len() as f32);

    //          object_displacement = Math::vec3_add(object_displacement, displacement);

    //          collision_objects.push(CollisionObject::new(
    //            name,
    //            displacement,
    //            indexs,
    //            vertices,
    //            min_bounds,
    //            max_bounds,
    //          ));
    //        }
    //      }
    //    }
    //  }
    //} else {
    for object in &collision_objects {
      object_displacement =
        (Vec3::from(object_displacement) + Vec3::from(*object.displacement())).to_array();
    }
    //}

    object_displacement = Math::vec3_div_f32(object_displacement, collision_objects.len() as f32);

    CollisionInformation {
      name: reference,
      objects: collision_objects,
      displacement: object_displacement,
      min_bounds: object_min_bounds,
      max_bounds: object_max_bounds,
    }
  }

  pub fn objects(&self) -> &Vec<CollisionObject> {
    &self.objects
  }

  pub fn displacement(&self) -> &[f32; 3] {
    &self.displacement
  }

  pub fn min_bounds(&self) -> &[f32; 3] {
    &self.min_bounds
  }

  pub fn max_bounds(&self) -> &[f32; 3] {
    &self.max_bounds
  }
}

impl Node {
  pub fn calculate_global_matrix(
    nodes: &[Node],
    idx: usize,
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
  ) -> Mat4 {
    let translation = nodes[idx].global_translation + translation;
    let rotation = nodes[idx].global_rotation * rotation;
    let scale = nodes[idx].global_scale * scale;

    Mat4::from_scale_rotation_translation(scale, rotation, translation)
  }

  pub fn calculate_all_global_transforms(nodes: &mut Vec<Node>) {
    let parent_ids: Vec<usize> = nodes
      .iter()
      .enumerate()
      .filter_map(|(i, x)| if x.parent == -1 { Some(i) } else { None })
      .collect();

    for parent_idx in parent_ids {
      nodes[parent_idx].global_translation = nodes[parent_idx].translation;
      nodes[parent_idx].global_rotation = nodes[parent_idx].rotation;
      nodes[parent_idx].global_scale = nodes[parent_idx].scale;

      let translation = nodes[parent_idx].global_translation;
      let rotation = nodes[parent_idx].global_rotation;
      let scale = nodes[parent_idx].global_scale;

      for child_idx in nodes[parent_idx].children.clone() {
        Node::calculate_recursive(translation, rotation, scale, nodes, child_idx);
      }
    }
  }

  fn calculate_recursive(
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
    nodes: &mut Vec<Node>,
    idx: usize,
  ) {
    nodes[idx].global_translation = translation * nodes[idx].translation;
    nodes[idx].global_rotation = rotation * nodes[idx].rotation;
    nodes[idx].global_scale = scale * nodes[idx].scale;

    let translation = nodes[idx].global_translation;
    let rotation = nodes[idx].global_rotation;
    let scale = nodes[idx].global_scale;

    for child_idx in nodes[idx].children.clone() {
      Node::calculate_recursive(translation, rotation, scale, nodes, child_idx);
    }
  }

  //pub fn calculate_local_matrix(&self) -> Mat4 {
  //  if let Some(matrix) = self.matrix {
  //    matrix
  //  } else {
  //    Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
  //  }
  //}

  //pub fn get_node_matrix(nodes: &Vec<Node>, idx: usize) -> [f32; 16] {
  //  let mut matrix = nodes[idx].calculate_local_matrix();

  //  let mut last_parent = nodes[idx].parent;
  //  while last_parent != -1 {
  //    let p_matrix = nodes[last_parent as usize].calculate_local_matrix();
  //    matrix = p_matrix * matrix;
  //    //matrix = Math::mat4_mul(matrix, nodes[idx].matrix);

  //    last_parent = nodes[last_parent as usize].parent;
  //  }

  //  matrix.to_cols_array()
  //}

  //pub fn matrix(&self) -> Mat4 {
  //  if let Some(matrix) = self.matrix {
  //    matrix
  //  } else {
  //    Mat4::IDENTITY
  //  }
  //}
}

impl GltfModel {
  pub fn mesh_descriptor(device: &VkDevice, descriptor_pool: &vk::DescriptorPool) -> DescriptorSet {
    DescriptorSet::builder()
      .uniform_buffer_fragment()
      .combined_image_sampler_fragment()
      .combined_image_sampler_fragment()
      .combined_image_sampler_fragment()
      .combined_image_sampler_fragment()
      .combined_image_sampler_fragment()
      .build(device, descriptor_pool)
  }

  pub fn nodes(&self) -> &Vec<Node> {
    &self.nodes
  }

  //pub fn images(&self) -> &Vec<MeshImage> {
  //  &self.mesh_images
  //}

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

  pub fn collision_info(&self) -> &CollisionInformation {
    &self.collision_info
  }

  pub fn update_animation(&mut self, _vulkan: &mut Vulkan, delta_time: f32) {
    if self.active_animation != -1 && self.active_animation < self.animations.len() as i32 {
      let anim_idx = self.active_animation as usize;

      self.animations[anim_idx].current_time += delta_time;
      if self.animations[anim_idx].current_time > self.animations[anim_idx].end {
        self.animations[anim_idx].current_time -= self.animations[anim_idx].end;
      }

      let current_time = self.animations[anim_idx].current_time;

      for i in 0..self.animations[anim_idx].channels.len() {
        let sampler = self.animations[anim_idx].samplers
          [self.animations[anim_idx].channels[i].sampler_index as usize]
          .clone();

        for j in 0..sampler.inputs.len() - 1 {
          match sampler.interpolation {
            AnimationInterpolation::Linear => {
              if current_time >= sampler.inputs[j] && current_time <= sampler.inputs[j + 1] {
                let a =
                  (current_time - sampler.inputs[j]) / (sampler.inputs[j + 1] - sampler.inputs[j]);

                let node_idx = self.animations[anim_idx].channels[i].node as usize;

                let j_0 = j % sampler.outputs.len();
                let j_1 = (j + 1) % sampler.outputs.len();

                match self.animations[anim_idx].channels[i].property {
                  Property::Translation => {
                    let translation = Math::vec4_mix(sampler.outputs[j_0], sampler.outputs[j_1], a);
                    self.nodes[node_idx].translation =
                      Vec3::from([translation[0], translation[1], translation[2]]);
                  }
                  Property::Rotation => {
                    let q1 = sampler.outputs[j_0];
                    let q2 = sampler.outputs[j_1];

                    let q1 = Quat::from_array(q1);
                    let q2 = Quat::from_array(q2);

                    self.nodes[node_idx].rotation = Quat::slerp(q1, q2, a);
                    //Math::vec4_normalise(Math::quat_slerp(q1, q2, a));
                    //self.nodes[node_idx].rotation = Math::vec4_normalise(Math::quat_short_mix(q1, q2, a));
                  }
                  Property::Scale => {
                    let scale = Math::vec4_mix(sampler.outputs[j_0], sampler.outputs[j_1], a);
                    self.nodes[node_idx].scale = Vec3::from([scale[0], scale[1], scale[2]]);
                  }
                  _ => {
                    // weights
                  }
                }
              }
            }
            _ => {
              println!("Warning (model): Only linear interpolation is implemented");
            }
          }
        }
      }

      Node::calculate_all_global_transforms(&mut self.nodes);

      //for i in 0..self.nodes.len() {
      //  update_joints(vulkan, &mut self.mesh_skins, &mut self.nodes, i);
      //}
    }
  }
}

fn load_animation(
  gltf: &gltf::Document,
  buffers: &[gltf::buffer::Data],
  nodes: &[Node],
  animations: &mut Vec<Animation>,
) {
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
        for (i, node) in nodes.iter().enumerate() {
          if node.idx == target_idx {
            node_idx = i as i32;
            break;
          }
        }

        node_idx
      };

      let sampler = channel.sampler();

      let interpolation = {
        match sampler.interpolation() {
          gltf::animation::Interpolation::Linear => AnimationInterpolation::Linear,
          gltf::animation::Interpolation::Step => AnimationInterpolation::Step,
          gltf::animation::Interpolation::CubicSpline => AnimationInterpolation::CubicSpline,
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

      for input in &inputs {
        if *input < animation_start {
          animation_start = *input;
        }
        if *input > animation_end {
          animation_end = *input;
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
            for i in (output_data.len() - left_over)..output_data.len() {
              remaing.push(outputs[i]);
            }
          }

          for vec3 in output_data.chunks(3) {
            outputs.push([vec3[0], vec3[1], vec3[2], 0.0]);
          }

          for remain in remaing {
            outputs.push(remain);
          }
        }
        gltf::accessor::Dimensions::Vec4 => {
          for vec4 in output_data.chunks(4) {
            outputs.push([vec4[0], vec4[1], vec4[2], vec4[3]]);
          }
        }
        _ => {}
      }

      let sampler_index = samplers.len() as i32;
      samplers.push(AnimationSampler {
        interpolation,
        inputs,
        outputs,
      });

      channels.push(AnimationChannel {
        property: target.property(),
        node,
        sampler_index,
      });
    }

    animations.push(Animation {
      name,
      samplers,
      channels,
      start: animation_start,
      end: animation_end,
      current_time: 0.0,
    });
  }
}

fn load_skins(
  vulkan: &mut Vulkan,
  gltf: &gltf::Document,
  buffers: &[gltf::buffer::Data],
  descriptor_pool: &vk::DescriptorPool,
  nodes: &mut Vec<Node>,
  skins: &mut Vec<Skin>,
) {
  let gltf_skins = gltf.skins();

  let mut nodes_updated = Vec::new();

  for skin in gltf_skins {
    for i in 0..nodes.len() as usize {
      if !nodes_updated.contains(&i)
        && nodes[i].skin != -1
        && skin.index() == nodes[i].skin as usize
      {
        nodes[i].skin = skins.len() as i32;
        nodes_updated.push(i);
      }
    }

    let name = skin.name().unwrap_or("").to_string();

    let skeleton_root = {
      let mut root_idx = -1;
      if let Some(root) = skin.skeleton() {
        for (i, node) in nodes.iter().enumerate() {
          if node.idx == root.index() as u32 {
            root_idx = i as i32;
            break;
          }
        }
      }

      root_idx
    };

    let mut joints = Vec::new();
    for joint in skin.joints() {
      for (i, node) in nodes.iter().enumerate() {
        if node.idx == joint.index() as u32 {
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
        let new_matrix = Mat4::from_cols_array_2d(&matrix);
        matrices.push(new_matrix);
        raw_matrices.append(&mut new_matrix.to_cols_array().to_vec());
      }
    }

    let inverse_bind_matrix_buffer =
      Buffer::<f32>::new_storage_buffer(vulkan.device(), &raw_matrices);
    let descriptor_set = DescriptorSet::builder()
      .storage_vertex()
      .build(vulkan.device(), descriptor_pool);
    let descriptor_set_writer =
      DescriptorWriter::builder().update_buffer(&inverse_bind_matrix_buffer, &descriptor_set);

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

fn load_images(
  vulkan: &mut Vulkan,
  gltf: &gltf::Document,
  buffers: &[gltf::buffer::Data],
  images: &mut Vec<vkimage>,
) {
  let gltf_images = gltf.images();

  for image in gltf_images {
    let image_data = image.source();
    let some_image = {
      match image_data {
        gltf::image::Source::View {
          ref view,
          mime_type: _,
        } => {
          let data = &buffers[view.buffer().index()].0;
          let begin = view.offset();
          let end = begin + view.length();
          let real_data = &data[begin..end];

          match image::load_from_memory(real_data) {
            Ok(i) => Some(i.to_rgba8()),
            _ => None,
          }
        }
        _ => {
          println!("Image stored in type that cant be loaded atm.");
          None
        }
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

    let mut sampler = Sampler::builder()
      .mipmap_mode_linear()
      .border_colour_float_opaque_white()
      .compare_op_never();

    if let Some(min) = t_sampler.min_filter() {
      sampler = {
        match min {
          gltf::texture::MinFilter::Nearest => sampler.min_filter_nearest(),
          gltf::texture::MinFilter::Linear => sampler.min_filter_linear(),
          _ => sampler.min_filter_linear(),
        }
      };
    } else {
      sampler = sampler.min_filter_linear();
    }

    if let Some(mag) = t_sampler.mag_filter() {
      sampler = {
        match mag {
          gltf::texture::MagFilter::Nearest => sampler.mag_filter_nearest(),
          gltf::texture::MagFilter::Linear => sampler.mag_filter_linear(),
        }
      };
    } else {
      sampler = sampler.mag_filter_linear();
    }

    match t_sampler.wrap_s() {
      gltf::texture::WrappingMode::ClampToEdge => {
        sampler = sampler.address_mode_clamp_to_edge();
      }
      gltf::texture::WrappingMode::MirroredRepeat => {
        sampler = sampler.address_mode_mirrored_repeat();
      }
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

fn load_material(
  vulkan: &mut Vulkan,
  descriptor_pool: &vk::DescriptorPool,
  gltf: &gltf::Document,
  materials: &mut Vec<Material>,
  textures: &[Texture],
  mesh_images: &[vkimage],
  dummy_image: &vkimage,
  dummy_sampler: &Sampler,
) {
  for material in gltf.materials() {
    let pbr = material.pbr_metallic_roughness();

    let base_colour_factor = pbr.base_color_factor();
    let emissive = material.emissive_factor();
    let roughness = pbr.roughness_factor();
    let metallic = pbr.metallic_factor();
    let double_sided = material.double_sided();

    let material_ubo = MaterialUbo {
      base_colour_factor,
      emissive: [emissive[0], emissive[1], emissive[2], 1.0],
      roughness,
      metallic,
      double_sided: if double_sided { 1.0 } else { -1.0 },
      pad: 0.0,
    };
    let material_buffer =
      Buffer::<MaterialUbo>::new_uniform_buffer(vulkan.device(), &vec![material_ubo]);

    let descriptor_set = GltfModel::mesh_descriptor(vulkan.device(), descriptor_pool);

    let mut images: Vec<vkimage> = Vec::new();
    let mut samplers = Vec::new();

    let mut descriptor_set_writer =
      DescriptorWriter::builder().update_buffer(&material_buffer, &descriptor_set);

    let base_colour_texture = if let Some(info) = pbr.base_color_texture() {
      let label = info.texture().index() as usize;
      let sampler = &textures[label].sampler;
      images.push(mesh_images[textures[label].image_index as usize].clone());
      samplers.push(sampler.clone());
      Some(label)
    } else {
      images.push(dummy_image.clone());
      samplers.push(dummy_sampler.clone());
      None
    };

    let normal_map = if let Some(normal_texture) = material.normal_texture() {
      let label = normal_texture.texture().index() as usize;
      let sampler = &textures[label].sampler;
      images.push(mesh_images[label].clone());
      samplers.push(sampler.clone());
      Some(label)
    } else {
      images.push(dummy_image.clone());
      samplers.push(dummy_sampler.clone());
      None
    };

    let metallic_roughness_texture = if let Some(info) = pbr.metallic_roughness_texture() {
      let label = info.texture().index() as usize;
      let sampler = &textures[label].sampler;
      images.push(mesh_images[label].clone());
      samplers.push(sampler.clone());
      Some(label)
    } else {
      images.push(dummy_image.clone());
      samplers.push(dummy_sampler.clone());
      None
    };

    let occlusion_texture = if let Some(occlusion_texture) = material.occlusion_texture() {
      let label = occlusion_texture.texture().index() as usize;
      let sampler = &textures[label].sampler;
      images.push(mesh_images[label].clone());
      samplers.push(sampler.clone());
      Some(label)
    } else {
      images.push(dummy_image.clone());
      samplers.push(dummy_sampler.clone());
      None
    };

    let emissive_texture = if let Some(info) = material.emissive_texture() {
      let label = info.texture().index() as usize;
      let sampler = &textures[label].sampler;
      images.push(mesh_images[label].clone());
      samplers.push(sampler.clone());
      Some(label)
    } else {
      images.push(dummy_image.clone());
      samplers.push(dummy_sampler.clone());
      None
    };

    descriptor_set_writer =
      descriptor_set_writer.update_images(&images, &samplers, &descriptor_set);
    descriptor_set_writer.build(vulkan.device());

    materials.push(Material {
      descriptor_set,
      material_ubo,
      base_colour_texture,
      metallic_roughness_texture,
      normal_map,
      occlusion_texture,
      emissive_texture,
    });
  }
}

fn load_node(
  nodes: &mut Vec<Node>,
  parent: i32,
  gltf_node: &gltf::Node,
  collision_objects: &mut Vec<CollisionObject>,
  buffers: &[gltf::buffer::Data],
  index_buffer: &mut Vec<u32>,
  vertex_buffer: &mut Vec<MeshVertex>,
) {
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

    translation: Vec3::ZERO,
    rotation: Quat::IDENTITY,
    scale: Vec3::ONE,

    global_translation: Vec3::ZERO,
    global_rotation: Quat::IDENTITY,
    global_scale: Vec3::ONE,
  });

  match gltf_node.transform() {
    gltf::scene::Transform::Matrix { matrix } => {
      let (translation, rotation, scale) =
        Mat4::from_cols_array_2d(&matrix).to_scale_rotation_translation();

      nodes[node_idx].translation = translation;
      nodes[node_idx].rotation = rotation;
      nodes[node_idx].scale = scale;
    }
    gltf::scene::Transform::Decomposed {
      translation,
      rotation,
      scale,
    } => {
      nodes[node_idx].translation = Vec3::from(translation);
      nodes[node_idx].rotation = Quat::from_array(rotation);
      nodes[node_idx].scale = Vec3::from(scale);
    }
  }

  for child in gltf_node.children() {
    let child_idx = nodes.len();
    nodes[node_idx].children.push(child_idx);
    load_node(
      nodes,
      node_idx as i32,
      &child,
      collision_objects,
      buffers,
      index_buffer,
      vertex_buffer,
    );
  }

  if let Some(mesh) = gltf_node.mesh() {
    for primitive in mesh.primitives() {
      let mut displacement = [0.0; 3];

      let mut vertices = Vec::new();
      let mut normals = Vec::new();
      let mut uvs = Vec::new();
      let mut colours = Vec::new();
      let mut joint_indices = Vec::new();
      let mut joint_weights = Vec::new();

      let mut all_verticies = Vec::new();
      let mut all_indices = Vec::new();

      let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

      if let Some(vertex_attribute) = reader
        .read_positions()
        .map(|v| v.collect::<Vec<[f32; 3]>>())
      {
        vertices = vertex_attribute;
      }

      if let Some(colour_attribute) = reader
        .read_colors(0)
        .map(|c| c.into_rgb_f32().collect::<Vec<[f32; 3]>>())
      {
        colours = colour_attribute;
      }

      if let Some(normal_attribute) = reader.read_normals().map(|n| n.collect::<Vec<[f32; 3]>>()) {
        normals = normal_attribute;
      }

      if let Some(tex_coords_0) = reader
        .read_tex_coords(0)
        .map(|tc| tc.into_f32().collect::<Vec<[f32; 2]>>())
      {
        uvs = tex_coords_0;
      }

      if let Some(some_read_joints) = reader.read_joints(0) {
        match some_read_joints {
          gltf::mesh::util::ReadJoints::U8(read_joints) => {
            for joint in read_joints {
              joint_indices.push([
                joint[0] as f32,
                joint[1] as f32,
                joint[2] as f32,
                joint[3] as f32,
              ]);
            }
          }
          gltf::mesh::util::ReadJoints::U16(read_joints) => {
            for joint in read_joints {
              joint_indices.push([
                joint[0] as f32,
                joint[1] as f32,
                joint[2] as f32,
                joint[3] as f32,
              ]);
            }
          }
        }
      }

      if let Some(some_read_weights) = reader.read_weights(0) {
        match some_read_weights {
          gltf::mesh::util::ReadWeights::U8(read_weights) => {
            for weights in read_weights {
              joint_weights.push([
                weights[0] as f32,
                weights[1] as f32,
                weights[2] as f32,
                weights[3] as f32,
              ]);
            }
          }
          gltf::mesh::util::ReadWeights::U16(read_weights) => {
            for weights in read_weights {
              joint_weights.push([
                weights[0] as f32,
                weights[1] as f32,
                weights[2] as f32,
                weights[3] as f32,
              ]);
            }
          }
          gltf::mesh::util::ReadWeights::F32(iter) => {
            joint_weights.extend(iter);
          }
        }
      }

      if let Some(indices) = reader.read_indices() {
        let indices = indices.into_u32();
        index_count = indices.len();

        for index in indices {
          index_buffer.push(index + vertex_start as u32);
          all_indices.push(index);
        }
      }

      for i in 0..vertices.len() {
        all_verticies.push((Vec3::from(vertices[i]) * nodes[node_idx].scale).to_array());
        displacement = Math::vec3_add(displacement, vertices[i]);

        vertex_buffer.push(MeshVertex {
          pos: vertices[i],
          normal: normals[i],
          uv: if uvs.len() <= i { [0.0, 0.0] } else { uvs[i] },
          colour: if colours.len() <= i {
            [1.0, 1.0, 1.0]
          } else {
            colours[i]
          },
          joint_indices: if joint_indices.len() <= i {
            [0.0, 0.0, 0.0, 0.0]
          } else {
            joint_indices[i]
          },
          joint_weights: if joint_weights.len() <= i {
            [1.0, 1.0, 1.0, 1.0]
          } else {
            joint_weights[i]
          },
        });
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
          b_box_min[0] = min[0] * nodes[node_idx].scale[0];
          b_box_max[0] = max[0] * nodes[node_idx].scale[0];
          b_box_min[1] = min[1] * nodes[node_idx].scale[1];
          b_box_max[1] = max[1] * nodes[node_idx].scale[1];
          b_box_min[2] = min[2] * nodes[node_idx].scale[2];
          b_box_max[2] = max[2] * nodes[node_idx].scale[2];
        }
      }

      displacement = (Vec3::from(displacement) * nodes[node_idx].scale)
        .div(vertices.len() as f32)
        .to_array();

      let name = mesh.name().unwrap();

      collision_objects.push(CollisionObject::new(
        name,
        displacement,
        all_indices,
        all_verticies,
        b_box_min,
        b_box_max,
      ));

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

pub fn update_joints(
  vulkan: &mut Vulkan,
  skins: &mut Vec<Skin>,
  nodes: &mut Vec<Node>,
  idx: usize,
) {
  if nodes[idx].skin != -1 {
    let matrix = Node::calculate_global_matrix(nodes, idx, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE); //&Node::get_node_matrix(nodes, idx));

    let inverse_transform = matrix.inverse(); //Math::mat4_inverse(matrix);
    let skin_idx = nodes[idx].skin as usize;

    let num_joints = skins[skin_idx].joints.len();

    let mut joint_matrices = Vec::new();
    for _ in 0..num_joints {
      joint_matrices.push([0.0; 16]);
    }

    let mut joint_data: Vec<f32> = Vec::new();
    for i in 0..num_joints {
      let joint_idx = skins[skin_idx].joints[i] as usize;
      let joint_matrix =
        Node::calculate_global_matrix(nodes, joint_idx, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE); //&Node::get_node_matrix(nodes, joint_idx));

      joint_matrices[i] = (inverse_transform
        * (joint_matrix * skins[skin_idx].inverse_bind_matrices[i]))
        .to_cols_array();
      //      joint_matrices[i] = Math::mat4_mul(joint_matrix, skins[skin_idx].inverse_bind_matrices[i]);
      //      joint_matrices[i] = Math::mat4_mul(inverse_transform, joint_matrices[i]);
      joint_data.append(&mut joint_matrices[i].to_vec());
    }

    skins[nodes[idx].skin as usize]
      .inverse_bind_matrix_buffer
      .update_data(vulkan.device(), joint_data);
  }
}

pub fn load_gltf<T: Into<String>>(
  vulkan: &mut Vulkan,
  sampler: &Sampler,
  dummy_texture: &vkimage,
  reference: T,
  location: &[u8], //T,
) -> GltfModel {
  //let location = location.into();

  let reference = reference.into();
  println!("Loading model: {}", reference.to_string());

  let mut images: Vec<vkimage> = Vec::new();
  let mut textures: Vec<Texture> = Vec::new();
  let mut materials: Vec<Material> = Vec::new();
  let mut nodes: Vec<Node> = Vec::new();
  let mut mesh_animations: Vec<Animation> = Vec::new();
  let mut mesh_skins: Vec<Skin> = Vec::new();

  let mut index_buffer = Vec::new();
  let mut vertex_buffer = Vec::new();

  let mut collision_objects = Vec::new();

  //let dummy_image = TextureHandler::create_blank_image();
  //let dummy_texture = TextureHandler::create_device_local_texture_from_image(vulkan, dummy_image);
  //let image_view_info = dummy_texture.build_imageview(&dummy_texture.internal());

  let (gltf, buffers, _images) = gltf::import_slice(location).unwrap();

  let mut descriptor_pool = DescriptorPoolBuilder::new()
    .num_uniform_buffers((images.len() as u32).max(1))
    .num_storage((gltf.skins().len() as u32).max(1))
    .num_combined_image_samplers((images.len() as u32).max(1) * 100)
    .build(vulkan.device());

  for scene in gltf.scenes() {
    for node in scene.nodes() {
      load_node(
        &mut nodes,
        -1,
        &node,
        &mut collision_objects,
        &buffers,
        &mut index_buffer,
        &mut vertex_buffer,
      );
    }
  }

  load_textures(vulkan, &gltf, &mut textures);
  load_images(vulkan, &gltf, &buffers, &mut images);
  load_material(
    vulkan,
    &mut descriptor_pool,
    &gltf,
    &mut materials,
    &mut textures,
    &images,
    &dummy_texture,
    sampler,
  );
  load_skins(
    vulkan,
    &gltf,
    &buffers,
    &descriptor_pool,
    &mut nodes,
    &mut mesh_skins,
  );
  load_animation(&gltf, &buffers, &nodes, &mut mesh_animations);

  let mesh_index_buffer = Buffer::<u32>::new_index(&vulkan.device(), index_buffer);
  let mesh_vertex_buffer = Buffer::<MeshVertex>::new_vertex(vulkan.device(), vertex_buffer);

  for i in 0..nodes.len() {
    update_joints(vulkan, &mut mesh_skins, &mut nodes, i);
  }

  let collision_info = CollisionInformation::new(
    reference.into(),
    /*location*/ "".to_string(),
    collision_objects,
  );

  Node::calculate_all_global_transforms(&mut nodes);

  GltfModel {
    nodes,
    collision_info,

    mesh_index_buffer,
    mesh_vertex_buffer,
    mesh_skins,

    animations: mesh_animations,

    textures,
    materials,

    descriptor_pool,
    active_animation: 0, //8, //2, //5,
  }
}
