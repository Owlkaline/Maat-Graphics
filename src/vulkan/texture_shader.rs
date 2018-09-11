use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferAccess;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;

use vulkano::instance::QueueFamily;

use vulkano::sampler;
use vulkano::sync::NowFuture;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

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
use drawcalls::DrawCall;
use font::GenericFont;

use cgmath::Vector2;
use cgmath::Vector4;
use cgmath::Matrix4;
use cgmath::ortho;

use std::sync::Arc;
use std::collections::HashMap;

mod vs_texture {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkTexture.vert"]
  struct Dummy;
}

mod fs_texture {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkTexture.frag"]
  struct Dummy;
}

mod vs_text {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkText.vert"]
  struct Dummy;
}

mod fs_text {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkText.frag"]
  struct Dummy;
}

pub struct TextureShader {
  renderpass: Arc<RenderPassAbstract + Send + Sync>,
  framebuffer: Arc<framebuffer::FramebufferAbstract + Send + Sync + Send + Sync>,
  
  vertex_buffer: Arc<BufferAccess + Send + Sync>,
  index_buffer: Arc<ImmutableBuffer<[u32]>>,
  
  texture_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
  texture_uniformbuffer: CpuBufferPool<vs_texture::ty::Data>,
  
  text_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
  text_uniformbuffer: CpuBufferPool<vs_text::ty::Data>,
  
  attachment_image: Arc<vkimage::AttachmentImage>,
  sampler: Arc<sampler::Sampler>,
  
  fonts: HashMap<String, GenericFont>,
}

impl TextureShader {
  pub fn create(device: Arc<Device>, queue: Arc<Queue>, dim: [u32; 2], samples: u32)-> (TextureShader, Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>) {
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
    
    let texture_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_texture.main_entry_point(), ())
       // .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_texture.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(Arc::clone(&renderpass), 0).unwrap())
        .build(Arc::clone(&device))
        .unwrap());
    
    let text_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_text.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_text.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(Arc::clone(&renderpass), 0).unwrap())
        .build(Arc::clone(&device))
        .unwrap());
    
    let (ms_colour_attachment, fullcolour_attachment) = TextureShader::create_texture_attachments(Arc::clone(&device), dim, samples);
    
    let (vertex_buffer, future_vtx) = TextureShader::create_vertex(Arc::clone(&device), Arc::clone(&queue));
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
    
    (
      TextureShader {
        renderpass: renderpass,
        framebuffer: framebuffer,
        
        vertex_buffer: vertex_buffer,
        index_buffer: idx_buffer,
        
        texture_pipeline: texture_pipeline,
        texture_uniformbuffer: texture_uniform,
        
        text_pipeline: text_pipeline,
        text_uniformbuffer: text_uniform,
        
        attachment_image: fullcolour_attachment,
        
        sampler: sampler,
        fonts: HashMap::new(),
      },
      vec!(future_idx, future_vtx)
    )
  }
  
  pub fn create_projection(width: f32, height: f32) -> Matrix4<f32> {
    ortho(0.0, width, height, 0.0, -1.0, 1.0)
  }
  
  pub fn create_vertex(device: Arc<Device>, queue: Arc<Queue>) -> (Arc<ImmutableBuffer<[Vertex2d]>>,
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
  
  pub fn recreate_framebuffer(&mut self, device: Arc<Device>, queue: Arc<Queue>, dim: [u32; 2], samples: u32) {
    let (ms_colour_attachment, fullcolour_attachment) = TextureShader::create_texture_attachments(Arc::clone(&device), dim, samples);
  
    let framebuffer = Arc::new({
        framebuffer::Framebuffer::start(Arc::clone(&self.renderpass))
            .add(Arc::clone(&ms_colour_attachment)).unwrap()
            .add(Arc::clone(&fullcolour_attachment)).unwrap()
            .build().unwrap()
    });
    
    self.attachment_image = fullcolour_attachment;
    
    self.framebuffer = framebuffer;
  }
  
  pub fn create_secondary_renderpass(&mut self, device: Arc<Device>, family: QueueFamily<'_>) -> AutoCommandBufferBuilder {
    let subpass = framebuffer::Subpass::from(Arc::clone(&self.renderpass), 0).unwrap();
    AutoCommandBufferBuilder::secondary_graphics_one_time_submit(Arc::clone(&device), family, subpass).unwrap()
  }
  
  pub fn begin_renderpass(&mut self, cb: AutoCommandBufferBuilder, secondary: bool, clear_value: ClearValue) -> AutoCommandBufferBuilder {
    cb.begin_render_pass(Arc::clone(&self.framebuffer), secondary, vec![clear_value, ClearValue::None]).unwrap()
  }
  
  pub fn draw_texture(&mut self, cb: AutoCommandBufferBuilder, dynamic_state: &DynamicState, texture_projection: Matrix4<f32>, draw: DrawCall, use_texture: bool, texture_image: Arc<ImmutableImage<format::R8G8B8A8Unorm>>, use_custom_buffer: bool, custom_buffer: Option<(Arc<BufferAccess + Send + Sync>, Arc<ImmutableBuffer<[u32]>>)>) -> AutoCommandBufferBuilder {
    let model = math::calculate_texture_model(draw.position(), Vector2::new(draw.scale().x, draw.scale().y), -draw.rotation().x -180.0);
    
    let has_texture  = {
      if use_texture {
        1.0
      } else {
        0.0
      }
    };
  
    let mut bw: f32 = 0.0;
    if draw.black_and_white_enabled() {
      bw = 1.0;
    }
    
    let uniform_data = vs_texture::ty::Data {
      projection: texture_projection.into(),
      model: model.into(),
      colour: draw.colour().into(),
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
    
    let uniform_subbuffer = self.texture_uniformbuffer.next(uniform_data).unwrap();
    
    let descriptor_set = Arc::new(PersistentDescriptorSet::start(Arc::clone(&pipeline), 0)
                                .add_sampled_image(Arc::clone(&texture_image), Arc::clone(&self.sampler)).unwrap()
                                .add_buffer(uniform_subbuffer.clone()).unwrap().build().unwrap());
    
    cb.draw_indexed(pipeline, dynamic_state, vec!(vertex), index, descriptor_set, ()).unwrap()
  }
  
  pub fn draw_text(&mut self, cb: AutoCommandBufferBuilder, dynamic_state: &DynamicState, texture_projection: Matrix4<f32>, draw: DrawCall, font_info: (GenericFont, Arc<ImmutableImage<format::R8G8B8A8Unorm>>)) -> AutoCommandBufferBuilder {
    let mut cb = cb;
    let (font, texture) = font_info;
    let wrapped_draw = drawcalls::setup_correct_wrapping(draw.clone(), font.clone());
    let size = draw.scale().x;
    
    let vertex_buffer = self.vertex_buffer.clone();
    let index_buffer = self.index_buffer.clone();
    
    for letter in wrapped_draw {
      let char_letter = {
        letter.display_text().unwrap().as_bytes()[0] 
      };
      
      if let Some(font_name) = draw.font_name() {
        let c = font.get_character(char_letter as i32);
        
        let model = drawcalls::calculate_text_model(letter.position(), size, &c.clone(), char_letter);
        let letter_uv = drawcalls::calculate_text_uv(&c.clone());
        let colour = letter.colour();
        let outline = letter.text_outline_colour();
        let edge_width = letter.text_edge_width(); 
        
        let uniform_buffer_text_subbuffer = {
          let uniform_data = vs_text::ty::Data {
            outlineColour: outline.into(),
            colour: colour.into(),
            edge_width: edge_width.into(),
            letter_uv: letter_uv.into(),
            model: model.into(),
            projection: texture_projection.into(),
          };
          self.text_uniformbuffer.next(uniform_data).unwrap()
        };
        
        let uniform_set = Arc::new(PersistentDescriptorSet::start(Arc::clone(&self.text_pipeline), 0)
                                   .add_sampled_image(Arc::clone(&texture), Arc::clone(&self.sampler)).unwrap()
                                   .add_buffer(uniform_buffer_text_subbuffer.clone()).unwrap()
                                   .build().unwrap());
        
        cb = cb.draw_indexed(Arc::clone(&self.text_pipeline),
                                dynamic_state,
                                vec!(Arc::clone(&vertex_buffer)),
                                Arc::clone(&index_buffer),
                                uniform_set, ()).unwrap();
      }
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
