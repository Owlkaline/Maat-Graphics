use vulkano::device::Queue;
use vulkano::device::Device;

use vulkano::buffer::BufferAccess;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::sync::NowFuture;
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::command_buffer::AutoCommandBuffer;

use graphics::Vertex2d;
use rawvk::{Model,DynamicModel};

use std::sync::Arc;

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
