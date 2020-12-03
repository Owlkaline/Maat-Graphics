use crate::vk;

use gltf::material::AlphaMode;

use crate::math;
use crate::camera::PerspectiveCamera;
use crate::camera::PerspectiveCameraDirection;
use crate::gltf_interpreter::ModelDetails;

use crate::vulkan::vkenums::{ImageType, ImageUsage, ImageViewType, SampleCount, ImageTiling, AttachmentLoadOp, AttachmentStoreOp, ImageLayout, ImageAspect, ShaderStage, VertexInputRate, AddressMode, MipmapMode, VkBool};

use crate::vulkan::{Instance, Device, RenderPass, Shader, Pipeline, PipelineBuilder, DescriptorSet, UpdateDescriptorSets, DescriptorSetBuilder, ImageAttachment, AttachmentInfo, SubpassInfo, RenderPassBuilder, Sampler, SamplerBuilder};
use crate::vulkan::buffer::{Buffer, BufferUsage, UniformBufferBuilder, UniformData, Framebuffer, CommandBufferBuilder};
use crate::vulkan::pool::{DescriptorPool, CommandPool};
use crate::shaders::FinalShader;
use crate::shaders::FinalVertex;

use crate::cgmath::{Vector2, Vector3, Vector4};

use std::mem;
use std::sync::Arc;

const MAX_INSTANCES: usize = 2048;
const _INSTANCED_SIZE: usize = 16;

#[derive(Clone)]
pub struct Light {
  pos: Vector3<f32>,
  colour: Vector3<f32>,
  intensity: f32,
}

impl Light {
  pub fn new() -> Light {
    Light {
      pos: Vector3::new(0.0, 0.0, 0.0),
      colour: Vector3::new(1.0, 1.0, 1.0),
      intensity: 100.0,
    }
  }
  
  pub fn _off() -> Light {
    Light {
      pos: Vector3::new(0.0, 0.0, 0.0),
      colour: Vector3::new(0.0, 0.0, 0.0),
      intensity: 0.0,
    }
  }
  
  pub fn update(&mut self, position: Vector3<f32>, colour: Vector3<f32>, intensity: f32) {
    self.pos = position;
    self.colour = colour;
    self.intensity = intensity;
  }
}

#[derive(Clone)]
pub struct ModelVertex {
  pos: Vector3<f32>,
  normal: Vector3<f32>,
  uvs: Vector2<f32>,
  colour: Vector4<f32>,
  tangent: Vector4<f32>,
}

#[derive(Clone)]
pub struct ModelInstanceData {
  model: Vector4<f32>,
  rotation: Vector4<f32>,
  colour: Vector4<f32>,
  hologram: Vector4<f32>,
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

impl ModelInstanceData {
  pub fn vertex_input_binding() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription {
      binding: 1,
      stride: (mem::size_of::<ModelInstanceData>()) as u32,
      inputRate: VertexInputRate::Instance.to_bits(),
    }
  }
  
  pub fn vertex_input_attributes() -> Vec<vk::VertexInputAttributeDescription> {
    let mut vertex_input_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> = Vec::with_capacity(2);
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 5,
        binding: 1,
        format: vk::FORMAT_R32G32B32A32_SFLOAT,
        offset: offset_of!(ModelInstanceData, model) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 6,
        binding: 1,
        format: vk::FORMAT_R32G32B32A32_SFLOAT,
        offset: offset_of!(ModelInstanceData, rotation) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 7,
        binding: 1,
        format: vk::FORMAT_R32G32B32A32_SFLOAT,
        offset: offset_of!(ModelInstanceData, colour) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 8,
        binding: 1,
        format: vk::FORMAT_R32G32B32A32_SFLOAT,
        offset: offset_of!(ModelInstanceData, hologram) as u32,
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
  
  _base_colour_factors: Vec<Vector4<f32>>,
  _alpha_cutoffs: Vec<(f32, f32)>,
  double_sided: Vec<bool>,
  
  uniform_buffers: Vec<Buffer<f32>>,
}

impl Model {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, reference: String, model: ModelDetails, base_textures: Vec<Option<ImageAttachment>>, dummy_texture: &ImageAttachment, command_pool: &CommandPool, descriptor_set_pool: &DescriptorPool, sampler: &Sampler, graphics_queue: &vk::Queue) -> Model {
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
    
    let mut uniform_buffers = Vec::with_capacity(num_models);
    
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
                       .anisotropy(VkBool::False)
                       .max_anisotropy(1.0)
                       .build(Arc::clone(&device));
        samplers.push(temp_sampler.clone());
        sampler = &temp_sampler;
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
      
      base_colour_factors.push(math::array4_to_vec4(base_colour_factor));
      alpha_cutoffs.push((alpha_cutoff, alpha_mask));
      double_sided.push(double_side);
      
      let use_base_texture = if model.base_colour_texture(i).is_some() { 1.0 } else { -1.0 };
      let use_metallic_roughness_texture = if model.metallic_roughness_texture(i).is_some() { 1.0 } else { -1.0 };
      let use_normal_texture = if model.normal_texture(i).is_some() { 1.0 } else { -1.0 };
      let use_occlusion_texture = if model.occlusion_texture(i).is_some() { 1.0 } else { -1.0 };
      let use_emissive_texture = if model.emissive_texture(i).is_some() { 1.0 } else { -1.0 };
      
      let metallic_factor = model.metallic_factor(i);
      let roughness_factor = model.roughness_factor(i);
      let normal_scale = model.normal_texture_scale(i);
      let occlusion_strength = model.occlusion_texture_strength(i);
      let emissive_factor = math::array3_to_vec3(model.emissive_factor(i)); // vec3
      
      let mut uniform_buffer = UniformBufferBuilder::new()
         .set_binding(0)
         .add_vector4()
         .add_vector4()
         .add_vector4()
         .add_vector4()
         .add_vector4()
         .build(Arc::clone(&instance), Arc::clone(&device), 1);
         
      let uniform_data = UniformData::new()
                           .add_vector4(Vector4::new(use_base_texture, use_metallic_roughness_texture,
                                                     use_normal_texture, use_occlusion_texture))
                           .add_vector4(Vector4::new(use_emissive_texture, normal_scale, alpha_cutoff, alpha_mask))
                           .add_vector4(math::array4_to_vec4(base_colour_factor))
                           .add_vector4(Vector4::new(metallic_factor, roughness_factor, occlusion_strength, 0.0))
                           .add_vector4(Vector4::new(emissive_factor.x, emissive_factor.y, emissive_factor.z, 0.0))
                           .build(Arc::clone(&device));
      
      uniform_buffer.fill_entire_buffer_all_frames(Arc::clone(&device), uniform_data);
      
      let descriptor_set;
      descriptor_set = DescriptorSetBuilder::new()
                           .vertex_uniform_buffer(0)
                         //  .vertex_uniform_buffer(1)
                           .fragment_combined_image_sampler(1)
                           .fragment_combined_image_sampler(2)
                           .fragment_combined_image_sampler(3)
                           .fragment_combined_image_sampler(4)
                           .fragment_combined_image_sampler(5)
                           .build(Arc::clone(&device), &descriptor_set_pool, 1);
      
      let mut base_texture = dummy_texture;
      let mut mro_texture = dummy_texture;
      let mut normal_texture = dummy_texture;
      let mut occlusion_texture = dummy_texture;
      let mut emissive_texture = dummy_texture;
      
      if let Some(ref texture) = &base_textures[i*5 + 0] { // base texture
        base_texture = texture;
      }
      if let Some(ref texture) = &base_textures[i*5 + 1] { // mro texture
        mro_texture = texture;
      }
      if let Some(ref texture) = &base_textures[i*5 + 2] { // normal texture
        normal_texture = texture;
      }
      if let Some(ref texture) = &base_textures[i*5 + 3] { // occlusion texture
        occlusion_texture = texture;
      }
      if let Some(ref texture) = &base_textures[i*5 + 4] { // emissive texture
        emissive_texture = texture;
      }
      /* UpdateDescriptorSets::new()
             .add_built_uniformbuffer(0, &mut uniform_buffer)
             .add_sampled_image(1, &texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .finish_update(Arc::clone(&device), &descriptor_set);
      } else {
        UpdateDescriptorSets::new()
             .add_built_uniformbuffer(0, &mut uniform_buffer)
             .add_sampled_image(1, &dummy_texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .finish_update(Arc::clone(&device), &descriptor_set);
      }*/
      
      UpdateDescriptorSets::new()
             .add_built_uniformbuffer(0, &mut uniform_buffer)
             .add_sampled_image(1, base_texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .add_sampled_image(2, mro_texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .add_sampled_image(3, normal_texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .add_sampled_image(4, occlusion_texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .add_sampled_image(5, emissive_texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
             .finish_update(Arc::clone(&device), &descriptor_set);
      
      uniform_buffers.push(uniform_buffer);
      descriptor_sets.push(descriptor_set);
    }
    
    Model {
      vertex_buffers,
      index_buffers,
      vertex_count,
      index_count,
      
      descriptor_sets,
      samplers,
      reference: reference.to_string(),
      
      _base_colour_factors: base_colour_factors,
      _alpha_cutoffs: alpha_cutoffs,
      double_sided,
      
      uniform_buffers,
    }
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    for vertex in &self.vertex_buffers {
      vertex.destroy(Arc::clone(&device));
    }
    
    for index in &self.index_buffers {
      index.destroy(Arc::clone(&device));
    }
    
    for buffer in &self.uniform_buffers {
      buffer.destroy(Arc::clone(&device));
    }
    
    for descriptor in &self.descriptor_sets {
      descriptor.destroy(Arc::clone(&device));
    }
    
    for sampler in &self.samplers {
      sampler.destroy(Arc::clone(&device));
    }
  }
  
  fn create_index_buffer(instance: Arc<Instance>, device: Arc<Device>, indexs: Vec<u32>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> (Buffer<u32>, u32) {
    let num_index = indexs.len() as u32;
    
    let usage = BufferUsage::index_buffer();
    let buffer: Buffer<u32> = Buffer::device_local_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), command_pool, graphics_queue, usage, indexs);
    
    (buffer, num_index)
  }
  
  fn create_vertex_buffer(instance: Arc<Instance>, device: Arc<Device>, vertexs: Vec<ModelVertex>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<ModelVertex> {
    let usage = BufferUsage::vertex_buffer();
    Buffer::<ModelVertex>::device_local_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), command_pool, graphics_queue, usage, vertexs)
  }
}

pub struct ModelShader {
  renderpass: RenderPass,
  framebuffers: Vec<Framebuffer>,
  framebuffer_colour_images: Vec<ImageAttachment>,
  framebuffer_albedo_colour_images: Vec<ImageAttachment>,
  framebuffer_mro_images: Vec<ImageAttachment>,
  framebuffer_emissive_images: Vec<ImageAttachment>,
  framebuffer_normal_images: Vec<ImageAttachment>,
  framebuffer_position_images: Vec<ImageAttachment>,
  framebuffer_msaa_images: Vec<ImageAttachment>,
  framebuffer_depth_images: Vec<ImageAttachment>,
  framebuffer_msaa_depth_images: Vec<ImageAttachment>,
  framebuffer_remaining_msaa_images: Vec<ImageAttachment>,
  descriptor_sets: Vec<DescriptorSet>,
  deffered_descriptor_set: DescriptorSet,
  dummy_uniform_buffer: Buffer<f32>,
  
  models: Vec<Model>,
  
  vertex_buffer_deffered: Buffer<FinalVertex>,
  index_buffer_deffered: Buffer<u32>,
  
  pipeline: Pipeline,
  pipeline_deffered: Pipeline,
  double_pipeline: Pipeline,
  instanced_pipeline: Pipeline,
  instanced_double_pipeline: Pipeline,
  instanced_cpu_buffers: Vec<(String, Buffer<f32>)>,
  instanced_cpu_data: Vec<UniformData>,
  
  vertex_shader: Shader,
  fragment_shader: Shader,
  vertex_shader_deffered: Shader,
  fragment_shader_deffered: Shader,
  vertex_shader_instanced: Shader,
  
  msaa: SampleCount,
  camera: PerspectiveCamera,
  
  scanline: f32,
  light: Light,
}

impl ModelShader {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, current_extent: &vk::Extent2D, format: &vk::Format, sampler: &Sampler, image_views: &Vec<vk::ImageView>, texture_image: &ImageAttachment, descriptor_set_pool: &DescriptorPool, command_pool: &CommandPool, graphics_queue: &vk::Queue, msaa: &SampleCount) -> ModelShader {
    let vertex_shader_instanced = Shader::new(Arc::clone(&device), include_bytes!("./sprv/VkModelInstancedVert.spv"));
    let vertex_shader = Shader::new(Arc::clone(&device), include_bytes!("./sprv/VkModelVert.spv"));
    
    let fragment_shader = Shader::new(Arc::clone(&device), include_bytes!("./sprv/VkModelFrag.spv"));
    let vertex_shader_deffered = Shader::new(Arc::clone(&device), include_bytes!("./sprv/VkModelDefferedVert.spv"));
    let fragment_shader_deffered = Shader::new(Arc::clone(&device), include_bytes!("./sprv/VkModelDefferedFrag.spv"));
    
    let colour_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(&SampleCount::OneBit)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::Store)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let albedo_colour_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(&SampleCount::OneBit)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let mro_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(&SampleCount::OneBit)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    let emissive_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(&SampleCount::OneBit)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
                                
    let normal_attachment = AttachmentInfo::new()
                                .format(vk::FORMAT_R8G8B8A8_SNORM)
                                .multisample(&SampleCount::OneBit)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let position_attachment = AttachmentInfo::new()
                                .format(vk::FORMAT_R16G16B16A16_SFLOAT)
                                .multisample(&SampleCount::OneBit)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
                                
    let depth_attachment = AttachmentInfo::new()
                                .format(vk::FORMAT_D32_SFLOAT)
                                .multisample(&SampleCount::OneBit)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::Store)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::DepthStencilAttachmentOptimal);
    
    let msaa_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(msaa)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let msaa_depth_attachment = AttachmentInfo::new()
                                .format(vk::FORMAT_D32_SFLOAT)
                                .multisample(msaa)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::Store)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::DepthStencilAttachmentOptimal);
    
    let msaa_albedo_colour_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(msaa)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let msaa_mro_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(msaa)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
                                
    let msaa_emissive_attachment = AttachmentInfo::new()
                                .format(*format)
                                .multisample(msaa)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
                                
    let msaa_normal_attachment = AttachmentInfo::new()
                                .format(vk::FORMAT_R8G8B8A8_SNORM)
                                .multisample(msaa)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let msaa_position_attachment = AttachmentInfo::new()
                                .format(vk::FORMAT_R16G16B16A16_SFLOAT)
                                .multisample(msaa)
                                .load(AttachmentLoadOp::Clear)
                                .store(AttachmentStoreOp::DontCare)
                                .stencil_load(AttachmentLoadOp::DontCare)
                                .stencil_store(AttachmentStoreOp::DontCare)
                                .initial_layout(ImageLayout::Undefined)
                                .final_layout(ImageLayout::ShaderReadOnlyOptimal)
                                .image_usage(ImageLayout::ColourAttachmentOptimal);
    
    let mut subpass = SubpassInfo::new().add_colour_attachment(if msaa == &SampleCount::OneBit { 0 } else { 7 })
                                    .add_colour_attachment(if msaa == &SampleCount::OneBit { 1 } else { 9 })
                                    .add_colour_attachment(if msaa == &SampleCount::OneBit { 2 } else { 10 })
                                    .add_colour_attachment(if msaa == &SampleCount::OneBit { 3 } else { 11 })
                                    .add_colour_attachment(if msaa == &SampleCount::OneBit { 4 } else { 12 })
                                    .add_depth_stencil(if msaa == &SampleCount::OneBit { 5 } else { 8 })
                                    .add_colour_attachment(if msaa == &SampleCount::OneBit { 6 } else { 13 });
    
    if msaa != &SampleCount::OneBit {
      subpass = subpass.add_resolve_attachment(0)
                       .add_resolve_attachment(1)
                       .add_resolve_attachment(2)
                       .add_resolve_attachment(3)
                       .add_resolve_attachment(4)
                       .add_resolve_attachment(6)
                       .add_resolve_attachment(5);
                       
    }
    
    let second_subpass = SubpassInfo::new().add_colour_attachment(0)
                                           .add_input_attachment(1)
                                           .add_input_attachment(2)
                                           .add_input_attachment(3)
                                           .add_input_attachment(4)
                                           .add_input_attachment(6)
                                           .add_input_attachment(5);
    
    let render_pass = RenderPassBuilder::new();
    let mut render_pass = render_pass.add_attachment(colour_attachment) // 0
                                     .add_attachment(albedo_colour_attachment) // 1
                                     .add_attachment(mro_attachment) // 2
                                     .add_attachment(emissive_attachment) // 3
                                     .add_attachment(normal_attachment) // 4
                                     .add_attachment(depth_attachment) // 6
                                     .add_attachment(position_attachment); // 5
    
    if msaa != &SampleCount::OneBit {
      render_pass = render_pass.add_attachment(msaa_attachment); // 7
      render_pass = render_pass.add_attachment(msaa_depth_attachment); // 8
      render_pass = render_pass.add_attachment(msaa_albedo_colour_attachment); // 9
      render_pass = render_pass.add_attachment(msaa_mro_attachment); // 10
      render_pass = render_pass.add_attachment(msaa_emissive_attachment); // 11
      render_pass = render_pass.add_attachment(msaa_normal_attachment); // 12
      render_pass = render_pass.add_attachment(msaa_position_attachment); // 13
    }
    
    let render_pass = render_pass.add_subpass(subpass)
                                 .add_subpass(second_subpass)
                                 .build(Arc::clone(&device));
    
    let (framebuffer_colour_images, framebuffer_albedo_colour_images, framebuffer_mro_images,
         framebuffer_emissive_images, framebuffer_normal_images, framebuffer_position_images, 
         framebuffer_msaa_images, framebuffer_depth_images, framebuffer_msaa_depth_images, framebuffer_remaining_msaa_images, framebuffers) 
                      = ModelShader::create_frame_buffers(Arc::clone(&instance), Arc::clone(&device),
                                                           &render_pass, current_extent, format, 
                                                           msaa, image_views.len(), command_pool,
                                                           graphics_queue);
    
    let mut descriptor_sets = Vec::new();
    let mut uniform_buffer = UniformBufferBuilder::new().add_vector4().build(Arc::clone(&instance), Arc::clone(&device), 1);
    for i in 0..image_views.len() {
      descriptor_sets.push(DescriptorSetBuilder::new()
        .vertex_uniform_buffer(0)
        .fragment_combined_image_sampler(1)
        .fragment_combined_image_sampler(2)
        .fragment_combined_image_sampler(3)
        .fragment_combined_image_sampler(4)
        .fragment_combined_image_sampler(5)
        .build(Arc::clone(&device), &descriptor_set_pool, 1));
      
      uniform_buffer.destroy(Arc::clone(&device));
      uniform_buffer = UniformBufferBuilder::new()
        .set_binding(0)
        .add_vector4()
        .add_vector4()
        .add_vector4()
        .add_vector4()
        .add_vector4()
        .build(Arc::clone(&instance), Arc::clone(&device), 1);
      
      let uniform_data = UniformData::new()
                           .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                           .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                           .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                           .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                           .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0));
      
      UpdateDescriptorSets::new()
        .add_uniformbuffer(Arc::clone(&device), 0, &mut uniform_buffer, uniform_data)
        .add_sampled_image(1, &texture_image, ImageLayout::ShaderReadOnlyOptimal, &sampler)
       .finish_update(Arc::clone(&device), &descriptor_sets[i]);
    }
    
        
    let camera = PerspectiveCamera::default_vk();
    
    let deffered_descriptor_set = DescriptorSetBuilder::new()
          .fragment_input_attachment(1)
          .fragment_input_attachment(2)
          .fragment_input_attachment(3)
          .fragment_input_attachment(4)
          .fragment_input_attachment(6)
          .fragment_input_attachment(5)
          .build(Arc::clone(&device), &descriptor_set_pool, image_views.len() as u32);
      
    UpdateDescriptorSets::new()
      .add_input_attachment_image(1, &framebuffer_albedo_colour_images)
      .add_input_attachment_image(2, &framebuffer_mro_images)
      .add_input_attachment_image(3, &framebuffer_emissive_images)
      .add_input_attachment_image(4, &framebuffer_normal_images)
      .add_input_attachment_image(6, &framebuffer_depth_images)
      .add_input_attachment_image(5, &framebuffer_position_images)
     .finish_update(Arc::clone(&device), &deffered_descriptor_set);
    
    let (pipeline, pipeline_deffered, double_pipeline) = ModelShader::create_pipline(Arc::clone(&device), &vertex_shader, &fragment_shader, &vertex_shader_deffered, &fragment_shader_deffered, &render_pass, &descriptor_sets[0], &deffered_descriptor_set, msaa);
    let (instanced_pipeline, instanced_double_pipeline) = ModelShader::create_instanced_pipline(Arc::clone(&device), &vertex_shader_instanced, &fragment_shader, &render_pass, &descriptor_sets[0], msaa);
    
    let vertex_buffer_deffered = FinalShader::create_vertex_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    let index_buffer_deffered = FinalShader::create_index_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    
    ModelShader {
      renderpass: render_pass,
      framebuffers,
      framebuffer_colour_images,
      framebuffer_albedo_colour_images,
      framebuffer_mro_images,
      framebuffer_emissive_images,
      framebuffer_normal_images,
      framebuffer_position_images,
      framebuffer_msaa_images,
      framebuffer_depth_images,
      framebuffer_msaa_depth_images,
      framebuffer_remaining_msaa_images,
      descriptor_sets,
      deffered_descriptor_set,
      dummy_uniform_buffer: uniform_buffer,
      
      models: Vec::new(),
      
      vertex_buffer_deffered,
      index_buffer_deffered,
      
      pipeline,
      double_pipeline,
      pipeline_deffered,
      instanced_pipeline,
      instanced_double_pipeline,
      instanced_cpu_buffers: Vec::new(),
      instanced_cpu_data: Vec::new(),
      
      vertex_shader,
      fragment_shader,
      vertex_shader_deffered,
      fragment_shader_deffered,
      vertex_shader_instanced,
      
      msaa: *msaa,
      camera,
      
      scanline: 0.0,
      light: Light::new(),
    }
  }
  
  pub fn update_scanline(&mut self, delta_time: f32) {
    self.scanline += delta_time;
    if self.scanline > 10000.0 {
      self.scanline = 0.0;
    }
  }
  
  pub fn set_light(&mut self, position: Vector3<f32>, colour: Vector3<f32>, intensity: f32) {
    self.light.update(position, colour, intensity);
  }
  
  pub fn set_camera(&mut self, camera: PerspectiveCamera) {
    self.camera = camera;
  }
  
  pub fn set_camera_move_speed(&mut self, speed: f32) {
    self.camera.set_move_speed(speed);
  }
  
  pub fn set_mouse_sensitivity(&mut self, sensitivity: f32) {
    self.camera.set_mouse_sensitivity(sensitivity);
  }
  
  pub fn move_camera(&mut self, direction: PerspectiveCameraDirection, delta_time: f32) {
    self.camera.process_movement(direction, delta_time);
  }
  
  pub fn process_mouse_movement(&mut self, x_offset: f32, y_offset: f32) {
    self.camera.process_mouse_movement(x_offset, y_offset);
  }
  
  pub fn get_texture(&self, current_buffer: usize) -> ImageAttachment {
    self.framebuffer_colour_images[current_buffer].clone()
  }
  
  pub fn _get_texture_ref(&self, current_buffer: usize) -> &ImageAttachment {
    &self.framebuffer_colour_images[current_buffer]
  }
  
  pub fn add_model(&mut self, instance: Arc<Instance>, device: Arc<Device>, reference: String, model: ModelDetails, base_textures: Vec<Option<ImageAttachment>>, dummy_texture: &ImageAttachment, command_pool: &CommandPool, descriptor_set_pool: &DescriptorPool, sampler: &Sampler, graphics_queue: &vk::Queue) {
    self.models.push(Model::new(Arc::clone(&instance), Arc::clone(&device), reference, model, base_textures, dummy_texture, command_pool, descriptor_set_pool, sampler, graphics_queue));
  }
  
  pub fn remove_model(&mut self, device: Arc<Device>, reference: String) {
    for i in 0..self.models.len() {
      if self.models[i].reference == reference {
        self.models[i].destroy(Arc::clone(&device));
        self.models.remove(i);
        break;
      }
    }
  }
  
  pub fn recreate(&mut self, instance: Arc<Instance>, device: Arc<Device>, format: &vk::Format, image_views: &Vec<vk::ImageView>, new_extent: &vk::Extent2D, command_pool: &CommandPool, graphics_queue: &vk::Queue) {
    for i in 0..self.framebuffers.len() {
      self.framebuffers[i].destroy(Arc::clone(&device));
      
      self.framebuffer_colour_images[i].destroy(Arc::clone(&device));
      self.framebuffer_albedo_colour_images[i].destroy(Arc::clone(&device));
      self.framebuffer_mro_images[i].destroy(Arc::clone(&device));
      self.framebuffer_emissive_images[i].destroy(Arc::clone(&device));
      self.framebuffer_normal_images[i].destroy(Arc::clone(&device));
      self.framebuffer_position_images[i].destroy(Arc::clone(&device));
      self.framebuffer_depth_images[i].destroy(Arc::clone(&device));
      
      if self.msaa != SampleCount::OneBit {
        self.framebuffer_msaa_images[i].destroy(Arc::clone(&device));
        self.framebuffer_msaa_depth_images[i].destroy(Arc::clone(&device));
      }
    }
    
    for framebuffer_msaa_image in &mut self.framebuffer_remaining_msaa_images {
      framebuffer_msaa_image.destroy(Arc::clone(&device));
    }
    
    self.framebuffers.clear();
    self.framebuffer_colour_images.clear();
    self.framebuffer_albedo_colour_images.clear();
    self.framebuffer_mro_images.clear();
    self.framebuffer_emissive_images.clear();
    self.framebuffer_normal_images.clear();
    self.framebuffer_position_images.clear();
    self.framebuffer_msaa_images.clear();
    self.framebuffer_depth_images.clear();
    self.framebuffer_msaa_depth_images.clear();
    self.framebuffer_remaining_msaa_images.clear();
    
    let (framebuffer_colour_images,
         framebuffer_albedo_colour_images,
         framebuffer_mro_images,
         framebuffer_emissive_images,
         framebuffer_normal_images,
         framebuffer_position_images, 
         framebuffer_msaa_images,
         framebuffer_depth_images,
         framebuffer_msaa_depth_images, 
         framebuffer_remaining_msaa_images,
         framebuffers) = ModelShader::create_frame_buffers(Arc::clone(&instance), Arc::clone(&device), 
                                                           &self.renderpass, new_extent, format, 
                                                           &self.msaa, image_views.len(), command_pool, 
                                                           graphics_queue);
    
    self.framebuffers = framebuffers;
    self.framebuffer_colour_images = framebuffer_colour_images;
    self.framebuffer_albedo_colour_images = framebuffer_albedo_colour_images;
    self.framebuffer_mro_images = framebuffer_mro_images;
    self.framebuffer_emissive_images = framebuffer_emissive_images;
    self.framebuffer_normal_images = framebuffer_normal_images;
    self.framebuffer_position_images = framebuffer_position_images;
    self.framebuffer_msaa_images = framebuffer_msaa_images;
    self.framebuffer_depth_images = framebuffer_depth_images;
    self.framebuffer_msaa_depth_images = framebuffer_msaa_depth_images;
    self.framebuffer_remaining_msaa_images = framebuffer_remaining_msaa_images;
    
    UpdateDescriptorSets::new()
      .add_input_attachment_image(1, &self.framebuffer_albedo_colour_images)
      .add_input_attachment_image(2, &self.framebuffer_mro_images)
      .add_input_attachment_image(3, &self.framebuffer_emissive_images)
      .add_input_attachment_image(4, &self.framebuffer_normal_images)
      .add_input_attachment_image(5, &self.framebuffer_position_images)
      .add_input_attachment_image(6, &self.framebuffer_depth_images)
     .finish_update(Arc::clone(&device), &self.deffered_descriptor_set);
  }
  
  fn create_pipline(device: Arc<Device>, vertex_shader: &Shader, fragment_shader: &Shader, vertex_shader_deffered: &Shader, fragment_shader_deffered: &Shader, render_pass: &RenderPass, descriptor_set: &DescriptorSet, deffered_descriptor_set: &DescriptorSet, msaa: &SampleCount) -> (Pipeline, Pipeline, Pipeline) {
    let push_constant_size = UniformData::new()
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .size_non_aligned();
    
    let pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStage::Vertex, push_constant_size as u32)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .vertex_binding(vec!(ModelVertex::vertex_input_binding()))
                  .vertex_attributes(ModelVertex::vertex_input_attributes())
                  .multisample(msaa)
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  //.polygon_mode_line()
                  .enable_depth_write()
                  .enable_depth_test()
                  .cull_mode_back()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    let double_pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStage::Vertex, push_constant_size as u32)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .vertex_binding(vec!(ModelVertex::vertex_input_binding()))
                  .vertex_attributes(ModelVertex::vertex_input_attributes())
                  .multisample(msaa)
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  //.polygon_mode_line()
                  .enable_depth_write()
                  .enable_depth_test()
                  .cull_mode_none()
                  .front_face_counter_clockwise()
                  
                  .build(Arc::clone(&device));
    
    let deffered_push_constant_size = UniformData::new()
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .size_non_aligned();
    
    let deffered_pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader_deffered.get_shader())
                  .fragment_shader(*fragment_shader_deffered.get_shader())
                  .push_constants(ShaderStage::Vertex, deffered_push_constant_size as u32)
                  .subpass(1)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(deffered_descriptor_set.layouts_clone())
                  .vertex_binding(vec!(FinalVertex::vertex_input_binding()))
                  .vertex_attributes(FinalVertex::vertex_input_attributes())
                  //.multisample(msaa)
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  //.polygon_mode_line()
                  .cull_mode_none()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    (pipeline, deffered_pipeline, double_pipeline)
  }
  
  fn create_instanced_pipline(device: Arc<Device>, vertex_shader: &Shader, fragment_shader: &Shader, render_pass: &RenderPass, descriptor_set: &DescriptorSet, msaa: &SampleCount) -> (Pipeline, Pipeline) {
    let push_constant_size = UniformData::new()
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .size(Arc::clone(&device));
    
    let mut attributes = ModelVertex::vertex_input_attributes();
    attributes.append(&mut ModelInstanceData::vertex_input_attributes());
    
    let pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStage::Vertex, push_constant_size as u32)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .vertex_binding(vec!(ModelVertex::vertex_input_binding(), ModelInstanceData::vertex_input_binding()))
                  .vertex_attributes(attributes)
                  .multisample(msaa)
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  //.polygon_mode_line()
                  .enable_depth_write()
                  .enable_depth_test()
                  .cull_mode_back()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    let mut attributes = ModelVertex::vertex_input_attributes();
    attributes.append(&mut ModelInstanceData::vertex_input_attributes());
    
    let double_pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStage::Vertex, push_constant_size as u32)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_set.layouts_clone())
                  .vertex_binding(vec!(ModelVertex::vertex_input_binding(), ModelInstanceData::vertex_input_binding()))
                  .vertex_attributes(attributes)
                  .multisample(msaa)
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  //.polygon_mode_line()
                  .enable_depth_write()
                  .enable_depth_test()
                  .cull_mode_none()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    (pipeline, double_pipeline)
  }
  /*
  pub fn create_index_buffer(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<u32> {
    let indices = vec!(0, 3, 2, 2, 1, 0, // back side
                       7, 3, 4, 4, 3, 0, // right side
                       4, 5, 7, 6, 7, 5, // front side
                       2, 5, 1, 2, 6, 5, // left side
                       5, 4, 0, 0, 1, 5, // top side
                       2, 7, 6, 7, 2, 3  // bottom side
                  );
    
    let usage = BufferUsage::index_buffer();
    Buffer::<u32>::device_local_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), command_pool, graphics_queue, usage, indices)
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
    
    let usage = BufferUsage::vertex_buffer();
    Buffer::<ModelVertex>::device_local_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), command_pool, graphics_queue, usage, cube)
  }*/
  
  fn create_frame_buffers(instance: Arc<Instance>, device: Arc<Device>, render_pass: &RenderPass, swapchain_extent: &vk::Extent2D, format: &vk::Format, msaa: &SampleCount, num_image_views: usize, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> (Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<ImageAttachment>, Vec<Framebuffer>) {
    
    let mut framebuffer_colour_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_albedo_colour_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_mro_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_emissive_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_normal_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_position_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_msaa_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_depth_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_msaa_depth_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_msaa_albedo_colour_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_msaa_mro_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_msaa_emissive_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_msaa_normal_images = Vec::with_capacity(num_image_views);
    let mut framebuffer_msaa_position_images = Vec::with_capacity(num_image_views);
    
    for _ in 0..num_image_views {
      framebuffer_colour_images.push(ImageAttachment::create_image_colour_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::colour_attachment_sampled(), &ImageLayout::Undefined, &SampleCount::OneBit, &ImageViewType::Type2D, format, swapchain_extent.width as u32, swapchain_extent.height as u32));
      
      framebuffer_albedo_colour_images.push(ImageAttachment::create_image_colour_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::colour_input_attachment_sampled(), &ImageLayout::Undefined, &SampleCount::OneBit, &ImageViewType::Type2D, format, swapchain_extent.width as u32, swapchain_extent.height as u32));
      
      framebuffer_mro_images.push(ImageAttachment::create_image_colour_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::colour_input_attachment_sampled(), &ImageLayout::Undefined, &SampleCount::OneBit, &ImageViewType::Type2D, format, swapchain_extent.width as u32, swapchain_extent.height as u32));
      
      framebuffer_emissive_images.push(ImageAttachment::create_image_colour_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::colour_input_attachment_sampled(), &ImageLayout::Undefined, &SampleCount::OneBit, &ImageViewType::Type2D, format, swapchain_extent.width as u32, swapchain_extent.height as u32));
      
      framebuffer_normal_images.push(ImageAttachment::create_image_colour_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::colour_input_attachment_sampled(), &ImageLayout::Undefined, &SampleCount::OneBit, &ImageViewType::Type2D, &vk::FORMAT_R8G8B8A8_SNORM, swapchain_extent.width as u32, swapchain_extent.height as u32));
      
      framebuffer_depth_images.push(ImageAttachment::create_image_depth_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::depth_stencil_input_attachment(), &ImageLayout::Undefined, &SampleCount::OneBit, &ImageViewType::Type2D, &vk::FORMAT_D32_SFLOAT, swapchain_extent.width as u32, swapchain_extent.height as u32));
      
      framebuffer_position_images.push(ImageAttachment::create_image_colour_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::colour_input_attachment_sampled(), &ImageLayout::Undefined, &SampleCount::OneBit, &ImageViewType::Type2D, &vk::FORMAT_R16G16B16A16_SFLOAT, swapchain_extent.width as u32, swapchain_extent.height as u32));
      
      if msaa != &SampleCount::OneBit {
        framebuffer_msaa_images.push(ImageAttachment::create_image_msaa_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::transient_colour_input_attachment(), &ImageLayout::Undefined, &ImageLayout::ColourAttachmentOptimal, &ImageAspect::Colour, msaa, &ImageViewType::Type2D, format, command_pool, graphics_queue, swapchain_extent.width as u32, swapchain_extent.height as u32));
        
        framebuffer_msaa_depth_images.push(ImageAttachment::create_image_depth_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::depth_stencil_input_attachment(), &ImageLayout::Undefined, msaa, &ImageViewType::Type2D, &vk::FORMAT_D32_SFLOAT, swapchain_extent.width as u32, swapchain_extent.height as u32));
        
        framebuffer_msaa_albedo_colour_images.push(ImageAttachment::create_image_msaa_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::transient_colour_input_attachment(), &ImageLayout::Undefined, &ImageLayout::ColourAttachmentOptimal, &ImageAspect::Colour, msaa, &ImageViewType::Type2D, format, command_pool, graphics_queue, swapchain_extent.width as u32, swapchain_extent.height as u32));
        
        framebuffer_msaa_mro_images.push(ImageAttachment::create_image_msaa_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::transient_colour_input_attachment(), &ImageLayout::Undefined, &ImageLayout::ColourAttachmentOptimal, &ImageAspect::Colour, msaa, &ImageViewType::Type2D, format, command_pool, graphics_queue, swapchain_extent.width as u32, swapchain_extent.height as u32));
        
        framebuffer_msaa_emissive_images.push(ImageAttachment::create_image_msaa_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::transient_colour_input_attachment(), &ImageLayout::Undefined, &ImageLayout::ColourAttachmentOptimal, &ImageAspect::Colour, msaa, &ImageViewType::Type2D, format, command_pool, graphics_queue, swapchain_extent.width as u32, swapchain_extent.height as u32));
        
        framebuffer_msaa_normal_images.push(ImageAttachment::create_image_msaa_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::transient_colour_input_attachment(), &ImageLayout::Undefined, &ImageLayout::ColourAttachmentOptimal, &ImageAspect::Colour, msaa, &ImageViewType::Type2D, &vk::FORMAT_R8G8B8A8_SNORM, command_pool, graphics_queue, swapchain_extent.width as u32, swapchain_extent.height as u32));
        
        framebuffer_msaa_position_images.push(ImageAttachment::create_image_msaa_attachment(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &ImageUsage::transient_colour_input_attachment(), &ImageLayout::Undefined, &ImageLayout::ColourAttachmentOptimal, &ImageAspect::Colour, msaa, &ImageViewType::Type2D, &vk::FORMAT_R16G16B16A16_SFLOAT, command_pool, graphics_queue, swapchain_extent.width as u32, swapchain_extent.height as u32));
      }
    }
    
    let mut framebuffers: Vec<Framebuffer> = Vec::with_capacity(framebuffer_colour_images.len() +
                                                                framebuffer_albedo_colour_images.len() +
                                                                framebuffer_emissive_images.len() +
                                                                framebuffer_mro_images.len() +
                                                                framebuffer_normal_images.len() +
                                                                framebuffer_position_images.len() +
                                                                framebuffer_depth_images.len() +
                                                                framebuffer_msaa_depth_images.len());
    
    for i in 0..framebuffer_colour_images.len() {
      let mut temp_image_views = Vec::new();
      
      temp_image_views.push(framebuffer_colour_images[i].get_image_view().clone());
      temp_image_views.push(framebuffer_albedo_colour_images[i].get_image_view().clone());
      temp_image_views.push(framebuffer_mro_images[i].get_image_view().clone());
      temp_image_views.push(framebuffer_emissive_images[i].get_image_view().clone());
      temp_image_views.push(framebuffer_normal_images[i].get_image_view().clone());
      
      temp_image_views.push(framebuffer_depth_images[i].get_image_view().clone());
      
      temp_image_views.push(framebuffer_position_images[i].get_image_view().clone());
      
      if msaa != &SampleCount::OneBit {
        temp_image_views.push(framebuffer_msaa_images[i].get_image_view().clone());
        temp_image_views.push(framebuffer_msaa_depth_images[i].get_image_view().clone());
        temp_image_views.push(framebuffer_msaa_albedo_colour_images[i].get_image_view().clone());
        temp_image_views.push(framebuffer_msaa_mro_images[i].get_image_view().clone());
        temp_image_views.push(framebuffer_msaa_emissive_images[i].get_image_view().clone());
        temp_image_views.push(framebuffer_msaa_normal_images[i].get_image_view().clone());
        temp_image_views.push(framebuffer_msaa_position_images[i].get_image_view().clone());
      }
      
      let framebuffer: Framebuffer = Framebuffer::new_with_imageviews(Arc::clone(&device), 
                                                                      render_pass, 
                                                                      swapchain_extent, 
                                                                      temp_image_views);
      
      framebuffers.push(framebuffer)
    }
    
    let mut framebuffer_remaining_msaa_images = Vec::new();
    if msaa != &SampleCount::OneBit {
      for i in 0..framebuffer_msaa_albedo_colour_images.len() {
       framebuffer_remaining_msaa_images.push(framebuffer_msaa_albedo_colour_images[i].clone());
       framebuffer_remaining_msaa_images.push(framebuffer_msaa_mro_images[i].clone());
       framebuffer_remaining_msaa_images.push(framebuffer_msaa_emissive_images[i].clone());
       framebuffer_remaining_msaa_images.push(framebuffer_msaa_normal_images[i].clone());
       framebuffer_remaining_msaa_images.push(framebuffer_msaa_position_images[i].clone());
      }
    }
    
    (framebuffer_colour_images, framebuffer_albedo_colour_images, framebuffer_mro_images, 
     framebuffer_emissive_images, framebuffer_normal_images, framebuffer_position_images, 
     framebuffer_msaa_images, framebuffer_depth_images, framebuffer_msaa_depth_images, framebuffer_remaining_msaa_images, framebuffers)
  }
  
  pub fn begin_renderpass(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, clear_value: &Vec<vk::ClearValue>, window_size: &vk::Extent2D, current_buffer: usize) -> CommandBufferBuilder {
    cmd.begin_render_pass(Arc::clone(&device), clear_value, &self.renderpass, &self.framebuffers[current_buffer].internal_object(), &window_size)
  }
  
  pub fn add_instanced_buffer(&mut self, instance: Arc<Instance>, device: Arc<Device>, image_views: u32, model_reference: String) {
    
    for i in 0..self.instanced_cpu_buffers.len() {
      if self.instanced_cpu_buffers[i].0 == model_reference.to_string() {
        return;
      }
    }
    
    let mut instanced_data = Vec::with_capacity(MAX_INSTANCES*16);
    for _ in 0..(MAX_INSTANCES*16) {
      instanced_data.push(0.0);
    }
    
    let usage = BufferUsage::vertex_transfer_src_buffer();
    let instanced_cpu_buffer = Buffer::cpu_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), &usage, image_views, instanced_data);
    
    self.instanced_cpu_buffers.push((model_reference, instanced_cpu_buffer));
    self.instanced_cpu_data.push(UniformData::new());
  }
  
  pub fn draw_model(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>, model_reference: String, hologram: bool, window_width: f32, window_height: f32, _delta_time: f32) -> CommandBufferBuilder {
    let mut cmd = cmd;
    
    if self.models.len() == 0 {
      return cmd;
    }
    
    for i in 0..self.models.len() {
      if self.models[i].reference != model_reference {
        continue;
      }
      
      let fov = 60.0;
      let aspect = window_width / window_height;
      let (c_pos, c_center, c_up) = self.camera.get_look_at();
      
      let camera_position    = Vector4::new(c_pos.x,    c_pos.y,    c_pos.z,    fov);
      let camera_center      = Vector4::new(c_center.x, c_center.y, c_center.z, aspect);
      let camera_up          = Vector4::new(c_up.x,     c_up.y,     c_up.z,     scale.x);
      let model              = Vector4::new(position.x, position.y, position.z, scale.y);
      let rotation           = Vector4::new(rotation.x, rotation.y, rotation.z, scale.z);
     // println!("MS: {}: Rotation {:?}", model_reference, rotation);
      let hologram           = Vector4::new(if hologram { 1.0 } else { -1.0 }, self.scanline, 0.0, 0.0);
      
      for j in 0..self.models[i].vertex_buffers.len() {
        let vertex = &self.models[i].vertex_buffers[j];
        let vertex_count = self.models[i].vertex_count[j];
        let index = &self.models[i].index_buffers[j];
        let index_count = self.models[i].index_count[j];
        
        let descriptor = &self.models[i].descriptor_sets[j];
       
        let double_sided = self.models[i].double_sided[j];
        
        let push_constant_data = UniformData::new()
                                 .add_vector4(camera_position)
                                 .add_vector4(camera_center)
                                 .add_vector4(camera_up)
                                 .add_vector4(model)
                                 .add_vector4(rotation)
                                 .add_vector4(hologram);
        
        cmd = cmd.push_constants(Arc::clone(&device), &self.pipeline, ShaderStage::Vertex, push_constant_data);
        
        if index_count == 0 {
          cmd = cmd.draw(Arc::clone(&device), &vertex.internal_object(0), vertex_count, 
                                 if double_sided { &self.double_pipeline } else { &self.pipeline },
                                 vec!(*descriptor.set(0)),
                                 Vec::with_capacity(0));
        } else {
          cmd = cmd.draw_indexed(Arc::clone(&device), &vertex.internal_object(0),
                                 &index.internal_object(0),
                                 index_count, 
                                 if double_sided { &self.double_pipeline } else { &self.pipeline },
                                 vec!(*descriptor.set(0)),
                                 Vec::with_capacity(0));
        }
      }
    }
    
    cmd
  }
  
  pub fn add_instanced_model(&mut self, position: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>, colour: Vector4<f32>, model_reference: String, hologram: bool) {
    if self.models.len() == 0 {
      return;
    }
    
    for i in 0..self.instanced_cpu_buffers.len() {
      if self.instanced_cpu_buffers[i].0 != model_reference {
        continue;
      }
      
      let model              = Vector4::new(position.x, position.y, position.z, scale.x);
      let rotation           = Vector4::new(rotation.x, rotation.y, rotation.z, scale.y);
      let colour             = colour;
      let hologram           = Vector4::new(if hologram { 1.0 } else { -1.0 }, self.scanline, 0.0, scale.z);
      
      let details = self.instanced_cpu_data[i].clone();
      let data = details.clone();
      self.instanced_cpu_data[i] = data
                                    .add_vector4(model)
                                    .add_vector4(rotation)
                                    .add_vector4(colour)
                                    .add_vector4(hologram);
    }
  }
  
  pub fn draw_instanced(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, model_reference: String, window_width: f32, window_height: f32, _delta_time: f32) -> CommandBufferBuilder {
    let mut cmd = cmd;
    
    if self.models.len() == 0 || self.instanced_cpu_buffers.len() == 0 {
      return cmd;
    }
    
    let mut idx = 0;
    for j in 0..self.instanced_cpu_buffers.len() {
      if self.instanced_cpu_buffers[j].0 != model_reference {
        continue;
      }
      
      idx = j
    };
    
    let (model_reference, mut buffer) = self.instanced_cpu_buffers[idx].clone();
    let mut instanced_data = self.instanced_cpu_data[idx].clone();
    
    let data = instanced_data.build(Arc::clone(&device));
    let num_instances = (data.len() as f32 / 16.0) as u32;
    
    if num_instances == 0 {
      return cmd;
    }
    
    buffer.fill_entire_buffer_single_frame(Arc::clone(&device), 0, data);
    
    for i in 0..self.models.len() {
      if self.models[i].reference != model_reference {
        continue;
      }
      
      let fov = 60.0;
      let aspect = window_width / window_height;
      let (c_pos, c_center, c_up) = self.camera.get_look_at();
      
      let camera_position    = Vector4::new(c_pos.x,    c_pos.y,    c_pos.z,    fov);
      let camera_center      = Vector4::new(c_center.x, c_center.y, c_center.z, aspect);
      let camera_up          = Vector4::new(c_up.x,     c_up.y,     c_up.z,     0.0); // x, y, z, 
      
      for j in 0..self.models[i].vertex_buffers.len() {
        let vertex = &self.models[i].vertex_buffers[j];
        let vertex_count = self.models[i].vertex_count[j];
        let index = &self.models[i].index_buffers[j];
        let index_count = self.models[i].index_count[j];
        
        let descriptor = &self.models[i].descriptor_sets[j];
       
        let double_sided = self.models[i].double_sided[j];
        
        let push_constant_data = UniformData::new()
                                 .add_vector4(camera_position)
                                 .add_vector4(camera_center)
                                 .add_vector4(camera_up);
        
        cmd = cmd.push_constants(Arc::clone(&device), &self.pipeline, ShaderStage::Vertex, push_constant_data);
        
        if index_count == 0 {
          
          cmd = cmd.draw_instanced(Arc::clone(&device), 
                                   &vertex.internal_object(0), 
                                   &buffer.internal_object(0),
                                   vertex_count, 
                                   num_instances,
                                 if double_sided { &self.instanced_double_pipeline } else { &self.instanced_pipeline },
                                 vec!(*descriptor.set(0)));
          println!("Instanced draw Not indexed! Not Implemented!");
        } else {
          
          cmd = cmd.draw_instanced_indexed(Arc::clone(&device), 
                                       &vertex.internal_object(0),
                                       &index.internal_object(0),
                                       &buffer.internal_object(0),
                                       index_count,
                                       num_instances,
                                       if double_sided { &self.instanced_double_pipeline } else { &self.instanced_pipeline },
                                       vec!(*descriptor.set(0)));
        }
        
      }
    }
    
    self.instanced_cpu_data[idx] = UniformData::new();
    
    cmd
  }
  
    pub fn draw_deffered(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, current_buffer: usize) -> CommandBufferBuilder {
      let mut cmd = cmd;
      
      let (c_pos, _, _) = self.camera.get_look_at();
      
      let light1_position     = Vector4::new(0.0, 0.0, 0.0, 10.0); // x y z, intensity
      let light1_colour       = Vector4::new(1.0, 0.0, 0.0, c_pos.x); // r, g, b, cam_x
      let light2_position     = Vector4::new(0.0, 0.0, 0.0, 10.0); // x2 y2 z2 intensity2
      let light2_colour       = Vector4::new(0.0, 1.0, 0.0, c_pos.y); // r2 g2 b2, cam_y
      let light3_position     = Vector4::new(0.0, 0.0, 0.0, 10.0);//x3 y3 z3, intensity3
      let light3_colour       = Vector4::new(0.0, 0.0, 1.0, c_pos.z); // r3, g3, b3, cam_z
      
      let sun_direction       = Vector4::new(0.0, 0.0, 0.0, 0.0);
      let sun_colour          = Vector4::new(0.0, 0.0, 0.0, 0.0);
      
      let push_constant_data = UniformData::new()
                                 .add_vector4(light1_position)
                                 .add_vector4(light1_colour)
                                 .add_vector4(light2_position)
                                 .add_vector4(light2_colour)
                                 .add_vector4(light3_position)
                                 .add_vector4(light3_colour)
                                 .add_vector4(sun_direction)
                                 .add_vector4(sun_colour);
    
      cmd = cmd.push_constants(Arc::clone(&device), &self.pipeline_deffered, ShaderStage::Vertex, push_constant_data);
      
      let index_count = 6;
      
      let cmd = cmd.draw_indexed(Arc::clone(&device), &self.vertex_buffer_deffered.internal_object(0),
                               &self.index_buffer_deffered.internal_object(0),
                               index_count, &self.pipeline_deffered,
                               vec!(*self.deffered_descriptor_set.set(current_buffer)),
                               Vec::with_capacity(0)
                      );
     
     cmd
  }
  
  pub fn destroy(&mut self, device: Arc<Device>) {
    self.index_buffer_deffered.destroy(Arc::clone(&device));
    self.vertex_buffer_deffered.destroy(Arc::clone(&device));
    
    for (_data, buffer) in &self.instanced_cpu_buffers {
      buffer.destroy(Arc::clone(&device));
    }
    
    for model in &self.models {
      model.destroy(Arc::clone(&device));
    }
    
    self.dummy_uniform_buffer.destroy(Arc::clone(&device));
    
    self.pipeline.destroy(Arc::clone(&device));
    self.double_pipeline.destroy(Arc::clone(&device));
    self.pipeline_deffered.destroy(Arc::clone(&device));
    self.instanced_pipeline.destroy(Arc::clone(&device));
    self.instanced_double_pipeline.destroy(Arc::clone(&device));
    
    for descriptor in &self.descriptor_sets {
      descriptor.destroy(Arc::clone(&device));
    }
    
    self.deffered_descriptor_set.destroy(Arc::clone(&device));
    
    self.vertex_shader.destroy(Arc::clone(&device));
    self.fragment_shader.destroy(Arc::clone(&device));
    self.vertex_shader_deffered.destroy(Arc::clone(&device));
    self.fragment_shader_deffered.destroy(Arc::clone(&device));
    self.vertex_shader_instanced.destroy(Arc::clone(&device));
    
    for framebuffer in &self.framebuffers {
     framebuffer.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_colour_images {
      images.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_albedo_colour_images {
      images.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_mro_images {
      images.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_emissive_images {
      images.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_normal_images {
      images.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_position_images {
      images.destroy(Arc::clone(&device)); 
    }
    
    for images in &self.framebuffer_msaa_images {
      images.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_depth_images {
      images.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_msaa_depth_images {
      images.destroy(Arc::clone(&device));
    }
    
    for images in &self.framebuffer_remaining_msaa_images {
      images.destroy(Arc::clone(&device));
    }
    
    self.renderpass.destroy(Arc::clone(&device));
  }
}
