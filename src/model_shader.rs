use vk;

use gltf::material::AlphaMode;

use crate::math;
use crate::drawcalls;
use crate::font::GenericFont; 
use crate::camera;
use crate::gltf_interpreter::ModelDetails;

use crate::vulkan::vkenums::{ImageType, ImageUsage, ImageViewType, Sample, ImageTiling, AttachmentLoadOp, AttachmentStoreOp, ImageLayout, ShaderStageFlagBits, VertexInputRate, AddressMode, MipmapMode, VkBool};

use crate::vulkan::{Instance, Device, RenderPass, Shader, Pipeline, PipelineBuilder, DescriptorSet, UpdateDescriptorSets, DescriptorSetBuilder, Image, ImageAttachment, AttachmentInfo, SubpassInfo, RenderPassBuilder, Sampler, SamplerBuilder};
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
  pub fn from(pos: Vector3<f32>, normal: Vector3<f32>, uvs: Vector2<f32>, colour: Vector4<f32>, tangent: Vector4<f32>) -> ModelVertex {
    ModelVertex {
      pos,
      normal,
      uvs,
      colour,
      tangent,
    }
  }
  
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

struct Model {
  vertex_buffers: Vec<Buffer<ModelVertex>>,
  index_buffers: Vec<Buffer<u32>>,
  vertex_count: Vec<u32>,
  index_count: Vec<u32>,
  
  descriptor_sets: Vec<DescriptorSet>,
  samplers: Vec<Sampler>,
  reference: String,
  
  base_colour_factors: Vec<Vector4<f32>>,
  alpha_cutoffs: Vec<(f32, f32)>,
  double_sided: Vec<bool>,
}

impl Model {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, reference: String, model: ModelDetails, base_textures: Vec<Option<Image>>, dummy_texture: &Image, command_pool: &CommandPool, descriptor_set_pool: &DescriptorPool, sampler: &Sampler, graphics_queue: &vk::Queue) -> Model {
    let num_models = model.num_models();
    
    let mut vertex_buffers = Vec::with_capacity(num_models);
    let mut index_buffers = Vec::with_capacity(num_models);
    let mut vertex_count = Vec::with_capacity(num_models);
    let mut index_count = Vec::with_capacity(num_models);
    let mut descriptor_sets = Vec::with_capacity(num_models);
    let mut samplers: Vec<Sampler> = Vec::with_capacity(num_models);
    
    let mut base_colour_factors = Vec::with_capacity(num_models);
    let mut alpha_cutoffs = Vec::with_capacity(num_models);
    let mut double_sided = Vec::with_capacity(num_models);
    
    for i in 0..num_models {
      let position = model.vertex(i); //vec3
      let normal = model.normal(i); //vec3
      let uv = model.texcoords(i); // vec2
      let colour = model.colours(i); // vec4 
      let tangent = model.tangent(i);//vec4
      
      let mut vertex = Vec::with_capacity(position.len());
      for j in 0..position.len() {
        let mut uvs = [0.0, 0.0];
        let mut model_tangent = [0.0, 0.0, 0.0, 0.0];
        let mut model_colour = [1.0, 1.0, 1.0, 1.0];
        if j < colour.len() {
          model_colour = colour[j];
        }
        if j < uv.len() {
          uvs = uv[j];
        }
        if j < tangent.len() {
          model_tangent = tangent[j];
        }
        
        let mut pos = math::array3_to_vec3(position[j]);
        pos.x *= -1.0;
        vertex.push(ModelVertex::from(pos, 
                                      math::array3_to_vec3(normal[j]), 
                                      math::array2_to_vec2(uvs), 
                                      math::array4_to_vec4(model_colour), 
                                      math::array4_to_vec4(model_tangent)));
      }
      
      let index = model.index(i);  // Vec<u32>
      let vertice = vertex.len() as u32;
      
      let v_buffer = Model::create_vertex_buffer(Arc::clone(&instance), Arc::clone(&device), vertex, command_pool, graphics_queue);
      let (i_buffer, indice) = Model::create_index_buffer(Arc::clone(&instance), Arc::clone(&device), index, command_pool, graphics_queue);
      
      let mut sampler = sampler;
      
      let temp_sampler;
      if let Some(sampler_info) = model.base_colour_sampler(i) {
        temp_sampler = SamplerBuilder::new()
                       .min_filter(sampler_info.min_filter)
                       .mag_filter(sampler_info.mag_filter)
                       .address_mode_u(sampler_info.s_wrap)
                       .address_mode_v(sampler_info.t_wrap)
                       .address_mode_w(AddressMode::ClampToEdge)
                       .mipmap_mode(MipmapMode::Nearest)
                       .anisotropy(VkBool::True)
                       .max_anisotropy(8.0)
                       .build(Arc::clone(&device));
        samplers.push(temp_sampler.clone());
        sampler = &temp_sampler;
      }
      
      let descriptor_set;
      descriptor_set = DescriptorSetBuilder::new()
                           .fragment_combined_image_sampler(0)
                           .build(Arc::clone(&device), &descriptor_set_pool, 1);
      if let Some(ref texture) = &base_textures[i] {
        UpdateDescriptorSets::new()
             .add_sampled_image(0, &texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .finish_update(Arc::clone(&device), &descriptor_set);
      } else {
        UpdateDescriptorSets::new()
             .add_sampled_image(0, &dummy_texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .finish_update(Arc::clone(&device), &descriptor_set);
      }
      
      let base_colour_factor = model.base_colour(i);
      let alpha_cutoff = model.alphacutoff(i);
      let alpha_mask = {
        match  model.alphamode(i) {
          AlphaMode::Opaque => {
            1.0
          },
          AlphaMode::Mask => {
            2.0
          },
          AlphaMode::Blend => {
            0.0
          }
        }
      };
      
      let double_side = model.double_sided(i);
      
      vertex_buffers.push(v_buffer);
      vertex_count.push(vertice);
      index_buffers.push(i_buffer);
      index_count.push(indice);
      descriptor_sets.push(descriptor_set);
      
      base_colour_factors.push(math::array4_to_vec4(base_colour_factor));
      alpha_cutoffs.push((alpha_cutoff, alpha_mask));
      double_sided.push(double_side);
    }
    
    Model {
      vertex_buffers,
      index_buffers,
      vertex_count,
      index_count,
      
      descriptor_sets,
      samplers,
      reference: reference.to_string(),
      
      base_colour_factors,
      alpha_cutoffs,
      double_sided,
    }
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    for vertex in &self.vertex_buffers {
      vertex.destroy(Arc::clone(&device));
    }
    
    for index in &self.index_buffers {
      index.destroy(Arc::clone(&device));
    }
    
    for descriptor in &self.descriptor_sets {
      descriptor.destroy(Arc::clone(&device));
    }
    
    for sampler in &self.samplers {
      println!("destroy samplers boiii");
      sampler.destroy(Arc::clone(&device));
    }
  }
  
  fn create_index_buffer(instance: Arc<Instance>, device: Arc<Device>, indexs: Vec<u32>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> (Buffer<u32>, u32) {
    let usage_src = BufferUsage::index_transfer_src_buffer();
    let usage_dst = BufferUsage::index_transfer_dst_buffer();
    
    let num_index = indexs.len() as u32;
    
    let staging_buffer: Buffer<u32> = Buffer::cpu_buffer(Arc::clone(&instance), Arc::clone(&device), usage_src, 1, indexs.clone());
    let buffer: Buffer<u32> = Buffer::device_local_buffer(Arc::clone(&instance), Arc::clone(&device), usage_dst, 1, indexs);
    
    let command_buffer = CoreMaat::begin_single_time_command(Arc::clone(&device), &command_pool);
    command_buffer.copy_buffer(Arc::clone(&device), &staging_buffer, &buffer, 0);
    CoreMaat::end_single_time_command(Arc::clone(&device), command_buffer, &command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    (buffer, num_index)
  }
  
  fn create_vertex_buffer(instance: Arc<Instance>, device: Arc<Device>, vertexs: Vec<ModelVertex>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<ModelVertex> {
    
    let usage_src = BufferUsage::vertex_transfer_src_buffer();
    let usage_dst = BufferUsage::vertex_transfer_dst_buffer();
    
    let staging_buffer: Buffer<ModelVertex> = Buffer::cpu_buffer(Arc::clone(&instance), Arc::clone(&device), usage_src, 1, vertexs.clone());
    let buffer: Buffer<ModelVertex> = Buffer::device_local_buffer(Arc::clone(&instance), Arc::clone(&device), usage_dst, 1, vertexs);
    
    let command_buffer = CoreMaat::begin_single_time_command(Arc::clone(&device), &command_pool);
    command_buffer.copy_buffer(Arc::clone(&device), &staging_buffer, &buffer, 0);
    CoreMaat::end_single_time_command(Arc::clone(&device), command_buffer, &command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    buffer
  }
}

pub struct ModelShader {
  renderpass: RenderPass,
  framebuffers: Vec<Framebuffer>,
  framebuffer_images: Vec<ImageAttachment>,
  framebuffer_depth: Vec<ImageAttachment>,
  descriptor_sets: Vec<DescriptorSet>,
  
  models: Vec<Model>,
  
  vertex_buffer: Buffer<ModelVertex>,
  index_buffer: Buffer<u32>,
  
  pipeline: Pipeline,
  double_pipeline: Pipeline,
  
  vertex_shader: Shader,
  fragment_shader: Shader,
  
  scale: f32,
  camera: camera::Camera,
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
                                .format(vk::FORMAT_D32_SFLOAT)
                                .multisample(0)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::DepthStencilAttachmentOptimal)
                                .image_usage(ImageLayout::DepthStencilAttachmentOptimal);
    
    let subpass = SubpassInfo::new().add_colour_attachment(0).add_depth_stencil(1);
    let render_pass = RenderPassBuilder::new()
                      .add_attachment(colour_attachment)
                      .add_attachment(depth_attachment)
                      .add_subpass(subpass)
                      .build(Arc::clone(&device));
    
    let mut framebuffer_images = Vec::with_capacity(image_views.len());
    let mut framebuffer_depth = Vec::with_capacity(image_views.len());
    for _ in 0..image_views.len() {
      framebuffer_images.push(ImageAttachment::create_image_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, ImageUsage::colour_attachment_storage_sampled(), &format, &vk::Extent3D { width: current_extent.width, height: current_extent.height, depth: 1 }, &Sample::Count1Bit, ImageLayout::Undefined, &ImageTiling::Optimal, &ImageViewType::Type2D));
      
      framebuffer_depth.push(ImageAttachment::create_depth_image_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, ImageUsage::depth_stencil_attachment(), &vk::FORMAT_D32_SFLOAT, &vk::Extent3D { width: current_extent.width, height: current_extent.height, depth: 1 }, &Sample::Count1Bit, ImageLayout::Undefined, &ImageTiling::Optimal, &ImageViewType::Type2D));
    }
    
    let framebuffers = ModelShader::create_frame_buffers(Arc::clone(&device), &render_pass, current_extent, &framebuffer_images, &framebuffer_depth);
    
    let mut descriptor_sets = Vec::new();
    for i in 0..image_views.len() {
      descriptor_sets.push(DescriptorSetBuilder::new()
        .fragment_combined_image_sampler(0)
        .build(Arc::clone(&device), &descriptor_set_pool, 1));
      
      UpdateDescriptorSets::new()
        .add_sampled_image(0, &texture_image, ImageLayout::ShaderReadOnlyOptimal, &sampler)
       .finish_update(Arc::clone(&device), &descriptor_sets[i]);
    }
    
    let (pipeline, double_pipeline) = ModelShader::create_pipline(Arc::clone(&device), &vertex_shader, &fragment_shader, &render_pass, &descriptor_sets[0]);
    
    let vertex_buffer = ModelShader::create_vertex_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    let index_buffer = ModelShader::create_index_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    
    let mut camera = camera::Camera::default_vk();
    
    ModelShader {
      renderpass: render_pass,
      framebuffers,
      framebuffer_images,
      framebuffer_depth,
      descriptor_sets,
      
      models: Vec::new(),
      
      vertex_buffer,
      index_buffer,
      
      pipeline,
      double_pipeline,
      
      vertex_shader,
      fragment_shader,
      
      scale: 1.0,
      camera,
    }
  }
  
  pub fn set_scale(&mut self, new_scale: f32) {
    self.scale = new_scale;
  }
  
  pub fn set_camera(&mut self, camera: camera::Camera) {
    self.camera = camera;
  }
  
  pub fn set_camera_move_speed(&mut self, speed: f32) {
    self.camera.set_move_speed(speed);
  }
  
  pub fn set_mouse_sensitivity(&mut self, sensitivity: f32) {
    self.camera.set_mouse_sensitivity(sensitivity);
  }
  
  pub fn move_camera(&mut self, direction: camera::Direction, delta_time: f32) {
    self.camera.process_movement(direction, delta_time);
  }
  
  pub fn process_mouse_movement(&mut self, x_offset: f32, y_offset: f32) {
    self.camera.process_mouse_movement(x_offset, y_offset);
  }
  
  pub fn get_texture(&mut self, current_buffer: usize) -> Image {
    self.framebuffer_images[current_buffer].to_image()
  }
  
  pub fn add_model(&mut self, instance: Arc<Instance>, device: Arc<Device>, reference: String, model: ModelDetails, base_textures: Vec<Option<Image>>, dummy_texture: &Image, command_pool: &CommandPool, descriptor_set_pool: &DescriptorPool, sampler: &Sampler, graphics_queue: &vk::Queue) {
    self.models.push(Model::new(Arc::clone(&instance), Arc::clone(&device), reference, model, base_textures, dummy_texture, command_pool, descriptor_set_pool, sampler, graphics_queue));
  }
  
  pub fn recreate(&mut self, instance: Arc<Instance>, device: Arc<Device>, format: &vk::Format, image_views: &Vec<vk::ImageView>, new_extent: &vk::Extent2D, textures: Vec<(String, Image)>, sampler: &Sampler, command_pool: &CommandPool, graphics_queue: vk::Queue) {
    for i in 0..self.framebuffers.len() {
      self.framebuffers[i].destroy(Arc::clone(&device));
      self.framebuffer_images[i].destroy(Arc::clone(&device));
      self.framebuffer_depth[i].destroy(Arc::clone(&device));
    }
    
    self.framebuffers.clear();
    self.framebuffer_images.clear();
    self.framebuffer_depth.clear();
    
    for _ in 0..image_views.len() {
      self.framebuffer_images.push(ImageAttachment::create_image_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, ImageUsage::colour_attachment_sampled(), format, &vk::Extent3D { width: new_extent.width, height: new_extent.height, depth: 1 }, &Sample::Count1Bit, ImageLayout::Undefined, &ImageTiling::Optimal, &ImageViewType::Type2D));
      
      self.framebuffer_depth.push(ImageAttachment::create_depth_image_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, ImageUsage::depth_stencil_attachment(), &vk::FORMAT_D32_SFLOAT, &vk::Extent3D { width: new_extent.width, height: new_extent.height, depth: 1 }, &Sample::Count1Bit, ImageLayout::Undefined, &ImageTiling::Optimal, &ImageViewType::Type2D));
    }
    
    for i in 0..image_views.len() {
      self.framebuffers.push(Framebuffer::new_with_depth(Arc::clone(&device), &self.renderpass, &new_extent, &self.framebuffer_images[i].get_image_view(), &self.framebuffer_depth[i].get_image_view()));
    }
  }
  
  fn create_pipline(device: Arc<Device>, vertex_shader: &Shader, fragment_shader: &Shader, render_pass: &RenderPass, descriptor_set: &DescriptorSet) -> (Pipeline, Pipeline) {
    let push_constant_size = UniformData::new()
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
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
                  //.enable_depth_clamp()
                  .enable_depth_write()
                  .enable_depth_test()
                  .cull_mode_back()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    let double_pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStageFlagBits::Vertex, push_constant_size as u32)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .vertex_binding(vec!(ModelVertex::vertex_input_binding()))
                  .vertex_attributes(ModelVertex::vertex_input_attributes())
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  .enable_depth_write()
                  .enable_depth_test()
                  .cull_mode_none()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    (pipeline, double_pipeline)
  }
  
  pub fn create_index_buffer(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<u32> {
    let indices = vec!(0, 3, 2, 2, 1, 0, // back side
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
  
  fn create_frame_buffers(device: Arc<Device>, render_pass: &RenderPass, swapchain_extent: &vk::Extent2D, framebuffer_images: &Vec<ImageAttachment>, framebuffer_depth: &Vec<ImageAttachment>) -> Vec<Framebuffer> {
    let mut framebuffers: Vec<Framebuffer> = Vec::with_capacity(framebuffer_images.len());
    
    for i in 0..framebuffer_images.len() {
     // let framebuffer: Framebuffer = Framebuffer::new(Arc::clone(&device), render_pass, swapchain_extent, &framebuffer_images[i].get_image_view());
      let framebuffer: Framebuffer = Framebuffer::new_with_depth(Arc::clone(&device), render_pass, swapchain_extent, &framebuffer_images[i].get_image_view(), &framebuffer_depth[i].get_image_view());
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  pub fn begin_renderpass(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, clear_value: &Vec<vk::ClearValue>, window_size: &vk::Extent2D, current_buffer: usize) -> CommandBufferBuilder {
    cmd.begin_render_pass(Arc::clone(&device), &clear_value, &self.renderpass, &self.framebuffers[current_buffer].internal_object(), &window_size)
  }
  
  pub fn draw_model(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>, model_reference: String, window_width: f32, window_height: f32, current_buffer: usize) -> CommandBufferBuilder {
    let mut cmd = cmd;
    
    if self.models.len() == 0 {
      return cmd;
    }
    
    for i in 0..self.models.len() {
      if self.models[i].reference != model_reference {
        continue;
      }
      
      let fov = 90.0;
      let aspect = window_width / window_height;
      let (c_pos, c_center, c_up) = self.camera.get_look_at();
      
      let camera_position    = Vector4::new(c_pos.x,    c_pos.y,    c_pos.z,    fov);
      let camera_center      = Vector4::new(c_center.x, c_center.y, c_center.z, aspect);
      let camera_up          = Vector4::new(c_up.x,     c_up.y,     c_up.z,     scale.x);
      let model              = Vector4::new(position.x, position.y, position.z, scale.y);
      let rotation           = Vector4::new(rotation.x, rotation.y, rotation.z, scale.z);
      
      for j in 0..self.models[i].vertex_buffers.len() {
        let vertex = &self.models[i].vertex_buffers[j];
        let vertex_count = self.models[i].vertex_count[j];
        let index = &self.models[i].index_buffers[j];
        let index_count = self.models[i].index_count[j];
        
        let descriptor = &self.models[i].descriptor_sets[j];
        
        let base_colour_factor = self.models[i].base_colour_factors[j];
        let (cutoff, mask) = self.models[i].alpha_cutoffs[j];
        let alpha_cutoff = Vector4::new(cutoff, mask, 0.0, 0.0);
        let double_sided = self.models[i].double_sided[j];
        
        let push_constant_data = UniformData::new()
                                 .add_vector4(camera_position)
                                 .add_vector4(camera_center)
                                 .add_vector4(camera_up)
                                 .add_vector4(model)
                                 .add_vector4(rotation)
                                 .add_vector4(base_colour_factor)
                                 .add_vector4(alpha_cutoff);
        
        cmd = cmd.push_constants(Arc::clone(&device), &self.pipeline, ShaderStageFlagBits::Vertex, push_constant_data);
        
        if index_count == 0 {
          cmd = cmd.draw(Arc::clone(&device), &vertex.internal_object(0), vertex_count, 
                                 if double_sided { &self.double_pipeline } else { &self.pipeline },
                                 vec!(*descriptor.set(0)));
        } else {
          cmd = cmd.draw_indexed(Arc::clone(&device), &vertex.internal_object(0),
                                 &index.internal_object(0),
                                 index_count, 
                                 if double_sided { &self.double_pipeline } else { &self.pipeline },
                                 vec!(*descriptor.set(0)));
        }
      }
    }
    
    cmd
    /*
    let index_count = 36;
    
    cmd.draw_indexed(Arc::clone(&device), &self.vertex_buffer.internal_object(0),
                             &self.index_buffer.internal_object(0),
                             index_count, &self.pipeline,
                             vec!(*self.descriptor_sets[current_buffer].set(0)))*/
  }
  
  pub fn destroy(&mut self, device: Arc<Device>) {
    self.index_buffer.destroy(Arc::clone(&device));
    self.vertex_buffer.destroy(Arc::clone(&device));
    
    for model in &self.models {
      model.destroy(Arc::clone(&device));
    }
    
    self.pipeline.destroy(Arc::clone(&device));
    self.double_pipeline.destroy(Arc::clone(&device));
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
    
    for images in &self.framebuffer_depth {
      images.destroy(Arc::clone(&device));
    }
    
    self.renderpass.destroy(Arc::clone(&device));
  }
}
