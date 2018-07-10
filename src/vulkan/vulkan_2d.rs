use vulkano::memory;
use vulkano::sync::NowFuture;
use vulkano::device::{Queue, Device};
use vulkano::buffer::{cpu_pool, BufferUsage, 
                      BufferAccess, CpuBufferPool, 
                      ImmutableBuffer, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, CommandBufferExecFuture};

use math;
use graphics::Vertex2d;
use drawcalls::DrawCall;
use vulkan::rawvk::{Model,DynamicModel, vs_texture};

use cgmath::{ortho, Vector4, Matrix4};

use std::sync::Arc;

pub fn create_2d_projection(width: f32, height: f32) -> Matrix4<f32> {
  ortho(0.0, width, height, 0.0, -1.0, 1.0)
}

pub fn create_vertex(device: Arc<Device>) -> Arc<BufferAccess + Send + Sync> {
  let square = {
    [
      Vertex2d { position: [  0.5 ,   0.5 ], uv: [1.0, 0.0] },
      Vertex2d { position: [ -0.5,    0.5 ], uv: [0.0, 0.0] },
      Vertex2d { position: [ -0.5,   -0.5 ], uv: [0.0, 1.0] },
      Vertex2d { position: [  0.5 ,  -0.5 ], uv: [1.0, 1.0] },
    ]
  };
  
  CpuAccessibleBuffer::from_iter(device,
                                 BufferUsage::vertex_buffer(), 
                                 square.iter().cloned())
                                 .expect("failed to create vertex buffer")
}

pub fn create_index(queue: Arc<Queue>) -> (Arc<ImmutableBuffer<[u32]>>,
                                  CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
  let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
  ImmutableBuffer::from_iter(indices.iter().cloned(), 
                             BufferUsage::index_buffer(),
                             queue)
                             .expect("failed to create immutable index buffer")
}

pub fn create_dynamic_custom_model(device: Arc<Device>, mut verts: Vec<Vertex2d>, indices: Vec<u32>) -> DynamicModel {
  for i in 0..verts.len() {
    verts[i].position[0] *= -1.0;
    verts[i].position[1] *= -1.0;
  }
  
  let vert =  CpuAccessibleBuffer::from_iter(device.clone(),
                                 BufferUsage::vertex_buffer(), 
                                 verts.iter().cloned())
                                 .expect("Vulkan failed to create custom vertex buffer");
  let idx = CpuAccessibleBuffer::from_iter(device,
                                 BufferUsage::index_buffer(), 
                                 indices.iter().cloned())
                                 .expect("Vulkan failed to create custom index buffer");
  
  DynamicModel {
    vertex_buffer: Some(vec!(vert)),
    index_buffer: Some(idx),
  }
}

pub fn create_static_custom_model(device: Arc<Device>, queue: Arc<Queue>, mut verts: Vec<Vertex2d>, indices: Vec<u32>) -> (Model, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
  for i in 0..verts.len() {
    verts[i].position[1] *= -1.0;
  }
  
  let vert =  CpuAccessibleBuffer::from_iter(device,
                                 BufferUsage::vertex_buffer(), 
                                 verts.iter().cloned())
                                 .expect("Vulkan failed to create custom vertex buffer");
  
  let (idx_buffer, future_idx) = ImmutableBuffer::from_iter(indices.iter().cloned(), 
                             BufferUsage::index_buffer(), 
                             queue)
                             .expect("failed to create immutable index buffer");
  
  let model = Model {
    vertex_buffer: Some(vec!(vert)),
    index_buffer: Some(idx_buffer),
  };
  
  (model, future_idx)
}

pub fn create_texture_subbuffer(draw: DrawCall, projection: Matrix4<f32>, uniform_buffer_texture: CpuBufferPool<vs_texture::ty::Data>) -> cpu_pool::CpuBufferPoolSubbuffer<vs_texture::ty::Data, Arc<memory::pool::StdMemoryPool>> {
  let model = math::calculate_texture_model(draw.get_translation(), draw.get_size(), -draw.get_x_rotation() -180.0);
  
  let has_texture = {
    let mut value = 1.0;
    if draw.get_texture() == &String::from("") {
      value = 0.0;
    }
    value
  };
  
  let mut bw: f32 = 0.0;
  if draw.is_back_and_white() {
    bw = 1.0;
  }
  
  let uniform_data = vs_texture::ty::Data {
    projection: projection.into(),
    model: model.into(),
    colour: draw.get_colour().into(),
    has_texture_blackwhite: Vector4::new(has_texture, bw, 0.0, 0.0).into(),
  };
  
  uniform_buffer_texture.next(uniform_data).unwrap()
}
