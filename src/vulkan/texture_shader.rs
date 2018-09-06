use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferAccess;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;

use vulkano::sync::NowFuture;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
 use vulkano::command_buffer::CommandBufferExecFuture;

use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;

use vulkano::device::Queue;
use vulkano::device::Device;

use vulkano::image as vkimage;
use vulkano::image::ImageUsage;

use vulkano::format;
use vulkano::format::ClearValue;

use vulkano::pipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

use graphics::Vertex2d;

use cgmath::Matrix4;
use cgmath::ortho;

use std::sync::Arc;

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
  
  texture_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
  texture_uniformbuffer: CpuBufferPool<vs_texture::ty::Data>,
  
  text_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
  text_uniformbuffer: CpuBufferPool<vs_text::ty::Data>,
  
  attachment_image: Arc<vkimage::AttachmentImage>,
}

impl TextureShader {
  pub fn create(device: Arc<Device>, dim: [u32; 2], samples: u32)-> TextureShader {
    let text_uniform = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());
    let texture_uniform = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());
    
    let vs_texture = vs_texture::Shader::load(device.clone()).expect("failed to create shader module");
    let fs_texture = fs_texture::Shader::load(device.clone()).expect("failed to create shader module");
    let vs_text = vs_text::Shader::load(device.clone()).expect("failed to create shader module");
    let fs_text = fs_text::Shader::load(device.clone()).expect("failed to create shader module");
    
    let renderpass = Arc::new(single_pass_renderpass!(device.clone(),
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
        .render_pass(framebuffer::Subpass::from(renderpass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
    
    let text_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_text.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_text.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(renderpass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
    
    let (ms_colour_attachment, fullcolour_attachment) = TextureShader::create_texture_attachments(device.clone(), dim, samples);
    
    let framebuffer = Arc::new({
      framebuffer::Framebuffer::start(renderpass.clone())
          .add(ms_colour_attachment.clone()).unwrap()
          .add(fullcolour_attachment.clone()).unwrap()
          .build().unwrap()
    });
    
    TextureShader {
      renderpass: renderpass,
      framebuffer: framebuffer,
      
      texture_pipeline: texture_pipeline,
      texture_uniformbuffer: texture_uniform,
      
      text_pipeline: text_pipeline,
      text_uniformbuffer: text_uniform,
      
      attachment_image: fullcolour_attachment,
    }
  }
  
  pub fn create_projection(width: f32, height: f32) -> Matrix4<f32> {
    ortho(0.0, width, height, 0.0, -1.0, 1.0)
  }
  
  pub fn create_vertex(device: Arc<Device>) -> Arc<BufferAccess + Send + Sync> {
    let square = {
      [
        Vertex2d { position: [  0.5 ,   0.5 ], uv: [1.0, 0.0] },
        Vertex2d { position: [ -0.5,    0.5 ], uv: [0.0, 0.0] },
        Vertex2d { position: [ -0.5,   -0.5 ], uv: [0.0, 1.0] },
        Vertex2d { position: [  0.5 ,  -0.5 ], uv: [1.0, 1.0] },
      ]
    };
    
    CpuAccessibleBuffer::from_iter(device,
                                   BufferUsage::vertex_buffer(), 
                                   square.iter().cloned())
                                   .expect("failed to create vertex buffer")
  }

  pub fn create_index(queue: Arc<Queue>) -> (Arc<ImmutableBuffer<[u32]>>,
                                    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
    ImmutableBuffer::from_iter(indices.iter().cloned(), 
                               BufferUsage::index_buffer(),
                               queue)
                               .expect("failed to create immutable index buffer")
  }
  
  pub fn recreate_framebuffer(&mut self, device: Arc<Device>, dim: [u32; 2], samples: u32) {
    let (ms_colour_attachment, fullcolour_attachment) = TextureShader::create_texture_attachments(device.clone(), dim, samples);
  
    let framebuffer = Arc::new({
        framebuffer::Framebuffer::start(self.renderpass.clone())
            .add(ms_colour_attachment.clone()).unwrap()
            .add(fullcolour_attachment.clone()).unwrap()
            .build().unwrap()
    });
    
    self.framebuffer = framebuffer;
  }
  
  pub fn begin_renderpass(&mut self, cb: AutoCommandBufferBuilder, secondary: bool, clear_value: ClearValue) -> AutoCommandBufferBuilder {
    cb.begin_render_pass(self.framebuffer.clone(), secondary, vec![clear_value, ClearValue::None]).unwrap()
  }
  
  pub fn end_renderpass(&mut self, cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
    cb.end_render_pass().unwrap()
  }
  
  pub fn get_texture_attachment(&mut self) -> Arc<vkimage::AttachmentImage> {
    self.attachment_image.clone()
  }
  
  fn create_texture_attachments(device: Arc<Device>, dim: [u32; 2], samples: u32) -> (Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>) {
    
    let image_usage = ImageUsage {
      color_attachment: true,
      transfer_destination: true,
      sampled: true,
      .. ImageUsage::none()
    };
    
    let ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(device.clone(), dim, samples, format::Format::R8G8B8A8Unorm).unwrap();
    //let fullcolour_attachment = vkimage::StorageImage::new(device.clone(), dim, format::R8G8B8A8Unorm, Some(queue.family())).unwrap();
    let fullcolour_attachment = vkimage::AttachmentImage::with_usage(device.clone(), dim, format::Format::R8G8B8A8Unorm, image_usage).unwrap();
    
    (ms_colour_attachment, fullcolour_attachment)
  }
}
