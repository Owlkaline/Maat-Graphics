use vulkano::descriptor;

use vulkano::buffer::BufferAccess;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::framebuffer::RenderPassAbstract;

use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipelineAbstract;

use std::sync::Arc;

pub fn draw_dynamic(cb: AutoCommandBufferBuilder, 
                    dimensions: [u32; 2], 
                    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>, 
                    vertex_buffer: Vec<Arc<BufferAccess + Send + Sync>>, 
                    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>, 
                    uniform_buffer: Arc<descriptor::DescriptorSet + Send + Sync>) -> AutoCommandBufferBuilder {
  cb.draw_indexed(pipeline,
                  DynamicState {
                    line_width: None,
                    viewports: Some(vec![Viewport {
                      origin: [0.0, 0.0],
                      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                      depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                  },
                  vertex_buffer,
                  index_buffer,
                  uniform_buffer, ()).unwrap()
}

pub fn draw_immutable(cb: AutoCommandBufferBuilder, 
                      dimensions: [u32; 2], 
                      pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>, 
                      vertex_buffer: Vec<Arc<BufferAccess + Send + Sync>>, 
                      index_buffer: Arc<ImmutableBuffer<[u32]>>, 
                      uniform_buffer: Arc<descriptor::DescriptorSet + Send + Sync>) -> AutoCommandBufferBuilder {
  cb.draw_indexed(pipeline,
                  DynamicState {
                    line_width: None,
                    viewports: Some(vec![Viewport {
                      origin: [0.0, 0.0],
                      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                      depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                  },
                  vertex_buffer,
                  index_buffer,
                  uniform_buffer, ()).unwrap()
}
