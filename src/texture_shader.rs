use vk;

use crate::math;

use crate::vulkan::vkenums::{AttachmentLoadOp, AttachmentStoreOp, ImageLayout, ShaderStageFlagBits};

use crate::vulkan::{Instance, Device, RenderPass, Shader, Pipeline, PipelineBuilder, DescriptorSet, DescriptorSetBuilder, UpdateDescriptorSets, Image, AttachmentInfo, SubpassInfo, RenderPassBuilder, Sampler};
use crate::vulkan::buffer::{Buffer, BufferUsage, UniformBufferBuilder, UniformData, Framebuffer, CommandBufferBuilder};
use crate::vulkan::pool::{DescriptorPool, CommandPool};
use crate::CoreMaat;

use cgmath::{Vector2, Vector3, Vector4, Matrix4, ortho, SquareMatrix};

use std::mem;
use std::ptr;
use std::sync::Arc;
use std::collections::HashMap; 

// Simple offset_of macro akin to C++ offsetof
#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::uninitialized();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

#[derive(Clone)]
struct Vertex {
  pos: Vector2<f32>,
  uvs: Vector2<f32>,
}

impl Vertex {
  pub fn vertex_input_binding() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription {
      binding: 0,
      stride: (mem::size_of::<Vertex>()) as u32,
      inputRate: vk::VERTEX_INPUT_RATE_VERTEX,
    }
  }
  
  pub fn vertex_input_attributes() -> Vec<vk::VertexInputAttributeDescription> {
    let mut vertex_input_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> = Vec::with_capacity(2);
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::FORMAT_R32G32_SFLOAT,
        offset: offset_of!(Vertex, pos) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::FORMAT_R32G32_SFLOAT,
        offset: offset_of!(Vertex, uvs) as u32,
      }
    );
    
    vertex_input_attribute_descriptions
  }
}

pub struct TextureShader {
  renderpass: RenderPass,
  framebuffers: Vec<Framebuffer>,
  
  descriptor_sets: HashMap<String, DescriptorSet>,//Vec<DescriptorSet>,
  vertex_buffer: Buffer<Vertex>,
  index_buffer: Buffer<u32>,
  uniform_buffers: Vec<Buffer<f32>>,
  
  texture_pipeline: Pipeline,
  
  vertex_shader: Shader,
  fragment_shader: Shader,
  
  scale: f32,
}

impl TextureShader {
  pub fn new(instance: Arc<Instance>, device: Arc<Device>, current_extent: &vk::Extent2D, format: &vk::Format, sampler: &Sampler, image_views: &Vec<vk::ImageView>, texture_image: &Image, descriptor_set_pool: &DescriptorPool, command_pool: &CommandPool, dpi: f32, graphics_queue: &vk::Queue) -> TextureShader {
    let vertex_shader = Shader::new(Arc::clone(&device), include_bytes!("./shaders/texture/VkTextureVert.spv"));
    let fragment_shader = Shader::new(Arc::clone(&device), include_bytes!("./shaders/texture/VkTextureFrag.spv"));
    
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
    let subpass = SubpassInfo::new().add_colour_attachment(0);
    let render_pass = RenderPassBuilder::new()
                      .add_attachment(colour_attachment)
                      .add_subpass(subpass)
                      .build(Arc::clone(&device));
    
    let framebuffers = TextureShader::create_frame_buffers(Arc::clone(&device), &render_pass, current_extent, image_views);
    
    let mut descriptor_sets: HashMap<String, DescriptorSet> = HashMap::new();
    descriptor_sets.insert("".to_string(), DescriptorSetBuilder::new()
                           .vertex_uniform_buffer(0, 0)
                           .fragment_combined_image_sampler(0, 1)
                           .build(Arc::clone(&device), &descriptor_set_pool, 1));
    
    let push_constant_size = UniformData::new()
                               .add_matrix4(Matrix4::identity())
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .add_vector4(Vector4::new(0.0, 0.0, 0.0, 0.0))
                               .size();
    
    let pipeline = PipelineBuilder::new()
                  .vertex_shader(*vertex_shader.get_shader())
                  .fragment_shader(*fragment_shader.get_shader())
                  .push_constants(ShaderStageFlagBits::Vertex, push_constant_size as u32)
                  .render_pass(render_pass.clone())
                  .descriptor_set_layout(descriptor_sets.get(&"".to_string()).unwrap().layouts_clone())
                  .vertex_binding(vec!(Vertex::vertex_input_binding()))
                  .vertex_attributes(Vertex::vertex_input_attributes())
                  .topology_triangle_list()
                  .polygon_mode_fill()
                  .cull_mode_back()
                  .front_face_counter_clockwise()
                  .build(Arc::clone(&device));
    
    let vertex_buffer = TextureShader::create_vertex_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    let index_buffer = TextureShader::create_index_buffer(Arc::clone(&instance), Arc::clone(&device), &command_pool, graphics_queue);
    
    let mut uniform_buffers: Vec<Buffer<f32>> = Vec::new(); 
    let mut uniform_buffer_description = UniformBufferBuilder::new().add_matrix4().add_matrix4();
    
    uniform_buffers.push(TextureShader::create_uniform_buffer(Arc::clone(&instance), Arc::clone(&device), descriptor_sets.get("").unwrap(), image_views.len() as u32, uniform_buffer_description));
      
    TextureShader::update_uniform_buffers(Arc::clone(&instance), Arc::clone(&device), &mut uniform_buffers, &texture_image, sampler, descriptor_sets.get("").unwrap(), 1.0/dpi, current_extent.width as f32, current_extent.height as f32);
    
    TextureShader {
      renderpass: render_pass,
      framebuffers,
      
      descriptor_sets: descriptor_sets,
      vertex_buffer,
      index_buffer,
      uniform_buffers,
      
      texture_pipeline: pipeline,
      
      vertex_shader,
      fragment_shader,
      
      scale: 1.0/dpi,
    }
  }
  
  pub fn set_scale(&mut self, new_dpi: f32) {
    self.scale = (1.0/new_dpi);
  }
  
  pub fn create_projection(width: f32, height: f32) -> Matrix4<f32> {
    ortho(0.0, width, height, 0.0, -1.0, 1.0)
  }
  
  pub fn recreate(&mut self, instance: Arc<Instance>, device: Arc<Device>, image_views: &Vec<vk::ImageView>, new_extent: &vk::Extent2D, texture: &Image, sampler: &Sampler) {
    for i in 0..self.framebuffers.len() {
      self.framebuffers[i].destroy(Arc::clone(&device));
    }
    
    self.framebuffers.clear();
    
    for i in 0..image_views.len() {
      self.framebuffers.push(Framebuffer::new(Arc::clone(&device), &self.renderpass, &new_extent, &image_views[i]));
    }
    /*
    TextureShader::update_uniform_buffers(Arc::clone(&instance), Arc::clone(&device), &mut self.uniform_buffers, texture, sampler, &self.descriptor_sets.get(&"".to_string()).unwrap(), new_extent.width as f32, new_extent.height as f32);*/
    /*for (reference, descriptor) in &self.descriptor_sets {
      TextureShader::update_uniform_buffers(Arc::clone(&instance), Arc::clone(&device), &mut self.uniform_buffers, descriptor.texture, sampler, &self.descriptor_sets.get(&"".to_string()).unwrap(), new_extent.width as f32, new_extent.height as f32);
    }*/
  }
  
  pub fn add_texture(&mut self, instance: Arc<Instance>, device: Arc<Device>, descriptor_set_pool: &DescriptorPool, texture_reference: String, texture_image: &Image, sampler: &Sampler, current_extent: &vk::Extent2D) {
   if !self.descriptor_sets.contains_key(&texture_reference) {
      let descriptor = DescriptorSetBuilder::new()
                           .vertex_uniform_buffer(0, 0)
                           .fragment_combined_image_sampler(0, 1)
                           .build(Arc::clone(&device), &descriptor_set_pool, 1);
      self.descriptor_sets.insert(texture_reference.to_string(), descriptor);
      
      if let Some(descriptor_set) = self.descriptor_sets.get(&texture_reference) {
        let mut uniform_buffer_description = UniformBufferBuilder::new().add_matrix4().add_matrix4();
        
        self.uniform_buffers.push(TextureShader::create_uniform_buffer(Arc::clone(&instance), Arc::clone(&device), &descriptor_set, self.framebuffers.len() as u32, uniform_buffer_description));
        
        TextureShader::update_uniform_buffers(Arc::clone(&instance), Arc::clone(&device), &mut self.uniform_buffers, &texture_image, sampler, &descriptor_set, self.scale, current_extent.width as f32, current_extent.height as f32);
      }
    }
  }
  
  fn create_uniform_buffer(instance: Arc<Instance>, device: Arc<Device>, descriptor_set: &DescriptorSet, num_sets: u32, uniform_buffer: UniformBufferBuilder) -> Buffer<f32> {
    let mut buffer = uniform_buffer.build(Arc::clone(&instance), Arc::clone(&device), num_sets);
    
    buffer
  }
  
  fn create_index_buffer(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<u32> {
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
  
  fn create_vertex_buffer(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue) -> Buffer<Vertex> {
    let triangle = vec!(
      Vertex { pos: Vector2::new(0.5, 0.5), uvs: Vector2::new(1.0, 0.0) },
      Vertex { pos: Vector2::new(-0.5, 0.5), uvs: Vector2::new(0.0, 0.0) },
      Vertex { pos: Vector2::new(-0.5, -0.5), uvs: Vector2::new(0.0, 1.0) },
      Vertex { pos: Vector2::new(0.5, -0.5), uvs: Vector2::new(1.0, 1.0) },
    );
    
    let usage_src = BufferUsage::vertex_transfer_src_buffer();
    let usage_dst = BufferUsage::vertex_transfer_dst_buffer();
    
    let staging_buffer: Buffer<Vertex> = Buffer::cpu_buffer(Arc::clone(&instance), Arc::clone(&device), usage_src, 1, triangle.clone());
    let buffer: Buffer<Vertex> = Buffer::device_local_buffer(Arc::clone(&instance), Arc::clone(&device), usage_dst, 1, triangle);
    
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
  
  fn update_uniform_buffers(instance: Arc<Instance>, device: Arc<Device>, uniform_buffer: &mut Vec<Buffer<f32>>, texture: &Image, sampler: &Sampler, descriptor_sets: &DescriptorSet, scale: f32, width: f32, height: f32) {
    let data = UniformData::new()
                 .add_matrix4(TextureShader::create_projection(width, height))
                 .add_matrix4(Matrix4::from_scale(scale));
    
    let len = uniform_buffer.len()-1;
    UpdateDescriptorSets::new()
       .add_uniformbuffer(Arc::clone(&device), 0, 0, &mut uniform_buffer[len], data)
       .add_sampled_image(1, texture, ImageLayout::ShaderReadOnlyOptimal, &sampler)
       .finish_update(Arc::clone(&instance), Arc::clone(&device), &descriptor_sets);
  }
  
  pub fn begin_renderpass(&mut self, device: Arc<Device>, cmd: CommandBufferBuilder, clear_value: &Vec<vk::ClearValue>, window_size: &vk::Extent2D, current_buffer: usize) -> CommandBufferBuilder {
    cmd.begin_render_pass(Arc::clone(&device), &clear_value, &self.renderpass, &self.framebuffers[current_buffer].internal_object(), &window_size)
  }
  
  pub fn draw_texture(&mut self, instance: Arc<Instance>, device: Arc<Device>, cmd: CommandBufferBuilder, position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, sprite_details: Option<Vector3<i32>>, colour: Option<Vector4<f32>>, black_and_white: bool, use_texture: bool, texture_reference: String, texture_image: &Image, sampler: &Sampler, current_extent: &vk::Extent2D, descriptor_set_pool: &DescriptorPool) -> CommandBufferBuilder {
    let mut cmd = cmd;
    
    let descriptor: &DescriptorSet = self.descriptor_sets.get(&texture_reference).unwrap();
    
    let model = math::calculate_texture_model(Vector3::new(position.x/self.scale, position.y/self.scale, 0.0), scale/self.scale, -rotation -180.0);
    
    let has_texture  = {
      if use_texture {
        1.0
      } else {
        0.0
      }
    };
  
    let mut bw: f32 = 0.0;
    if black_and_white {
      bw = 1.0;
    }
    
    let sprite = {
      let mut tex_view = Vector4::new(0.0, 0.0, 1.0, 0.0);
      if let Some(details) = sprite_details {
        tex_view = Vector4::new(details.x as f32, details.y as f32, details.z as f32, 0.0);
      }
      tex_view
    };
    
    let draw_colour;
    if let Some(colour) = colour {
      draw_colour = colour;
    } else {
      draw_colour = Vector4::new(1.0, 1.0, 1.0, 1.0);
    }
    
    let texture_blackwhite = Vector4::new(has_texture, bw, 0.0, 0.0);
    
    let push_constant_data = UniformData::new()
                               .add_matrix4(model)
                               .add_vector4(draw_colour)
                               .add_vector4(sprite)
                               .add_vector4(texture_blackwhite);
    
    cmd = cmd.push_constants(Arc::clone(&device), &self.texture_pipeline, ShaderStageFlagBits::Vertex, push_constant_data);
    
    let index_count = 6;
    
    cmd.draw_indexed(Arc::clone(&device), &self.vertex_buffer.internal_object(0),
                             &self.index_buffer.internal_object(0),
                             index_count, &self.texture_pipeline,
                             vec!(&descriptor.set(0)))
  }
  
  pub fn destroy(&mut self, device: Arc<Device>) {
    for uniform in &self.uniform_buffers {
      uniform.destroy(Arc::clone(&device));
    }
    
    self.index_buffer.destroy(Arc::clone(&device));
    self.vertex_buffer.destroy(Arc::clone(&device));
    
    self.texture_pipeline.destroy(Arc::clone(&device));
    
    for (reference, descriptor_set) in &self.descriptor_sets {
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
