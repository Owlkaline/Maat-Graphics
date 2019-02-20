use vk;

use crate::vulkan::vkenums::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout, ShaderStage,
                             VertexInputRate, SampleCount};

use crate::vulkan::{Instance, Device, RenderPass, Shader, Pipeline, PipelineBuilder, DescriptorSet, UpdateDescriptorSets, DescriptorSetBuilder, ImageAttachment, AttachmentInfo, SubpassInfo, RenderPassBuilder, Sampler, ClearValues};
use crate::vulkan::buffer::{Buffer, BufferUsage, UniformData, Framebuffer, CommandBufferBuilder};
use crate::vulkan::pool::{DescriptorPool, CommandPool};
use crate::CoreMaat;

use cgmath::{Vector2, Vector4};

use std::mem;
use std::sync::Arc;

#[derive(Clone)]
pub struct FinalVertex {
  pos: Vector2<f32>,
  uvs: Vector2<f32>,
}

impl FinalVertex {
  pub fn vertex_input_binding() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription {
      binding: 0,
      stride: (mem::size_of::<FinalVertex>()) as u32,
      inputRate: VertexInputRate::Vertex.to_bits(),
    }
  }
  
  pub fn vertex_input_attributes() -> Vec<vk::VertexInputAttributeDescription> {
    let mut vertex_input_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> = Vec::with_capacity(2);
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::FORMAT_R32G32B32_SFLOAT,
        offset: offset_of!(FinalVertex, pos) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::FORMAT_R32G32_SFLOAT,
        offset: offset_of!(FinalVertex, uvs) as u32,
      }
    );
    
    vertex_input_attribute_descriptions
  }
}

pub struct FinalShader {
  renderpass: RenderPass,
  framebuffers: Vec<Framebuffer>,
  descriptor_sets: Vec<DescriptorSet>,
  ds: Vec<DescriptorSet>,
  
  vertex_buffer: Buffer<FinalVertex>,
  index_buffer: Buffer<u32>,
  
  pipeline: Pipeline,
  
  vertex_shader: Shader,
  fragment_shader: Shader,
}

impl FinalShader {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, current_extent: &vk::Extent2D, format: &vk::Format, sampler: &Sampler, image_views: &Vec<vk::ImageView>, texture_image: &ImageAttachment, descriptor_set_pool: &DescriptorPool, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> FinalShader {
    let vertex_shader = Shader::new(Arc::clone(&device), include_bytes!("shaders/sprv/VkFinalVert.spv"));
    let fragment_shader = Shader::new(Arc::clone(&device), include_bytes!("shaders/sprv/VkFinalFrag.spv"));
    
    let colour_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(&SampleCount::OneBit)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::Store)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::PresentSrcKHR)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    let subpass = SubpassInfo::new().add_colour_attachment(0);
    let render_pass = RenderPassBuilder::new()
                      .add_attachment(colour_attachment)
                      .add_subpass(subpass)
                      .build(Arc::clone(&device));
    
    let framebuffers = FinalShader::create_frame_buffers(Arc::clone(&device), &render_pass, current_extent, image_views);
    
    let mut descriptor_sets = Vec::new();
    let mut ds = Vec::new();
    
    for i in 0..image_views.len() {
      descriptor_sets.push(DescriptorSetBuilder::new()
        .fragment_combined_image_sampler(0)
        .build(Arc::clone(&device), &descriptor_set_pool, 1));
      
      UpdateDescriptorSets::new()
       .add_sampled_image(0, texture_image, ImageLayout::ColourAttachmentOptimal, sampler)
       .finish_update(Arc::clone(&device), &descriptor_sets[i]);
       
      ds.push(DescriptorSetBuilder::new()
        .fragment_combined_image_sampler(0)
        .build(Arc::clone(&device), &descriptor_set_pool, 1));
      
      UpdateDescriptorSets::new()
       .add_sampled_image(0, texture_image, ImageLayout::ColourAttachmentOptimal, sampler)
       .finish_update(Arc::clone(&device), &ds[i]);
    }
    
    let pipeline = FinalShader::create_pipline(Arc::clone(&device), &vertex_shader, &fragment_shader, &render_pass, &descriptor_sets[0], &ds[0]);
    
    let vertex_buffer = FinalShader::create_vertex_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    let index_buffer = FinalShader::create_index_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    
    FinalShader {
      renderpass: render_pass,
      framebuffers,
      descriptor_sets,
      ds,
      
      vertex_buffer,
      index_buffer,
      
      pipeline,
      
      vertex_shader,
      fragment_shader,
    }
  }
  
  pub fn recreate(&mut self, device: Arc<Device>, image_views: &Vec<vk::ImageView>, new_extent: &vk::Extent2D, textures: Vec<(String, ImageAttachment)>, sampler: &Sampler) {
    for i in 0..self.framebuffers.len() {
      self.framebuffers[i].destroy(Arc::clone(&device));
    }
    
    self.framebuffers.clear();
    
    for i in 0..image_views.len() {
      self.framebuffers.push(Framebuffer::new(Arc::clone(&device), &self.renderpass, &new_extent, &image_views[i]));
      UpdateDescriptorSets::new()
         .add_sampled_image(0, &textures[0].1, ImageLayout::ColourAttachmentOptimal, sampler)
         .finish_update(Arc::clone(&device), &self.descriptor_sets[i]);
      UpdateDescriptorSets::new()
         .add_sampled_image(0, &textures[0].1, ImageLayout::ColourAttachmentOptimal, sampler)
         .finish_update(Arc::clone(&device), &self.ds[i]);
    }
  }
  
  fn create_pipline(device: Arc<Device>, vertex_shader: &Shader, fragment_shader: &Shader, render_pass: &RenderPass, descriptor_set: &DescriptorSet, ds: &DescriptorSet) -> Pipeline {
    let push_constant_size = UniformData::new()
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .size();
    
    let pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStage::Vertex, push_constant_size as u32)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .descriptor_set_layout(ds.layouts_clone())
                  .vertex_binding(vec!(FinalVertex::vertex_input_binding()))
                  .vertex_attributes(FinalVertex::vertex_input_attributes())
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  .cull_mode_back()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    pipeline
  }
  
  pub fn create_index_buffer(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<u32> {
    let indices = vec!(0, 1, 2, 2, 3, 0);
    
    let usage_src = BufferUsage::index_transfer_src_buffer();
    let usage_dst = BufferUsage::index_transfer_dst_buffer();
    
    let staging_buffer: Buffer<u32> = Buffer::cpu_buffer(Arc::clone(&instance), Arc::clone(&device), usage_src, 1, indices.clone());
    let buffer: Buffer<u32> = Buffer::device_local_buffer(Arc::clone(&instance), Arc::clone(&device), usage_dst, 1, indices);
    
    let command_buffer = CoreMaat::begin_single_time_command(Arc::clone(&device), &command_pool);
    command_buffer.copy_buffer(Arc::clone(&device), &staging_buffer, &buffer, 0);
    CoreMaat::end_single_time_command(Arc::clone(&device), command_buffer, &command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    buffer
  }
  
  pub fn create_vertex_buffer(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<FinalVertex> {
    let square = vec!(
      FinalVertex { pos: Vector2::new(0.5, 0.5), uvs: Vector2::new(0.99, 0.0) },
      FinalVertex { pos: Vector2::new(-0.5, 0.5), uvs: Vector2::new(0.0, 0.0) },
      FinalVertex { pos: Vector2::new(-0.5, -0.5), uvs: Vector2::new(0.0, 0.99) },
      FinalVertex { pos: Vector2::new(0.5, -0.5), uvs: Vector2::new(0.99, 0.99) },
    );
    
    let usage_src = BufferUsage::vertex_transfer_src_buffer();
    let usage_dst = BufferUsage::vertex_transfer_dst_buffer();
    
    let staging_buffer: Buffer<FinalVertex> = Buffer::cpu_buffer(Arc::clone(&instance), Arc::clone(&device), usage_src, 1, square.clone());
    let buffer: Buffer<FinalVertex> = Buffer::device_local_buffer(Arc::clone(&instance), Arc::clone(&device), usage_dst, 1, square);
    
    let command_buffer = CoreMaat::begin_single_time_command(Arc::clone(&device), &command_pool);
    command_buffer.copy_buffer(Arc::clone(&device), &staging_buffer, &buffer, 0);
    CoreMaat::end_single_time_command(Arc::clone(&device), command_buffer, &command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    buffer
  }
  
  fn create_frame_buffers(device: Arc<Device>, render_pass: &RenderPass, swapchain_extent: &vk::Extent2D, image_views: &Vec<vk::ImageView>) -> Vec<Framebuffer> {
    let mut framebuffers: Vec<Framebuffer> = Vec::with_capacity(image_views.len());
    
    for i in 0..image_views.len() {
      let framebuffer: Framebuffer = Framebuffer::new(Arc::clone(&device), render_pass, swapchain_extent, &image_views[i]);
      
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  pub fn begin_renderpass(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, clear_value: &Vec<vk::ClearValue>, window_size: &vk::Extent2D, current_buffer: usize) -> CommandBufferBuilder {
    cmd.begin_render_pass(Arc::clone(&device), clear_value, &self.renderpass, &self.framebuffers[current_buffer].internal_object(), &window_size)
  }
  
  pub fn draw_to_screen(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, texture_image: ImageAttachment, model_image: ImageAttachment, sampler: &Sampler, window_width: f32, window_height: f32, current_buffer: usize) -> CommandBufferBuilder {
    let mut cmd = cmd;
    
    UpdateDescriptorSets::new()
       .add_sampled_image(0, &texture_image, ImageLayout::ColourAttachmentOptimal, sampler)
       .finish_update(Arc::clone(&device), &self.descriptor_sets[current_buffer]);
    
    UpdateDescriptorSets::new()
       .add_sampled_image(0, &model_image, ImageLayout::ColourAttachmentOptimal, sampler)
       .finish_update(Arc::clone(&device), &self.ds[current_buffer]);
    
    let model = Vector4::new(window_width*0.5, window_height*0.5, window_width, window_height);
    
    let top = window_height;
    let right = window_width;
    let projection_details = Vector4::new(right, top, 0.0, 0.0);
    
    let push_constant_data = UniformData::new()
                               .add_vector4(model)
                               .add_vector4(projection_details);
    
    cmd = cmd.push_constants(Arc::clone(&device), &self.pipeline, ShaderStage::Vertex, push_constant_data);
    
    let index_count = 6;
    
    cmd.draw_indexed(Arc::clone(&device), &self.vertex_buffer.internal_object(0),
                             &self.index_buffer.internal_object(0),
                             index_count, &self.pipeline,
                             vec!(*self.descriptor_sets[current_buffer].set(0), *self.ds[current_buffer].set(0)))
  }
  
  pub fn destroy(&mut self, device: Arc<Device>) {
    self.index_buffer.destroy(Arc::clone(&device));
    self.vertex_buffer.destroy(Arc::clone(&device));
    
    self.pipeline.destroy(Arc::clone(&device));
    
    for descriptor_set in &self.descriptor_sets {
      descriptor_set.destroy(Arc::clone(&device));
    }
    
    for descriptor_set in &self.ds {
      descriptor_set.destroy(Arc::clone(&device));
    }
    
    self.vertex_shader.destroy(Arc::clone(&device));
    self.fragment_shader.destroy(Arc::clone(&device));
    
    for framebuffer in &self.framebuffers {
     framebuffer.destroy(Arc::clone(&device));
    }
    
    self.renderpass.destroy(Arc::clone(&device));
  }
}
