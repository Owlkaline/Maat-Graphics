
use crate::TextureShader;
use crate::Vertex;

use crate::vulkan::vkenums::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout, ShaderStageFlagBits};

use crate::vulkan::{Instance, Device, RenderPass, Shader, Pipeline, PipelineBuilder, DescriptorSet, DescriptorSetBuilder, UpdateDescriptorSets, Image, AttachmentInfo, SubpassInfo, RenderPassBuilder, Sampler};
use crate::vulkan::buffer::{Buffer, BufferUsage, UniformBufferBuilder, UniformData, Framebuffer, CommandBufferBuilder};
use crate::vulkan::pool::{DescriptorPool, CommandPool};

use cgmath::{Vector2, Vector3, Vector4, Matrix4, ortho, SquareMatrix};

use std::mem;
use std::ptr;
use std::sync::Arc;

pub struct FinalShader {
  vertex_shader: Shader,
  fragment_shader: Shader,
  
  renderpass: RenderPass,
  framebuffers: Vec<Framebuffer>,
  descriptor_set: DescriptorSet,
  
  vertex_buffer: Buffer<Vertex>,
  index_buffer: Buffer<u32>,
  uniform_buffer: Buffer<f32>,
  
  pipeline: Pipeline,
}

impl FinalShader {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, current_extent: &vk::Extent2D, format: &vk::Format, sampler: &Sampler, image_views: &Vec<vk::ImageView>, texture_image: &Image, descriptor_set_pool: &DescriptorPool, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> FinalShader {
    let vertex_shader = Shader::new(Arc::clone(&device), include_bytes!("./shaders/sprv/VkFinalVert.spv"));
    let fragment_shader = Shader::new(Arc::clone(&device), include_bytes!("./shaders/sprv/VkFinalFrag.spv"));
    
    let colour_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(0)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::Store)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::PresentSrcKHR)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let subpass = SubpassInfo::new().add_input_attachment(0).add_colour_attachment(0);
    let renderpass = RenderPassBuilder::new()
                      .add_attachment(colour_attachment)
                      .add_subpass(subpass)
                      .build(Arc::clone(&device));
    
    let framebuffers = FinalShader::create_frame_buffers(Arc::clone(&device), &renderpass, current_extent, image_views);
    
    let mut descriptor_set: DescriptorSet = DescriptorSetBuilder::new()
                                             .vertex_uniform_buffer(0, 0)
                                             .fragment_combined_image_sampler(0, 1)
                                             .build(Arc::clone(&device), &descriptor_set_pool, 1);
    
    let push_constant_size = UniformData::new()
                               .add_matrix4(Matrix4::identity())
                               .size();
    
    let pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStageFlagBits::Vertex, push_constant_size as u32)
                  .render_pass(renderpass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .vertex_binding(vec!(Vertex::vertex_input_binding()))
                  .vertex_attributes(Vertex::vertex_input_attributes())
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  .cull_mode_back()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    let vertex_buffer = TextureShader::create_vertex_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    let index_buffer = TextureShader::create_index_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    
    let mut uniform_buffer: Buffer<f32>; 
    let mut uniform_buffer_description = UniformBufferBuilder::new().add_matrix4();
    
    uniform_buffer = FinalShader::create_uniform_buffer(Arc::clone(&instance), Arc::clone(&device), uniform_buffer_description);
    
    FinalShader {
      vertex_shader,
      fragment_shader,
      
      renderpass,
      framebuffers,
      descriptor_set,
      
      vertex_buffer,
      index_buffer,
      uniform_buffer,
      
      pipeline,
    }
  }
  
  fn create_frame_buffers(device: Arc<Device>, render_pass: &RenderPass, swapchain_extent: &vk::Extent2D, image_views: &Vec<vk::ImageView>) -> Vec<Framebuffer> {
    let mut framebuffers: Vec<Framebuffer> = Vec::with_capacity(image_views.len());
    
    for i in 0..image_views.len() {
      let framebuffer: Framebuffer = Framebuffer::new(Arc::clone(&device), render_pass, swapchain_extent, &image_views[i]);
      
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  fn create_uniform_buffer(instance: Arc<Instance>, device: Arc<Device>, uniform_buffer: UniformBufferBuilder) -> Buffer<f32> {
    let mut buffer = uniform_buffer.build(Arc::clone(&instance), Arc::clone(&device), 1);
    
    buffer
  }
  
}
