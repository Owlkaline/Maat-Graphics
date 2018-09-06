use vulkano::memory;
use vulkano::format;
use vulkano::pipeline;
use vulkano::framebuffer;
use vulkano::sync::NowFuture;
use vulkano::image as vkimage;
use vulkano::image::ImageUsage;
use vulkano::device::{Queue, Device};
use vulkano::buffer::{cpu_pool, BufferUsage, 
                      BufferAccess, CpuBufferPool, 
                      ImmutableBuffer, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, CommandBufferExecFuture};

use math;
use graphics::Vertex2d;
use drawcalls::DrawCall;
use vulkan::rawvk::{Model,DynamicModel, vs_texture, fs_texture, vs_text, fs_text};
use vulkan::renderpass::CustomRenderpass;

use cgmath::{ortho, Vector2, Vector4, Matrix4};

use std::sync::Arc;

pub fn create_2d_projection(width: f32, height: f32) -> Matrix4<f32> {
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

pub fn create_dynamic_custom_model(device: Arc<Device>, mut verts: Vec<Vertex2d>, indices: Vec<u32>) -> DynamicModel {
  for i in 0..verts.len() {
    verts[i].position[0] *= -1.0;
    verts[i].position[1] *= -1.0;
  }
  
  let vert =  CpuAccessibleBuffer::from_iter(device.clone(),
                                 BufferUsage::vertex_buffer(), 
                                 verts.iter().cloned())
                                 .expect("Vulkan failed to create custom vertex buffer");
  let idx = CpuAccessibleBuffer::from_iter(device,
                                 BufferUsage::index_buffer(), 
                                 indices.iter().cloned())
                                 .expect("Vulkan failed to create custom index buffer");
  
  DynamicModel {
    vertex_buffer: Some(vec!(vert)),
    index_buffer: Some(idx),
  }
}

pub fn create_static_custom_model(device: Arc<Device>, queue: Arc<Queue>, mut verts: Vec<Vertex2d>, indices: Vec<u32>) -> (Model, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
  for i in 0..verts.len() {
    verts[i].position[1] *= -1.0;
  }
  
  let vert =  CpuAccessibleBuffer::from_iter(device,
                                 BufferUsage::vertex_buffer(), 
                                 verts.iter().cloned())
                                 .expect("Vulkan failed to create custom vertex buffer");
  
  let (idx_buffer, future_idx) = ImmutableBuffer::from_iter(indices.iter().cloned(), 
                             BufferUsage::index_buffer(), 
                             queue)
                             .expect("failed to create immutable index buffer");
  
  let model = Model {
    vertex_buffer: Some(vec!(vert)),
    index_buffer: Some(idx_buffer),
  };
  
  (model, future_idx)
}

pub fn create_texture_subbuffer(draw: DrawCall, projection: Matrix4<f32>, uniform_buffer_texture: CpuBufferPool<vs_texture::ty::Data>) -> cpu_pool::CpuBufferPoolSubbuffer<vs_texture::ty::Data, Arc<memory::pool::StdMemoryPool>> {
  let model = math::calculate_texture_model(draw.position(), Vector2::new(draw.scale().x, draw.scale().y), -draw.rotation().x -180.0);
  
  let has_texture = {
    let mut value = 1.0;
    if let Some(texture_name) = draw.texture_name() {
      if texture_name == String::from("") {
        value = 0.0;
      }
    }
    value
  };
  
  let mut bw: f32 = 0.0;
  if draw.black_and_white_enabled() {
    bw = 1.0;
  }
  
  let uniform_data = vs_texture::ty::Data {
    projection: projection.into(),
    model: model.into(),
    colour: draw.colour().into(),
    has_texture_blackwhite: Vector4::new(has_texture, bw, 0.0, 0.0).into(),
  };
  
  uniform_buffer_texture.next(uniform_data).unwrap()
}

pub fn create_texture_attachments(device: Arc<Device>, dim: [u32; 2], samples: u32) -> (Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>) {
  
  let image_usage = ImageUsage {
    color_attachment: true,
    transfer_destination: true,
    sampled: true,
    .. ImageUsage::none()
  };
  
  let ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(device.clone(), dim, samples, format::Format::R16G16B16A16Unorm).unwrap();
  let fullcolour_attachment = vkimage::AttachmentImage::with_usage(device.clone(), dim, format::Format::R16G16B16A16Unorm, image_usage).unwrap();
  
  (ms_colour_attachment, fullcolour_attachment)
}

pub fn create_texturebuffer(device: Arc<Device>, dim: [u32; 2], samples: u32) -> CustomRenderpass {
  let texturebuffer_renderpass = Arc::new(single_pass_renderpass!(device.clone(),
      attachments: {
        multisample_colour: {
          load: Clear,
          store: DontCare,
          format: format::Format::R16G16B16A16Unorm,
          samples: samples,
        },
        resolve_fullcolour: {
          load: DontCare,
          store: Store,
          format: format::Format::R16G16B16A16Unorm,
          samples: 1,
        }
      },
      pass: {
        color: [multisample_colour],
        depth_stencil: {},
        resolve: [resolve_fullcolour],
      }
    ).unwrap());
  
  let vs_texture = vs_texture::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_texture = fs_texture::Shader::load(device.clone()).expect("failed to create shader module");
  let vs_text = vs_text::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_text = fs_text::Shader::load(device.clone()).expect("failed to create shader module");
  
  let texture_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_texture.main_entry_point(), ())
       // .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_texture.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(texturebuffer_renderpass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
  
  let text_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_text.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_text.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(texturebuffer_renderpass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
  
  let (ms_colour_attachment, fullcolour_attachment) = create_texture_attachments(device.clone(), dim, samples);
  
  let textureframebuffer = Arc::new({
      framebuffer::Framebuffer::start(texturebuffer_renderpass.clone())
          .add(ms_colour_attachment.clone()).unwrap()
          .add(fullcolour_attachment.clone()).unwrap()
          .build().unwrap()
  });
  
  let mut texture_customrenderpass = CustomRenderpass::new(vec!(ms_colour_attachment, fullcolour_attachment));
  
  texture_customrenderpass.set_renderpass(texturebuffer_renderpass);
  texture_customrenderpass.set_pipelines(vec!(texture_pipeline, text_pipeline));
  texture_customrenderpass.set_framebuffer(textureframebuffer);
  
  texture_customrenderpass
}

pub fn recreate_texturebuffer(renderpass: &mut CustomRenderpass, device: Arc<Device>, dim: [u32; 2], samples: u32) {
  let (ms_colour_attachment, fullcolour_attachment) = create_texture_attachments(device.clone(), dim, samples);
  
  let textureframebuffer = Arc::new({
      framebuffer::Framebuffer::start(renderpass.renderpass().clone())
          .add(ms_colour_attachment.clone()).unwrap()
          .add(fullcolour_attachment.clone()).unwrap()
          .build().unwrap()
  });
  
  renderpass.update_attachments(vec!(ms_colour_attachment, fullcolour_attachment));
  renderpass.set_framebuffer(textureframebuffer);
}

pub fn create_finalbuffer(device: Arc<Device>, dimensions: [u32; 2]) {
  
}
