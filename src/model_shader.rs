use vk;

use crate::math;
use crate::drawcalls;
use crate::font::GenericFont; 
use crate::Camera;
use crate::camera;

use crate::vulkan::vkenums::{ImageType, ImageUsage, ImageViewType, Sample, ImageTiling, AttachmentLoadOp, AttachmentStoreOp, ImageLayout, ShaderStageFlagBits, VertexInputRate};

use crate::vulkan::{Instance, Device, RenderPass, Shader, Pipeline, PipelineBuilder, DescriptorSet, UpdateDescriptorSets, DescriptorSetBuilder, Image, ImageAttachment, AttachmentInfo, SubpassInfo, RenderPassBuilder, Sampler};
use crate::vulkan::buffer::{Buffer, BufferUsage, UniformBufferBuilder, UniformData, Framebuffer, CommandBufferBuilder};
use crate::vulkan::pool::{DescriptorPool, CommandPool};
use crate::CoreMaat;

use cgmath::{Vector2, Vector3, Vector4, Matrix4, SquareMatrix};

use std::mem;
use std::sync::Arc;
use std::collections::HashMap;

const MAX_INSTANCES: usize = 8096;

#[derive(Clone)]
pub struct ModelVertex {
  pos: Vector3<f32>,
  normal: Vector3<f32>,
  uvs: Vector2<f32>,
  colour: Vector4<f32>,
  tangent: Vector4<f32>,
}

impl ModelVertex {
  pub fn vertex_input_binding() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription {
      binding: 0,
      stride: (mem::size_of::<ModelVertex>()) as u32,
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
        offset: offset_of!(ModelVertex, pos) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::FORMAT_R32G32B32_SFLOAT,
        offset: offset_of!(ModelVertex, normal) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 2,
        binding: 0,
        format: vk::FORMAT_R32G32_SFLOAT,
        offset: offset_of!(ModelVertex, uvs) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 3,
        binding: 0,
        format: vk::FORMAT_R32G32B32A32_SFLOAT,
        offset: offset_of!(ModelVertex, colour) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 4,
        binding: 0,
        format: vk::FORMAT_R32G32B32A32_SFLOAT,
        offset: offset_of!(ModelVertex, tangent) as u32,
      }
    );
    
    vertex_input_attribute_descriptions
  }
}

pub struct ModelShader {
  renderpass: RenderPass,
  framebuffers: Vec<Framebuffer>,
  framebuffer_images: Vec<ImageAttachment>,
  descriptor_sets: Vec<DescriptorSet>,
  
  vertex_buffer: Buffer<ModelVertex>,
  index_buffer: Buffer<u32>,
  
  pipeline: Pipeline,
  
  vertex_shader: Shader,
  fragment_shader: Shader,
  
  scale: f32,
  camera: Camera,
}

impl ModelShader {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, current_extent: &vk::Extent2D, format: &vk::Format, sampler: &Sampler, image_views: &Vec<vk::ImageView>, texture_image: &Image, descriptor_set_pool: &DescriptorPool, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> ModelShader {
    let vertex_shader = Shader::new(Arc::clone(&device), include_bytes!("shaders/sprv/VkModelVert.spv"));
    let fragment_shader = Shader::new(Arc::clone(&device), include_bytes!("shaders/sprv/VkModelFrag.spv"));
    
    let colour_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(0)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::Store)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let depth_attachment = AttachmentInfo::new()
                                .format(vk::FORMAT_D16_UNORM)
                                .multisample(0)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::Store)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::DepthStencilReadOnlyOptimal)
                                .image_usage(ImageLayout::DepthStencilAttachmentOptimal);
    
    let subpass = SubpassInfo::new().add_colour_attachment(0);//.add_depth_stencil(1);
    let render_pass = RenderPassBuilder::new()
                      .add_attachment(colour_attachment)
                      //.add_attachment(depth_attachment)
                      .add_subpass(subpass)
                      .build(Arc::clone(&device));
    
    let mut framebuffer_images = Vec::with_capacity(image_views.len());
    for _ in 0..image_views.len() {
      framebuffer_images.push(ImageAttachment::create_image_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, ImageUsage::colour_attachment_storage_sampled(), &format, &vk::Extent3D { width: current_extent.width, height: current_extent.height, depth: 1 }, &Sample::Count1Bit, ImageLayout::Undefined, &ImageTiling::Optimal, &ImageViewType::Type2D));
    }
    
    let framebuffers = ModelShader::create_frame_buffers(Arc::clone(&device), &render_pass, current_extent, &framebuffer_images);
    
    let mut descriptor_sets = Vec::new();
    for i in 0..image_views.len() {
      descriptor_sets.push(DescriptorSetBuilder::new()
        .build(Arc::clone(&device), &descriptor_set_pool, 1));
      
      UpdateDescriptorSets::new()
       .finish_update(Arc::clone(&device), &descriptor_sets[i]);
    }
    
    let pipeline = ModelShader::create_pipline(Arc::clone(&device), &vertex_shader, &fragment_shader, &render_pass, &descriptor_sets[0]);
    
    let vertex_buffer = ModelShader::create_vertex_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    let index_buffer = ModelShader::create_index_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    
    let mut camera = Camera::default_vk();
    
    ModelShader {
      renderpass: render_pass,
      framebuffers,
      framebuffer_images,
      descriptor_sets,
      
      vertex_buffer,
      index_buffer,
      
      pipeline,
      
      vertex_shader,
      fragment_shader,
      
      scale: 1.0,
      camera,
    }
  }
  
  pub fn set_scale(&mut self, new_scale: f32) {
    self.scale = new_scale;
  }
  
  pub fn get_texture(&mut self, current_buffer: usize) -> Image {
    self.framebuffer_images[current_buffer].to_image()
  }
  
  pub fn recreate(&mut self, instance: Arc<Instance>, device: Arc<Device>, format: &vk::Format, image_views: &Vec<vk::ImageView>, new_extent: &vk::Extent2D, textures: Vec<(String, Image)>, sampler: &Sampler) {
    for i in 0..self.framebuffers.len() {
      self.framebuffers[i].destroy(Arc::clone(&device));
      self.framebuffer_images[i].destroy(Arc::clone(&device));
    }
    
    self.framebuffers.clear();
    self.framebuffer_images.clear();
    
    for _ in 0..image_views.len() {
      self.framebuffer_images.push(ImageAttachment::create_image_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, ImageUsage::colour_attachment_sampled(), format, &vk::Extent3D { width: new_extent.width, height: new_extent.height, depth: 1 }, &Sample::Count1Bit, ImageLayout::Undefined, &ImageTiling::Optimal, &ImageViewType::Type2D));
    }
    
    for i in 0..image_views.len() {
      self.framebuffers.push(Framebuffer::new(Arc::clone(&device), &self.renderpass, &new_extent, &self.framebuffer_images[i].get_image_view()));
      UpdateDescriptorSets::new()
         .finish_update(Arc::clone(&device), &self.descriptor_sets[i]);
    }
  }
  
  fn create_pipline(device: Arc<Device>, vertex_shader: &Shader, fragment_shader: &Shader, render_pass: &RenderPass, descriptor_set: &DescriptorSet) -> Pipeline {
    let push_constant_size = UniformData::new()
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .size();
    
    let pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStageFlagBits::Vertex, push_constant_size as u32)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .vertex_binding(vec!(ModelVertex::vertex_input_binding()))
                  .vertex_attributes(ModelVertex::vertex_input_attributes())
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  .enable_depth_clamp()
                  .enable_depth_write()
                  .cull_mode_back()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    pipeline
  }
  
  pub fn create_index_buffer(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<u32> {
    let indices = vec!(0, 1, 2, 2, 3, 0, // back side
                       7, 3, 4, 4, 3, 0, // right side
                       4, 5, 7, 6, 7, 5, // front side
                       2, 5, 1, 2, 6, 5, // left side
                       5, 4, 0, 0, 1, 5, // top side
                       2, 7, 6, 7, 2, 3  // bottom side
                  );
    
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
  
  pub fn create_vertex_buffer(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<ModelVertex> {
    let cube = vec!(
      ModelVertex { pos: Vector3::new(0.5, 0.5, 0.5), normal: Vector3::new(0.0, 0.0, 0.0), 
                    uvs: Vector2::new(0.99, 0.99), colour: Vector4::new(1.0, 0.0, 0.0, 1.0), 
                    tangent: Vector4::new(0.0, 0.0, 0.0, 0.0) },
      ModelVertex { pos: Vector3::new(-0.5, 0.5, 0.5), normal: Vector3::new(0.0, 0.0, 0.0), 
                    uvs: Vector2::new(0.0, 0.99), colour: Vector4::new(1.0, 0.0, 0.0, 1.0), 
                    tangent: Vector4::new(0.0, 0.0, 0.0, 0.0) },
      ModelVertex { pos: Vector3::new(-0.5, -0.5, 0.5), normal: Vector3::new(0.0, 0.0, 0.0), 
                    uvs: Vector2::new(0.0, 0.0), colour: Vector4::new(1.0, 0.0, 0.0, 1.0), 
                    tangent: Vector4::new(0.0, 0.0, 0.0, 0.0) },
      ModelVertex { pos: Vector3::new(0.5, -0.5, 0.5), normal: Vector3::new(0.0, 0.0, 0.0), 
                    uvs: Vector2::new(0.99, 0.0), colour: Vector4::new(1.0, 0.0, 0.0, 1.0), 
                    tangent: Vector4::new(0.0, 0.0, 0.0, 0.0) },
      ModelVertex { pos: Vector3::new(0.5, 0.5, -0.5), normal: Vector3::new(0.0, 0.0, 0.0), 
                    uvs: Vector2::new(0.99, 0.99), colour: Vector4::new(1.0, 0.0, 0.0, 1.0), 
                    tangent: Vector4::new(0.0, 0.0, 0.0, 0.0) },
      ModelVertex { pos: Vector3::new(-0.5, 0.5, -0.5), normal: Vector3::new(0.0, 0.0, 0.0), 
                    uvs: Vector2::new(0.0, 0.99), colour: Vector4::new(1.0, 0.0, 0.0, 1.0), 
                    tangent: Vector4::new(0.0, 0.0, 0.0, 0.0) },
      ModelVertex { pos: Vector3::new(-0.5, -0.5, -0.5), normal: Vector3::new(0.0, 0.0, 0.0), 
                    uvs: Vector2::new(0.0, 0.0), colour: Vector4::new(1.0, 0.0, 0.0, 1.0), 
                    tangent: Vector4::new(0.0, 0.0, 0.0, 0.0) },
      ModelVertex { pos: Vector3::new(0.5, -0.5, -0.5), normal: Vector3::new(0.0, 0.0, 0.0), 
                    uvs: Vector2::new(0.99, 0.0), colour: Vector4::new(1.0, 0.0, 0.0, 1.0), 
                    tangent: Vector4::new(0.0, 0.0, 0.0, 0.0) },    );
    
    let usage_src = BufferUsage::vertex_transfer_src_buffer();
    let usage_dst = BufferUsage::vertex_transfer_dst_buffer();
    
    let staging_buffer: Buffer<ModelVertex> = Buffer::cpu_buffer(Arc::clone(&instance), Arc::clone(&device), usage_src, 1, cube.clone());
    let buffer: Buffer<ModelVertex> = Buffer::device_local_buffer(Arc::clone(&instance), Arc::clone(&device), usage_dst, 1, cube);
    
    let command_buffer = CoreMaat::begin_single_time_command(Arc::clone(&device), &command_pool);
    command_buffer.copy_buffer(Arc::clone(&device), &staging_buffer, &buffer, 0);
    CoreMaat::end_single_time_command(Arc::clone(&device), command_buffer, &command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    buffer
  }
  
  fn create_frame_buffers(device: Arc<Device>, render_pass: &RenderPass, swapchain_extent: &vk::Extent2D, framebuffer_images: &Vec<ImageAttachment>) -> Vec<Framebuffer> {
    let mut framebuffers: Vec<Framebuffer> = Vec::with_capacity(framebuffer_images.len());
    
    for i in 0..framebuffer_images.len() {
      let framebuffer: Framebuffer = Framebuffer::new(Arc::clone(&device), render_pass, swapchain_extent, &framebuffer_images[i].get_image_view());
      
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  pub fn begin_renderpass(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, clear_value: &Vec<vk::ClearValue>, window_size: &vk::Extent2D, current_buffer: usize) -> CommandBufferBuilder {
    cmd.begin_render_pass(Arc::clone(&device), &clear_value, &self.renderpass, &self.framebuffers[current_buffer].internal_object(), &window_size)
  }
  
  pub fn draw_model(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>, model_reference: String, window_width: f32, window_height: f32, current_buffer: usize) -> CommandBufferBuilder {
    let mut cmd = cmd;
    
    let fov = 90.0;
    let aspect = window_width / window_height;
    let (c_pos, c_center, c_up) = self.camera.get_look_at();
    
    let camera_position = Vector4::new(c_pos.x,    c_pos.y,    c_pos.z,    fov);
    let camera_center   = Vector4::new(c_center.x, c_center.y, c_center.z, aspect);
    let camera_up       = Vector4::new(c_up.x,     c_up.y,     c_up.z,     scale.x);
    let model           = Vector4::new(position.x, position.y, position.z, scale.y);
    let rotation        = Vector4::new(rotation.x, rotation.y, rotation.x, scale.z);
    
    let push_constant_data = UniformData::new()
                               .add_vector4(camera_position)
                               .add_vector4(camera_center)
                               .add_vector4(camera_up)
                               .add_vector4(model)
                               .add_vector4(rotation);
    
    cmd = cmd.push_constants(Arc::clone(&device), &self.pipeline, ShaderStageFlagBits::Vertex, push_constant_data);
    
    let index_count = 36;
    
    cmd.draw_indexed(Arc::clone(&device), &self.vertex_buffer.internal_object(0),
                             &self.index_buffer.internal_object(0),
                             index_count, &self.pipeline,
                             vec!(*self.descriptor_sets[current_buffer].set(0)))
  }
  
  pub fn destroy(&mut self, device: Arc<Device>) {
    self.index_buffer.destroy(Arc::clone(&device));
    self.vertex_buffer.destroy(Arc::clone(&device));
    
    self.pipeline.destroy(Arc::clone(&device));
    for descriptor in &self.descriptor_sets {
      descriptor.destroy(Arc::clone(&device));
    }
    
    self.vertex_shader.destroy(Arc::clone(&device));
    self.fragment_shader.destroy(Arc::clone(&device));
    
    for framebuffer in &self.framebuffers {
     framebuffer.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_images {
      images.destroy(Arc::clone(&device));
    }
    
    self.renderpass.destroy(Arc::clone(&device));
  }
}
