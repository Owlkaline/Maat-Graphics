use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferAccess;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;

use vulkano::instance::QueueFamily;

use vulkano::sampler;
use vulkano::sync::NowFuture;
use vulkano::memory::pool::StdMemoryPool;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;

use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferExecFuture;

use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;

use vulkano::device::Queue;
use vulkano::device::Device;

use vulkano::image as vkimage;
use vulkano::image::ImageUsage;
use vulkano::image::ImmutableImage;

use vulkano::format;
use vulkano::format::ClearValue;

use vulkano::pipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

use math;
use graphics::Vertex2d;
use drawcalls;
use font::GenericFont;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;
use cgmath::ortho;

use std::sync::Arc;

/*
mod vs_texture {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkTexture.vert"]
  struct _Dummy;
}

mod fs_texture {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkTexture.frag"]
  struct _Dummy;
}

mod vs_text {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkText.vert"]
  struct _Dummy;
}

mod fs_text {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkText.frag"]
  struct _Dummy;
}*/

mod vs_texture {
  vulkano_shaders::shader! {
    ty: "vertex",
    path: "src/shaders/glsl/VkTexture.vert"
  }
}

mod fs_texture {
  vulkano_shaders::shader! {
    ty: "fragment",
    path: "src/shaders/glsl/VkTexture.frag"
  }
}

mod vs_text {
  vulkano_shaders::shader! {
    ty: "vertex",
    path: "src/shaders/glsl/VkText.vert"
  }
}

mod fs_text {
  vulkano_shaders::shader! {
    ty: "fragment",
    path: "src/shaders/glsl/VkText.frag"
  }
}

pub struct TextureShader {
  renderpass: Arc<RenderPassAbstract + Send + Sync>,
  framebuffer: Arc<framebuffer::FramebufferAbstract + Send + Sync + Send + Sync>,
  
  vertex_buffer: Arc<BufferAccess + Send + Sync>,
  index_buffer: Arc<ImmutableBuffer<[u32]>>,
  
  texture_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
  texture_uniformbuffer: CpuBufferPool<vs_texture::ty::Data>,
  texture_subbuffer: CpuBufferPoolSubbuffer<vs_texture::ty::Data, Arc<StdMemoryPool>>,
  texture_descriptor_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>, 
  
  text_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
  text_uniformbuffer: CpuBufferPool<vs_text::ty::Data>,
  text_subbuffer: CpuBufferPoolSubbuffer<vs_text::ty::Data, Arc<StdMemoryPool>>,
  text_descriptor_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  
  attachment_image: Arc<vkimage::AttachmentImage>,
  sampler: Arc<sampler::Sampler>,
  
  scale_matrix: Matrix4<f32>,
}

impl TextureShader {
  pub fn create(device: Arc<Device>, queue: Arc<Queue>, dim: [u32; 2], samples: u32, texture_projection: Matrix4<f32>)-> (TextureShader, Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>) {
    let text_uniform = CpuBufferPool::uniform_buffer(Arc::clone(&device));
    let texture_uniform = CpuBufferPool::uniform_buffer(Arc::clone(&device));
    
    let vs_texture = vs_texture::Shader::load(Arc::clone(&device)).expect("failed to create shader module");
    let fs_texture = fs_texture::Shader::load(Arc::clone(&device)).expect("failed to create shader module");
    let vs_text = vs_text::Shader::load(Arc::clone(&device)).expect("failed to create shader module");
    let fs_text = fs_text::Shader::load(Arc::clone(&device)).expect("failed to create shader module");
    
    let renderpass = Arc::new(single_pass_renderpass!(Arc::clone(&device),
      attachments: {
        multisample_colour: {
          load: Clear,
          store: DontCare,
          format: format::Format::R8G8B8A8Unorm,
          samples: samples,
        },
        resolve_fullcolour: {
          load: DontCare,
          store: Store,
          format: format::Format::R8G8B8A8Unorm,
          samples: 1,
        }
      },
      pass: {
        color: [multisample_colour],
        depth_stencil: {},
        resolve: [resolve_fullcolour],
      }
    ).unwrap());
    
    let texture_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_texture.main_entry_point(), ())
       // .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        //.viewports_scissors_dynamic(1)
        .fragment_shader(fs_texture.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(Arc::clone(&renderpass), 0).unwrap())
        .build(Arc::clone(&device))
        .unwrap());
    
    let text_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_text.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_text.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(Arc::clone(&renderpass), 0).unwrap())
        .build(Arc::clone(&device))
        .unwrap());
    
    let (ms_colour_attachment, fullcolour_attachment) = TextureShader::create_texture_attachments(Arc::clone(&device), dim, samples);
    
    let (vertex_buffer, future_vtx) = TextureShader::create_vertex(Arc::clone(&queue));
    let (idx_buffer, future_idx) = TextureShader::create_index(queue);
    
    let framebuffer = Arc::new({
      framebuffer::Framebuffer::start(Arc::clone(&renderpass))
          .add(Arc::clone(&ms_colour_attachment)).unwrap()
          .add(Arc::clone(&fullcolour_attachment)).unwrap()
          .build().unwrap()
    });
    
    let sampler = sampler::Sampler::new(Arc::clone(&device), sampler::Filter::Linear,
                                                   sampler::Filter::Linear, 
                                                   sampler::MipmapMode::Nearest,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   0.0, 1.0, 0.0, 0.0).unwrap();
    let scale_matrix = Matrix4::from_scale(0.5);
    let uniform_data = vs_texture::ty::Data {
      projection: texture_projection.into(),
      scale: scale_matrix.into(),
    };
    let texture_subbuffer = texture_uniform.next(uniform_data).unwrap();
    let uniform_data = vs_text::ty::Data {
      projection: texture_projection.into(),
      scale: scale_matrix.into(),
    };
    let text_subbuffer = text_uniform.next(uniform_data).unwrap();
    
    let texture_descriptor_set = FixedSizeDescriptorSetsPool::new(Arc::clone(&texture_pipeline), 0);
    let text_descriptor_set = FixedSizeDescriptorSetsPool::new(Arc::clone(&text_pipeline), 0);
    
    (
      TextureShader {
        renderpass: renderpass,
        framebuffer: framebuffer,
        
        vertex_buffer: vertex_buffer,
        index_buffer: idx_buffer,
        
        texture_pipeline: texture_pipeline,
        texture_uniformbuffer: texture_uniform,
        texture_subbuffer: texture_subbuffer,
        texture_descriptor_pool: texture_descriptor_set,
        
        text_pipeline: text_pipeline,
        text_uniformbuffer: text_uniform,
        text_subbuffer: text_subbuffer,
        text_descriptor_pool: text_descriptor_set,
        
        attachment_image: fullcolour_attachment,
        
        sampler: sampler,
        scale_matrix: scale_matrix,
      },
      vec!(future_idx, future_vtx)
    )
  }
  
  pub fn create_projection(width: f32, height: f32) -> Matrix4<f32> {
    ortho(0.0, width, height, 0.0, -1.0, 1.0)
  }
  
  pub fn create_vertex(queue: Arc<Queue>) -> (Arc<ImmutableBuffer<[Vertex2d]>>,
                                    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    let square = {
      [
        Vertex2d { position: [  0.5 ,   0.5 ], uv: [1.0, 0.0] },
        Vertex2d { position: [ -0.5,    0.5 ], uv: [0.0, 0.0] },
        Vertex2d { position: [ -0.5,   -0.5 ], uv: [0.0, 1.0] },
        Vertex2d { position: [  0.5 ,  -0.5 ], uv: [1.0, 1.0] },
      ]
    };
    
    ImmutableBuffer::from_iter(square.iter().cloned(),
                               BufferUsage::vertex_buffer(),
                               queue)
                               .expect("failed to create immutable vertex buffer")
  }

  pub fn create_index(queue: Arc<Queue>) -> (Arc<ImmutableBuffer<[u32]>>,
                                    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
    ImmutableBuffer::from_iter(indices.iter().cloned(), 
                               BufferUsage::index_buffer(),
                               queue)
                               .expect("failed to create immutable index buffer")
  }
  
  pub fn set_scale(&mut self, scale: f32, texture_projection: Matrix4<f32>) {
    self.scale_matrix = Matrix4::from_scale(scale);
    self.update_uniform_buffers(texture_projection);
  }
  
  fn update_uniform_buffers(&mut self, texture_projection: Matrix4<f32>) {
    let uniform_data = vs_texture::ty::Data {
      projection: texture_projection.into(),
      scale: self.scale_matrix.into(),
    };
    
    self.texture_subbuffer = self.texture_uniformbuffer.next(uniform_data).unwrap();
    
    let uniform_data = vs_text::ty::Data {
      projection: texture_projection.into(),
      scale: self.scale_matrix.into(),
    };
    
    self.text_subbuffer = self.text_uniformbuffer.next(uniform_data).unwrap();
  }
  
  pub fn recreate_framebuffer(&mut self, device: Arc<Device>, dim: [u32; 2], samples: u32, texture_projection: Matrix4<f32>) {
    let (ms_colour_attachment, fullcolour_attachment) = TextureShader::create_texture_attachments(Arc::clone(&device), dim, samples);
  
    let framebuffer = Arc::new({
        framebuffer::Framebuffer::start(Arc::clone(&self.renderpass))
            .add(Arc::clone(&ms_colour_attachment)).unwrap()
            .add(Arc::clone(&fullcolour_attachment)).unwrap()
            .build().unwrap()
    });
    
    self.attachment_image = fullcolour_attachment;
    
    self.framebuffer = framebuffer;
    
    self.update_uniform_buffers(texture_projection);
  }
  
  pub fn create_secondary_renderpass(&mut self, device: Arc<Device>, family: QueueFamily<'_>) -> AutoCommandBufferBuilder {
    let subpass = framebuffer::Subpass::from(Arc::clone(&self.renderpass), 0).unwrap();
    AutoCommandBufferBuilder::secondary_graphics_one_time_submit(Arc::clone(&device), family, subpass).unwrap()
  }
  
  pub fn begin_renderpass(&mut self, cb: AutoCommandBufferBuilder, secondary: bool, clear_value: ClearValue) -> AutoCommandBufferBuilder {
    cb.begin_render_pass(Arc::clone(&self.framebuffer), secondary, vec![clear_value, ClearValue::None]).unwrap()
  }
  
  pub fn draw_texture(&mut self, cb: AutoCommandBufferBuilder, dynamic_state: &DynamicState, position: Vector2<f32>, scale: Vector2<f32>, rotation: f32, sprite_details: Option<Vector3<i32>>, colour: Option<Vector4<f32>>, black_and_white: bool, use_texture: bool, texture_image: Arc<ImmutableImage<format::R8G8B8A8Unorm>>, use_custom_buffer: bool, custom_buffer: Option<(Arc<BufferAccess + Send + Sync>, Arc<ImmutableBuffer<[u32]>>)>) -> AutoCommandBufferBuilder {
    let model = math::calculate_texture_model(Vector3::new(position.x , position.y, 0.0), scale, -rotation -180.0);
    
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
    
    let push_constants = vs_texture::ty::PushConstants {
      model: model.into(),
      colour: draw_colour.into(),
      sprite_sheet: sprite.into(),
      has_texture_blackwhite: Vector4::new(has_texture, bw, 0.0, 0.0).into(),
    };
    
    let pipeline = Arc::clone(&self.texture_pipeline);
    let (vertex, index) = {
      let mut vertex = Arc::clone(&self.vertex_buffer);
      let mut index = Arc::clone(&self.index_buffer);
       if use_custom_buffer {
         if let Some((vtx, idx)) = custom_buffer {
           vertex = vtx;
           index = idx;
         }
       }
      
      (vertex, index)
    };
    
    let descriptor_set = self.texture_descriptor_pool.next()
                             .add_sampled_image(Arc::clone(&texture_image), Arc::clone(&self.sampler)).unwrap()
                             .add_buffer(self.texture_subbuffer.clone()).unwrap()
                             .build().unwrap();
    
    cb.draw_indexed(pipeline, dynamic_state, vec!(vertex), index, descriptor_set, push_constants).unwrap()
  }
  
  pub fn draw_text(&mut self, cb: AutoCommandBufferBuilder, dynamic_state: &DynamicState, display_text: String, font: String, position: Vector2<f32>, scale: Vector2<f32>, colour: Vector4<f32>, outline_colour: Vector3<f32>, edge_width: Vector4<f32>, wrap_length: u32, centered: bool, font_info: (GenericFont, Arc<ImmutableImage<format::R8G8B8A8Unorm>>)) -> AutoCommandBufferBuilder {
    let mut cb = cb;
    let scale = scale *2.0;
    let (fonts, texture) = font_info;
    let wrapped_draw = drawcalls::setup_correct_wrapping(display_text.clone(), font, position, scale, colour, outline_colour, edge_width, wrap_length, centered, fonts.clone());
    let size = scale.x;
    
    let vertex_buffer = self.vertex_buffer.clone();
    let index_buffer = self.index_buffer.clone();
    
    for letter in wrapped_draw {
      let (_font, display_text, position, _scale, colour, outline_colour, edge_width, _wrapped, _wrap_length, _centered) = letter.draw_font_details().unwrap();
      let char_letter = {
        display_text.as_bytes()[0] 
      };
      
        let c = fonts.get_character(char_letter as i32);
        
        let model = drawcalls::calculate_text_model(Vector3::new(position.x, position.y, 0.0), size, &c.clone(), char_letter);
        let letter_uv = drawcalls::calculate_text_uv(&c.clone());
        let colour = colour;
        let outline = Vector4::new(outline_colour.x, outline_colour.y, outline_colour.z, 0.0);
        let edge_width = edge_width; 
        
        let push_constants = vs_text::ty::PushConstants {
          model: model.into(),
          letter_uv: letter_uv.into(),
          edge_width: edge_width.into(),
          colour: colour.into(),
          outlineColour: outline.into(),
        };
        
        let uniform_set = self.text_descriptor_pool.next()
                                   .add_sampled_image(Arc::clone(&texture), Arc::clone(&self.sampler)).unwrap()
                                   .add_buffer(self.text_subbuffer.clone()).unwrap()
                                   .build().unwrap();
        
        cb = cb.draw_indexed(Arc::clone(&self.text_pipeline),
                                dynamic_state,
                                vec!(Arc::clone(&vertex_buffer)),
                                Arc::clone(&index_buffer),
                                uniform_set,
                                push_constants).unwrap();
    }
    
    cb
  }
  
  pub fn end_renderpass(&mut self, cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
    cb.end_render_pass().unwrap()
  }
  
  pub fn get_texture_attachment(&mut self) -> Arc<vkimage::AttachmentImage> {
    Arc::clone(&self.attachment_image)
  }
  
  fn create_texture_attachments(device: Arc<Device>, dim: [u32; 2], samples: u32) -> (Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>) {
    
    let image_usage = ImageUsage {
      //color_attachment: true,
      transfer_destination: true,
      sampled: true,
      storage: true,
      .. ImageUsage::none()
    };
    
    let ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(Arc::clone(&device), dim, samples, format::Format::R8G8B8A8Unorm).unwrap();
    let fullcolour_attachment = vkimage::AttachmentImage::with_usage(Arc::clone(&device), dim, format::Format::R8G8B8A8Unorm, image_usage).unwrap();
    
    (ms_colour_attachment, fullcolour_attachment)
  }
}
