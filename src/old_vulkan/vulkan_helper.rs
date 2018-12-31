use vulkano::descriptor;

use vulkano::buffer::BufferAccess;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipelineAbstract;

use std::sync::Arc;

pub fn draw_dynamic(cb: AutoCommandBufferBuilder, 
                    dimensions: [u32; 2], 
                    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>, 
                    dynamic_state: &DynamicState,
                    vertex_buffer: Vec<Arc<BufferAccess + Send + Sync>>, 
                    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>, 
                    uniform_buffer: Arc<descriptor::DescriptorSet + Send + Sync>) -> AutoCommandBufferBuilder {
  cb.draw_indexed(pipeline,
                  dynamic_state,
                  vertex_buffer,
                  index_buffer,
                  uniform_buffer, ()).unwrap()
}

pub fn draw_immutable(cb: AutoCommandBufferBuilder, 
                      dimensions: [u32; 2], 
                      pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>, 
                      dynamic_state: &DynamicState,
                      vertex_buffer: Vec<Arc<BufferAccess + Send + Sync>>, 
                      index_buffer: Arc<ImmutableBuffer<[u32]>>, 
                      uniform_buffer: Arc<descriptor::DescriptorSet + Send + Sync>) -> AutoCommandBufferBuilder {
  cb.draw_indexed(pipeline,
                  dynamic_state,
                  vertex_buffer,
                  index_buffer,
                  uniform_buffer, ()).unwrap()
}
