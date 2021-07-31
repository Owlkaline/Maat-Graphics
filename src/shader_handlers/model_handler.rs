use std::collections::HashMap;
use std::io::Cursor;
use std::mem;

use ash::vk;

use crate::extra::gltf_loader::{CollisionInformation, GltfModel, MeshVertex};
use crate::extra::{gltf_loader, Math};
use crate::offset_of;
use crate::shader_handlers::{Camera, TextureHandler};
use crate::vkwrapper::{
  Buffer, DescriptorPoolBuilder, DescriptorSet, DescriptorWriter, GraphicsPipelineBuilder, Sampler,
  Shader, VkDevice, Vulkan,
};
use crate::DrawMode;

const MAX_INSTANCES: usize = 4096;

#[derive(Clone, Copy)]
pub struct MeshUniformBuffer {
  projection: [f32; 16],
  model: [f32; 16],
  light_pos: [f32; 4],
  window_size: [f32; 2],
}

#[derive(Clone, Copy)]
pub struct InstancedMeshData {
  model: [f32; 16],
  offset: [f32; 3],
  scale: [f32; 3],
}

impl InstancedMeshData {
  pub fn new() -> InstancedMeshData {
    InstancedMeshData {
      model: [0.0; 16],
      offset: [0.0; 3],
      scale: [0.0; 3],
    }
  }
}

pub struct ModelHandler {
  camera: Camera,
  sampler: Sampler,

  models: HashMap<String, GltfModel>,
  mesh_shader: Shader<MeshVertex>,

  //instanced_mesh_shader: Shader<MeshVertex>,
  //instanced_mesh_buffer: HashMap<String, (Buffer<InstancedMeshData>, usize, Vec<(u32, u32)>)>,
  uniform_buffer: Buffer<MeshUniformBuffer>,
  uniform_descriptor_set: DescriptorSet,

  dummy_texture: DescriptorSet,
  dummy_skin_buffer: Buffer<f32>,
  dummy_skin: DescriptorSet,

  storage_descriptor_set: DescriptorSet,

  window_size: [f32; 2],

  descriptor_pool: vk::DescriptorPool,
}

impl ModelHandler {
  pub fn new(vulkan: &mut Vulkan, screen_resolution: vk::Extent2D) -> ModelHandler {
    let descriptor_pool = DescriptorPoolBuilder::new()
      .num_uniform_buffers(30)
      .num_storage(30)
      .num_combined_image_samplers(30)
      .build(vulkan.device());

    let sampler = Sampler::builder()
      .min_filter_linear()
      .mag_filter_linear()
      .address_mode_clamp_to_edge()
      .mipmap_mode_linear()
      .border_colour_float_opaque_white()
      .compare_op_never()
      .build(vulkan.device());

    let descriptor_set0 = DescriptorSet::builder()
      .uniform_buffer_vertex()
      .build(vulkan.device(), &descriptor_pool);
    let descriptor_set1 = DescriptorSet::builder()
      .storage_vertex()
      .build(vulkan.device(), &descriptor_pool);
    let descriptor_set2 = DescriptorSet::builder()
      .combined_image_sampler_fragment()
      .build(vulkan.device(), &descriptor_pool);

    let mesh_shader = ModelHandler::create_mesh_shaders(
      vulkan,
      DrawMode::Polygon,
      vec![
        descriptor_set0.clone(),
        descriptor_set1.clone(),
        descriptor_set2.clone(),
      ],
    );

    let mut camera = Camera::new();
    camera.update_aspect_ratio(screen_resolution.width as f32 / screen_resolution.height as f32);

    let window_size = [
      screen_resolution.width as f32,
      screen_resolution.height as f32,
    ];
    let uniform_data = vec![MeshUniformBuffer {
      projection: camera.perspective_matrix(),
      model: camera.view_matrix(),
      light_pos: [5.0, 5.0, -5.0, 1.0],
      window_size,
    }];

    let uniform_buffer =
      Buffer::<MeshUniformBuffer>::new_uniform_buffer(vulkan.device(), &uniform_data);

    let descriptor_set_writer =
      DescriptorWriter::builder().update_uniform_buffer(&uniform_buffer, &descriptor_set0);

    descriptor_set_writer.build(vulkan.device());

    let image = ModelHandler::create_blank_image();
    let dummy_texture = TextureHandler::create_device_local_texture_from_image(vulkan, image);
    let descriptor_set_writer =
      DescriptorWriter::builder().update_image(&dummy_texture, &sampler, &descriptor_set2);

    descriptor_set_writer.build(vulkan.device());

    let dummy_buffer =
      Buffer::<f32>::new_storage_buffer(vulkan.device(), &Math::mat4_identity().to_vec());
    let dummy_skin = DescriptorSet::builder()
      .storage_vertex()
      .build(vulkan.device(), &descriptor_pool);
    let descriptor_set_writer =
      DescriptorWriter::builder().update_storage_buffer(&dummy_buffer, &dummy_skin);

    descriptor_set_writer.build(vulkan.device());

    ModelHandler {
      camera,
      sampler,

      models: HashMap::new(),
      mesh_shader,

      //instanced_mesh_shader,
      //instanced_mesh_buffer: HashMap::new(),
      uniform_buffer,
      uniform_descriptor_set: descriptor_set0,

      dummy_texture: descriptor_set2,
      dummy_skin_buffer: dummy_buffer,
      dummy_skin,

      storage_descriptor_set: descriptor_set1,

      window_size,

      descriptor_pool,
    }
  }

  pub fn set_draw_mode(&mut self, vulkan: &Vulkan, mode: DrawMode) {
    self.mesh_shader.destroy(vulkan.device());
    //  self.instanced_mesh_shader.destroy(vulkan.device());

    let mesh_shader = ModelHandler::create_mesh_shaders(
      vulkan,
      mode,
      vec![
        self.uniform_descriptor_set.clone(),
        self.storage_descriptor_set.clone(),
        self.dummy_texture.clone(),
      ],
    );

    self.mesh_shader = mesh_shader;
    //self.instanced_mesh_shader = instanced_mesh_shader;
  }

  pub fn create_instance_render_buffer<T: Into<String>>(
    &mut self,
    _vulkan: &mut Vulkan,
    _model_ref: T,
  ) {
    //let mut prim_info = Vec::new();

    //let model_ref = model_ref.into();

    //if let Some(model) = self.models.get_mut(&model_ref.to_string()) {
    //  for i in 0..model.nodes().len() {
    //    for primitive in &model.nodes()[i].mesh.primitives {
    //      let index_count = primitive.index_count;
    //      let first_index = primitive.first_index;
    //      prim_info.push((index_count, first_index));
    //    }
    //  }

    //  let dummy_instanced_data = vec![InstancedMeshData::new(); MAX_INSTANCES];

    //  let buffer = Buffer::<InstancedMeshData>::new_vertex(vulkan.device(), dummy_instanced_data);

    //  self
    //    .instanced_mesh_buffer
    //    .insert(model_ref.to_string(), (buffer, 0, prim_info));
    //}
  }

  pub fn all_collision_models(&self) -> HashMap<String, CollisionInformation> {
    let mut data = HashMap::new();
    for (model_ref, model) in &self.models {
      data.insert(model_ref.to_string(), model.collision_info().clone());
    }

    data
  }

  pub fn model_collision_meshes(&self) -> Vec<(String, Vec<[f32; 3]>, Vec<u32>)> {
    let mut data = Vec::new();
    for (model_ref, model) in &self.models {
      let mut vertices = Vec::new();
      let mut indicies = Vec::new();
      for vertex in model.vertex_buffer().data() {
        vertices.push(vertex.pos);
      }
      for index in model.index_buffer().data() {
        indicies.push(*index);
      }

      data.push((model_ref.to_string(), vertices, indicies));
    }

    data
  }

  pub fn create_blank_image() -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    image::ImageBuffer::from_fn(2, 2, |_x, _y| image::Rgba([255, 255, 255, 255]))
  }

  pub fn window_resized(&mut self, width: u32, height: u32) {
    self.window_size = [width as f32, height as f32];
    self
      .camera
      .update_aspect_ratio(width as f32 / height as f32);
  }

  pub fn update_uniform_buffer(&mut self, device: &VkDevice) {
    let mut data = self.uniform_buffer.data()[0];
    data.projection = self.camera.perspective_matrix();
    data.model = self.camera.view_matrix();
    data.window_size = self.window_size;

    self.uniform_buffer.update_data(device, vec![data]);
  }

  pub fn load_model<T: Into<String>>(&mut self, vulkan: &mut Vulkan, model_ref: T, model: T) {
    let model_ref = model_ref.into();
    let gltf_model = gltf_loader::load_gltf(vulkan, &self.sampler, model_ref.to_string(), model);
    self.models.insert(model_ref, gltf_model);
  }

  pub fn camera(&self) -> &Camera {
    &self.camera
  }

  pub fn mut_camera(&mut self) -> &mut Camera {
    &mut self.camera
  }

  pub fn update_animations(&mut self, vulkan: &mut Vulkan, delta_time: f32) {
    for (_model_ref, model) in &mut self.models {
      model.update_animation(vulkan, delta_time);
    }
  }

  pub fn draw(&mut self, vulkan: &mut Vulkan, data: Vec<f32>, model_ref: &str) {
    if let Some(model) = &self.models.get(model_ref) {
      /*if let Some((buffer, count, _)) = self.instanced_mesh_buffer.get_mut(model_ref) {
        for i in 0..model.nodes().len() {
          let matrix = Node::get_node_matrix(model.nodes(), i);

          buffer.data[*count].model = matrix;
          buffer.data[*count].offset = [data[0], data[1], data[2]];
          buffer.data[*count].scale = [data[4], data[5], data[6]];

          *count += 1;
        }
      } else {*/
      vulkan.draw_mesh(
        &self.mesh_shader,
        &self.uniform_descriptor_set,
        &self.dummy_texture,
        &self.dummy_skin,
        data,
        model,
      );
      //}
    }
  }
  /*
  pub fn draw_instanced_models(&mut self, vulkan: &mut Vulkan) {
    for (model, (buffer, count, prim_info)) in &mut self.instanced_mesh_buffer {
      buffer.update_data(vulkan.device(), buffer.data.clone());

      if let Some(model) = self.models.get(model) {
        vulkan.draw_instanced_mesh(&model.index_buffer(),
                                   model.vertex_buffer(),
                                   &self.instanced_mesh_shader,
                                   &self.uniform_descriptor_set,
                                   &self.dummy_texture,
                                   &self.dummy_skin,
                                   buffer,
                                   *count,
                                   prim_info);
      }
      *count = 0;
    }
  }*/

  fn create_mesh_pipeline_builder(mode: DrawMode) -> GraphicsPipelineBuilder {
    let mut gpb = GraphicsPipelineBuilder::new()
      .topology_triangle_list()
      .polygon_mode_fill()
      .front_face_counter_clockwise()
      .cull_front()
      .samples_1();
    gpb = {
      match mode {
        DrawMode::Polygon => gpb.polygon_mode_fill(),
        DrawMode::Wireframe => gpb.polygon_mode_line(),
        DrawMode::PointsOnly => gpb.polygon_mode_point(),
      }
    };

    gpb
  }

  fn create_mesh_shaders(
    vulkan: &Vulkan,
    draw_mode: DrawMode,
    descriptor_sets: Vec<DescriptorSet>,
  ) -> Shader<MeshVertex> {
    let template_mesh_vertex = MeshVertex {
      pos: [0.0, 0.0, 0.0],
      normal: [0.0, 0.0, 0.0],
      uv: [0.0, 0.0],
      colour: [0.0, 0.0, 0.0],
      joint_indices: [0.0, 0.0, 0.0, 0.0],
      joint_weights: [1.0, 1.0, 1.0, 1.0],
    };

    //let template_instanced_data = InstancedMeshData::new();

    let graphics_pipeline_builder = ModelHandler::create_mesh_pipeline_builder(draw_mode);

    let layouts = {
      let mut sets = Vec::new();
      for i in 0..descriptor_sets.len() {
        sets.push(descriptor_sets[i].layouts()[0]);
      }

      sets
    };

    let mesh_shader = Shader::new(
      vulkan.device(),
      Cursor::new(&include_bytes!("../../shaders/mesh_animated_vert.spv")[..]),
      Cursor::new(&include_bytes!("../../shaders/mesh_pbr_frag.spv")[..]),
      template_mesh_vertex,
      vec![
        offset_of!(MeshVertex, pos) as u32,
        offset_of!(MeshVertex, normal) as u32,
        offset_of!(MeshVertex, uv) as u32,
        offset_of!(MeshVertex, colour) as u32,
        offset_of!(MeshVertex, joint_indices) as u32,
        offset_of!(MeshVertex, joint_weights) as u32,
      ],
      &graphics_pipeline_builder,
      vulkan.model_renderpass(),
      vulkan.viewports(),
      vulkan.scissors(),
      &layouts,
      None as Option<(u32, Vec<u32>)>,
    );
    /*
    let instanced_mesh_shader = Shader::new(
      vulkan.device(),
      Cursor::new(&include_bytes!("../../shaders/instanced_mesh_animated_vert.spv")[..]),
      Cursor::new(&include_bytes!("../../shaders/mesh_animated_frag.spv")[..]),
      template_mesh_vertex,
      vec![
        offset_of!(MeshVertex, pos) as u32,
        offset_of!(MeshVertex, normal) as u32,
        offset_of!(MeshVertex, uv) as u32,
        offset_of!(MeshVertex, colour) as u32,
        offset_of!(MeshVertex, joint_indices) as u32,
        offset_of!(MeshVertex, joint_weights) as u32,
      ],
      &graphics_pipeline_builder,
      vulkan.model_renderpass(),
      vulkan.viewports(),
      vulkan.scissors(),
      &layouts,
      Some((
        template_instanced_data,
        vec![
          offset_of!(InstancedMeshData, model) as u32,
          offset_of!(InstancedMeshData, offset) as u32,
          offset_of!(InstancedMeshData, scale) as u32,
        ],
      )),
    );*/

    mesh_shader
  }
}
