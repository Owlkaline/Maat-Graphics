use std::io::Cursor;

use ash::vk;
use glam::Mat4;

use crate::{
  shader_handlers::Camera,
  vkwrapper::{
    Buffer, ComputeShader, DescriptorPoolBuilder, DescriptorSet, DescriptorWriter, Vulkan,
  },
};

const TILE_SIZE: i32 = 16;

#[derive(Clone, Copy)]
pub struct LightVisibility {
  count: u32,
  lights: [u32; 16],
}

#[derive(Clone, Copy)]
pub struct PointLight {
  pos: [f32; 3],
  radius: f32,
  intensity: [f32; 3],
  padding: f32,
}

#[derive(Clone, Copy)]
pub struct CameraUbo {
  view: [f32; 16],
  proj: [f32; 16],
  proj_view: [f32; 16],
  cam_pos: [f32; 3],
}

pub struct PushConstantObject {
  viewport_size: [i32; 2],
  tile_nums: [i32; 2],
}

pub struct ComputeHandler {
  compute_shader: ComputeShader,

  light_visibility_buffer: Buffer<LightVisibility>,
  point_light_buffer: Buffer<PointLight>,
  camera_buffer: Buffer<CameraUbo>,

  light_culling_descriptor_set: DescriptorSet,
  camera_descriptor_set: DescriptorSet,
  intermediate_descriptor_set: DescriptorSet,

  descriptor_pool: vk::DescriptorPool,
}

impl ComputeHandler {
  pub fn new(vulkan: &mut Vulkan, camera: &Camera) -> ComputeHandler {
    let descriptor_pool = DescriptorPoolBuilder::new()
      .num_storage(5)
      .build(vulkan.device());

    let intermediate_descriptor_set = DescriptorSet::builder()
      .combined_image_sampler_compute_fragment() // intermediate descriptor set
      .build(vulkan.device(), &descriptor_pool);

    let light_culling_descriptor_set = DescriptorSet::builder()
      .storage_compute_fragment() // light culling descriptor set
      .storage_compute_fragment() // ligth visibility
      .build(vulkan.device(), &descriptor_pool);

    let camera_descriptor_set = DescriptorSet::builder()
      .storage_vertex_compute_fragment() // Camera descriptor set
      .build(vulkan.device(), &descriptor_pool);

    let compute_shader = ComputeShader::new(
      vulkan.device(),
      Cursor::new(&include_bytes!("../../shaders/light_culling_comp.spv")[..]),
      &vec![
        light_culling_descriptor_set.clone(),
        camera_descriptor_set.clone(),
        intermediate_descriptor_set.clone(),
      ],
    );

    let light_visibility_data = LightVisibility {
      count: 1,
      lights: [0; 16],
    };

    let point_light_data = PointLight {
      pos: [1.0, 1.0, 1.0],
      radius: 5.0,
      intensity: [1.0, 1.0, 1.0],
      padding: 0.0,
    };
    let camera_data = CameraUbo {
      view: camera.view_matrix().into(),
      proj: camera.perspective_matrix(),
      proj_view: (Mat4::from_cols_array(&camera.view_matrix()) *
        Mat4::from_cols_array(&camera.perspective_matrix()))
      .to_cols_array(),
      cam_pos: camera.position().into(),
    };
    let light_visibility_buffer =
      Buffer::<LightVisibility>::new_storage_buffer(&vulkan.device(), &vec![light_visibility_data]);
    let point_light_buffer =
      Buffer::<PointLight>::new_storage_buffer(&vulkan.device(), &vec![point_light_data]);
    let camera_buffer =
      Buffer::<CameraUbo>::new_storage_buffer(vulkan.device(), &vec![camera_data]);

    let uniform_descriptor_set_writer = DescriptorWriter::builder()
      .update_buffer(&light_visibility_buffer, &light_culling_descriptor_set)
      .update_buffer(&point_light_buffer, &light_culling_descriptor_set);

    uniform_descriptor_set_writer.build(vulkan.device());

    let uniform_descriptor_set_writer =
      DescriptorWriter::builder().update_buffer(&camera_buffer, &camera_descriptor_set);

    uniform_descriptor_set_writer.build(vulkan.device());

    ComputeHandler {
      compute_shader,

      light_visibility_buffer,
      point_light_buffer,
      camera_buffer,

      light_culling_descriptor_set,
      camera_descriptor_set,
      intermediate_descriptor_set,

      descriptor_pool,
    }
  }

  pub fn run(&mut self, vulkan: &mut Vulkan) {
    let width = vulkan.swapchain().extent().width as i32;
    let height = vulkan.swapchain().extent().height as i32;

    let tile_count_per_row = (width - 1) / TILE_SIZE + 1;
    let tile_count_per_col = (height - 1) / TILE_SIZE + 1;

    let mut push_constants = [0 as i32; 16];
    push_constants[0] = width;
    push_constants[1] = height;
    push_constants[2] = tile_count_per_row;
    push_constants[3] = tile_count_per_col;

    //let push_constants = PushConstantObject {
    //  viewport_size: [width, height],
    //  tile_nums: [tile_count_per_row, tile_count_per_col],
    //};

    //let mut compute_data = vec![64, 32, 8, 12, 96];
    vulkan.run_compute_simultaneous(
      &self.compute_shader,
      &push_constants,
      &vec![
        self.light_culling_descriptor_set.internal()[0],
        self.camera_descriptor_set.internal()[0],
        self.intermediate_descriptor_set.internal()[0],
      ],
      &mut self.light_visibility_buffer,
      &mut self.point_light_buffer,
      tile_count_per_row as u32,
      tile_count_per_col as u32,
      1,
      //&mut compute_data,
    );

    //println!("Compute Data: {:?}", compute_data);
  }
}
