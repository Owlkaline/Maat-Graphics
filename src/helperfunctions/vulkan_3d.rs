use vulkano::device::Device;
use vulkano::device::Queue;

use vulkano::sync::NowFuture;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferAccess;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::CommandBufferExecFuture;

use graphics::Vertex3d;

use std::sync::Arc;
use std::iter;
use std::slice;


pub fn create_vertex(device: Arc<Device>, verticies: iter::Cloned<slice::Iter<Vertex3d>>) -> Arc<BufferAccess + Send + Sync> {
    CpuAccessibleBuffer::from_iter(device, 
                                   BufferUsage::vertex_buffer(), 
                                   verticies)
                                   .expect("failed to create vertex buffer")
}

pub fn create_index(queue: Arc<Queue>, indices: iter::Cloned<slice::Iter<u32>>) -> (Arc<ImmutableBuffer<[u32]>>,
                                                                         CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    ImmutableBuffer::from_iter(indices, BufferUsage::index_buffer(), 
                               queue)
                               .expect("failed to create immutable teapot index buffer")
}
