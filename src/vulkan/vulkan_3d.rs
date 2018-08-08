use vulkano::memory;
use vulkano::format;
use vulkano::sampler;
use vulkano::pipeline;
use vulkano::framebuffer;
use vulkano::sync::NowFuture;
use vulkano::image as vkimage;
use vulkano::image::ImageUsage;
use vulkano::image::ImmutableImage;
use vulkano::device::{Device, Queue};
use vulkano::buffer::{CpuBufferPool, cpu_pool,
                      BufferUsage, BufferAccess,
                      ImmutableBuffer, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBuffer, CommandBufferExecFuture};

use image;

use graphics::Vertex2d;
use graphics::Vertex3d;
use drawcalls::DrawCall;

use vulkan::rawvk::{vs_3d, vs_gbuffer_3d, fs_gbuffer_3d, vs_plain, fs_lights, fs_post_bloom, vs_shadow, fs_shadow};
use vulkan::renderpass::CustomRenderpass;

use gltf_interpreter::Sampler;
use gltf::texture::MagFilter;
use gltf::texture::MinFilter;
use gltf::texture::WrappingMode;

use cgmath::{Deg, perspective, Vector4, Matrix4};

use std::sync::Arc;
use std::iter;
use std::slice;

pub fn create_3d_projection(width: f32, height: f32) -> Matrix4<f32> {
  perspective(Deg(45.0), { width as f32 / height as f32 }, 0.1, 100.0)
}

pub fn create_subpass_vertex(device: Arc<Device>) -> Arc<BufferAccess + Send + Sync> {
  let square = {
    [
      Vertex2d { position: [  1.0 ,   1.0 ], uv: [1.0, 0.0] },
      Vertex2d { position: [ -1.0,    1.0 ], uv: [0.0, 0.0] },
      Vertex2d { position: [ -1.0,   -1.0 ], uv: [0.0, 1.0] },
      Vertex2d { position: [ -1.0,   -1.0 ], uv: [0.0, 1.0] },
      Vertex2d { position: [  1.0 ,  -1.0 ], uv: [1.0, 1.0] },
      Vertex2d { position: [  1.0 ,   1.0 ], uv: [1.0, 0.0] },
    ]
  };
  
  CpuAccessibleBuffer::from_iter(device,
                                 BufferUsage::vertex_buffer(), 
                                 square.iter().cloned())
                                 .expect("failed to create vertex buffer")
}

pub fn create_vertex(device: Arc<Device>, verticies: iter::Cloned<slice::Iter<Vertex3d>>) -> Arc<BufferAccess + Send + Sync> {
    CpuAccessibleBuffer::from_iter(device, 
                                   BufferUsage::vertex_buffer(), 
                                   verticies)
                                   .expect("failed to create vertex buffer")
}

pub fn create_index(queue: Arc<Queue>, indices: iter::Cloned<slice::Iter<u32>>) -> (Arc<ImmutableBuffer<[u32]>>,
                                                                         CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    ImmutableBuffer::from_iter(indices, BufferUsage::index_buffer(), 
                               queue)
                               .expect("failed to create immutable teapot index buffer")
}

pub fn create_depth_buffer(device: Arc<Device>, dimensions: [u32; 2]) -> Option<Arc<vkimage::AttachmentImage<format::D16Unorm>>> {
  Some(vkimage::attachment::AttachmentImage::transient(
                              device,
                              dimensions,
                              format::D16Unorm)
                              .unwrap())
}

pub fn create_sampler_from_gltfsampler(device: Arc<Device>, sampler: Sampler) -> Arc<sampler::Sampler> {
  let mag_filter = match sampler.mag_filter {
    MagFilter::Linear => sampler::Filter::Linear,
    MagFilter::Nearest => sampler::Filter::Nearest,
  };
  
  let (min_filter, mipmap_mode) = match sampler.min_filter {
    MinFilter::Linear => (sampler::Filter::Linear, sampler::MipmapMode::Nearest),
    MinFilter::Nearest => (sampler::Filter::Nearest, sampler::MipmapMode::Nearest),
    MinFilter::NearestMipmapNearest => (sampler::Filter::Nearest, sampler::MipmapMode::Nearest),
    MinFilter::LinearMipmapNearest => (sampler::Filter::Linear, sampler::MipmapMode::Nearest),
    MinFilter::NearestMipmapLinear => (sampler::Filter::Nearest, sampler::MipmapMode::Linear),
    MinFilter::LinearMipmapLinear => (sampler::Filter::Linear, sampler::MipmapMode::Linear),
  };
  
  let wrap_s = match sampler.wrap_s {
    WrappingMode::ClampToEdge => sampler::SamplerAddressMode::ClampToEdge,
    WrappingMode::MirroredRepeat => sampler::SamplerAddressMode::MirroredRepeat,
    WrappingMode::Repeat => sampler::SamplerAddressMode::Repeat,
  };
  
  let wrap_t = match sampler.wrap_t {
    WrappingMode::ClampToEdge => sampler::SamplerAddressMode::ClampToEdge,
    WrappingMode::MirroredRepeat => sampler::SamplerAddressMode::MirroredRepeat,
    WrappingMode::Repeat => sampler::SamplerAddressMode::Repeat,
  };
  
  sampler::Sampler::new(device, mag_filter, min_filter, mipmap_mode, wrap_s, wrap_t,
                                        sampler::SamplerAddressMode::ClampToEdge,
                                        0.0, 1.0, 0.0, 0.0).unwrap()
}

pub fn create_texture_from_dynamicimage(queue: Arc<Queue>, data: Option<image::DynamicImage>) -> Option<(Arc<ImmutableImage<format::R8G8B8A8Unorm>>, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>)> {
//  mesh_data.base_colour_texture(i)
  let mut final_texture = None;
  
  if data.is_some() {
    let texture_img = data.clone().unwrap().to_rgba();
        let dim = texture_img.dimensions();
        let image_data = texture_img.into_raw().clone();
        
        final_texture = Some(vkimage::immutable::ImmutableImage::from_iter(
            image_data.iter().cloned(),
            vkimage::Dimensions::Dim2d { width: dim.0, height: dim.1 },
            format::R8G8B8A8Unorm,
            queue).unwrap());
  }
  
  final_texture
}

pub fn create_3d_subbuffer(draw: DrawCall, projection: Matrix4<f32>, view_matrix: Matrix4<f32>, uniform_buffer: CpuBufferPool<vs_3d::ty::Data>) -> cpu_pool::CpuBufferPoolSubbuffer<vs_3d::ty::Data, Arc<memory::pool::StdMemoryPool>> {
  
  let rotation_x: Matrix4<f32> = Matrix4::from_angle_x(Deg(draw.rotation().x));
  let rotation_y: Matrix4<f32> = Matrix4::from_angle_y(Deg(draw.rotation().y));
  let rotation_z: Matrix4<f32> = Matrix4::from_angle_z(Deg(draw.rotation().z));
  
  let transformation: Matrix4<f32> = (Matrix4::from_translation(draw.position())* Matrix4::from_scale(draw.scale().x)) * (rotation_x*rotation_y*rotation_z);
  
  let point_light = 2.0;
  let directional_light = 0.0;
  
  let lighting_position: Matrix4<f32> =
    Matrix4::from_cols(
      // (x, y, z, n/a)
      Vector4::new(0.0, -0.6, 25.0, -1.0),
      Vector4::new(7.0, -0.6, 25.0, -1.0),
      Vector4::new(-2000000.0, 1000000.0, -2000000.0, -1.0),
      Vector4::new(0.0, 0.0, 0.0, -1.0)
    );
  
  let lighting_colour: Matrix4<f32> =
    // (R, G, B, light type)
    Matrix4::from_cols(
      Vector4::new(0.0, 0.0, 1.0, point_light), // blue light
      Vector4::new(1.0, 0.0, 0.0, point_light),  // red light
      Vector4::new(0.4, 0.4, 0.4, directional_light), //sun
      Vector4::new(0.0, 0.0, 0.0, -1.0)
    );
  
  // (Intensity, 1)
  let attenuation: Matrix4<f32> =
    Matrix4::from_cols(
      Vector4::new(1.0, 0.25, 0.25, -1.0),
      Vector4::new(1.0, 0.25, 0.25, -1.0),
      Vector4::new(1.0, 0.0, 0.0, -1.0),
      Vector4::new(0.0, 0.0, 0.0, -1.0) 
    );
  
  let uniform_data = vs_3d::ty::Data {
    transformation: transformation.into(),
    view : (view_matrix).into(),
    proj : projection.into(),
    lightpositions: lighting_position.into(),
    lightcolours: lighting_colour.into(),
    attenuations: attenuation.into(),
  };
  
  uniform_buffer.next(uniform_data).unwrap()
}

pub fn create_shadow_attachments(device: Arc<Device>, dim: [u32; 2]) -> Arc<vkimage::AttachmentImage> {
  let shadow_usage = ImageUsage {
    transfer_destination: true,
    sampled: true,
    .. ImageUsage::none()
  };
  
  let shadow_attachment = vkimage::AttachmentImage::with_usage(device, dim, format::Format::D16Unorm, shadow_usage).unwrap();
  
  shadow_attachment
}

pub fn create_shadow_renderpass(device: Arc<Device>, dim: [u32; 2]) -> CustomRenderpass {
  let shadow_renderpass = Arc::new(single_pass_renderpass!(device.clone(),
      attachments: {
        depth_attachment: {
          load: DontCare,
          store: Store,
          format: format::Format::D16Unorm,
          samples: 1,
        }
      },
      pass: {
        color: [],
        depth_stencil: {depth_attachment},
        resolve: [],
      }
    ).unwrap());
    
  let vs_shadow = vs_shadow::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_shadow = fs_shadow::Shader::load(device.clone()).expect("failed to create shader module");
  
  let shadow_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
      .vertex_input_single_buffer::<Vertex3d>()
      .vertex_shader(vs_shadow.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .fragment_shader(fs_shadow.main_entry_point(), ())
      .depth_clamp(true)
      .depth_stencil_simple_depth()
      .render_pass(framebuffer::Subpass::from(shadow_renderpass.clone(), 0).unwrap())
      .build(device.clone())
      .unwrap());
  
  let shadow_attachment = create_shadow_attachments(device.clone(), dim);
  
  let shadowframebuffer = Arc::new({
      framebuffer::Framebuffer::start(shadow_renderpass.clone())
          .add(shadow_attachment.clone()).unwrap()
          .build().unwrap()
  });
  
  let mut shadow_customrenderpass = CustomRenderpass::new(vec!(shadow_attachment));
  
  shadow_customrenderpass.set_renderpass(shadow_renderpass);
  shadow_customrenderpass.set_pipelines(vec!(shadow_pipeline));
  shadow_customrenderpass.set_framebuffer(shadowframebuffer);
  
  shadow_customrenderpass
}

pub fn recreate_shadow_renderpass(renderpass: &mut CustomRenderpass, device: Arc<Device>, dim: [u32; 2]) {
  let shadow_attachment = create_shadow_attachments(device.clone(), dim);
  
  let shadowframebuffer = Arc::new({
      framebuffer::Framebuffer::start(renderpass.renderpass().clone())
          .add(shadow_attachment.clone()).unwrap()
          .build().unwrap()
  });
  
  renderpass.update_attachments(vec!(shadow_attachment));
  renderpass.set_framebuffer(shadowframebuffer);
}

pub fn create_gbuffer_attachments(device: Arc<Device>, dim: [u32; 2], samples: u32) -> (Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>, Arc<vkimage::AttachmentImage>) {
  let gbuffer_usage_input = ImageUsage {
    color_attachment: true,
    input_attachment: true,
    transfer_destination: true,
    sampled: true,
    .. ImageUsage::none()
  };
  
  let gbuffer_usage = ImageUsage {
    color_attachment: true,
    transfer_destination: true,
    sampled: true,
    .. ImageUsage::none()
  };
  
  let colour_attachment = vkimage::AttachmentImage::transient_multisampled_input_attachment(device.clone(), dim, samples, format::Format::R16G16B16A16Unorm).unwrap();
  let normal_attachment = vkimage::AttachmentImage::transient_multisampled_input_attachment(device.clone(), dim, samples, format::Format::R8G8B8A8Unorm).unwrap();
  let position_attachment = vkimage::AttachmentImage::transient_multisampled_input_attachment(device.clone(), dim, samples, format::Format::R8G8B8A8Unorm).unwrap();
  let uv_attachment = vkimage::AttachmentImage::transient_multisampled_input_attachment(device.clone(), dim, samples, format::Format::R8G8B8A8Unorm).unwrap();
  let mr_attachment = vkimage::AttachmentImage::transient_multisampled_input_attachment(device.clone(), dim, samples, format::Format::R8G8B8A8Unorm).unwrap();
  let ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(device.clone(), dim, samples, format::Format::R16G16B16A16Unorm).unwrap();
  let ms_depth_attachment = vkimage::AttachmentImage::transient_multisampled_input_attachment(device.clone(), dim, samples, format::Format::D16Unorm).unwrap();
  let fullcolour_attachment = vkimage::AttachmentImage::with_usage(device.clone(), dim, format::Format::R16G16B16A16Unorm, gbuffer_usage_input).unwrap();
  let bloom_attachment = vkimage::AttachmentImage::with_usage(device, dim, format::Format::R16G16B16A16Unorm, gbuffer_usage).unwrap();
  
  (colour_attachment, normal_attachment, position_attachment, uv_attachment, mr_attachment, ms_colour_attachment, ms_depth_attachment, fullcolour_attachment, bloom_attachment)
}

pub fn create_gbuffer(device: Arc<Device>, dim: [u32; 2], samples: u32) -> CustomRenderpass {
  let gbuffer_renderpass = Arc::new(ordered_passes_renderpass!(device.clone(),
    attachments: {
      colour_attachment: {
        load: Clear,
        store: DontCare,
        format: format::Format::R16G16B16A16Unorm,
        samples: samples,
      },
      normal_attachment: {
        load: Clear,
        store: DontCare,
        format: format::Format::R8G8B8A8Unorm,
        samples: samples,
      },
      position_attachment: {
        load: Clear,
        store: DontCare,
        format: format::Format::R8G8B8A8Unorm,
        samples: samples,
      },
      uv_attachment: {
        load: Clear,
        store: DontCare,
        format: format::Format::R8G8B8A8Unorm,
        samples: samples,
      },
      mr_attachment: { // metallic_roughness_attachment
        load: Clear,
        store: DontCare,
        format: format::Format::R8G8B8A8Unorm,
        samples: samples,
      },
      multisample_colour: {
        load: Clear,
        store: DontCare,
        format: format::Format::R16G16B16A16Unorm,
        samples: samples,
      },
      ms_depth_attachment: {
        load: Clear,
        store: DontCare,
        format: format::Format::D16Unorm,
        samples: samples,
      },
      resolve_fullcolour: {
        load: DontCare,
        store: Store,
        format: format::Format::R16G16B16A16Unorm,
        samples: 1,
      },
      bloom_attachment: {
        load: DontCare,
        store: Store,
        format: format::Format::R16G16B16A16Unorm,
        samples: 1,
      }
    },
    passes: [
      {
        color: [colour_attachment, normal_attachment, position_attachment, uv_attachment, mr_attachment],
        depth_stencil: {ms_depth_attachment},
        input: []
      },
      {
        color: [multisample_colour],
        depth_stencil: {},
        input: [colour_attachment, normal_attachment, position_attachment, uv_attachment, mr_attachment, ms_depth_attachment],
        resolve: [resolve_fullcolour]
      }, 
      {
        color: [bloom_attachment],
        depth_stencil: {},
        input: [resolve_fullcolour]
      }
    ]
  ).unwrap());
  
  let vs_gbuffer_3d = vs_gbuffer_3d::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_gbuffer_3d = fs_gbuffer_3d::Shader::load(device.clone()).expect("failed to create shader module");
  let vs_plain = vs_plain::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_lights = fs_lights::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_post_bloom = fs_post_bloom::Shader::load(device.clone()).expect("failed to create shader module");
  
  let gbuffer_pipeline_0 = Arc::new(pipeline::GraphicsPipeline::start()
      .vertex_input_single_buffer::<Vertex3d>()
      .vertex_shader(vs_gbuffer_3d.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .fragment_shader(fs_gbuffer_3d.main_entry_point(), ())
      .depth_clamp(true)
      .depth_stencil_simple_depth()
      .blend_alpha_blending()
      .polygon_mode_fill()
      .render_pass(framebuffer::Subpass::from(gbuffer_renderpass.clone(), 0).unwrap())
      .build(device.clone())
      .unwrap());
  
  let gbuffer_pipeline_1 = Arc::new(pipeline::GraphicsPipeline::start()
      .vertex_input_single_buffer::<Vertex2d>()
      .vertex_shader(vs_plain.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .fragment_shader(fs_lights.main_entry_point(), ())
      .blend_alpha_blending()
      .depth_stencil_disabled()
      .render_pass(framebuffer::Subpass::from(gbuffer_renderpass.clone(), 1).unwrap())
      .build(device.clone())
      .unwrap());
      
  let gbuffer_pipeline_2 = Arc::new(pipeline::GraphicsPipeline::start()
      .vertex_input_single_buffer::<Vertex2d>()
      .vertex_shader(vs_plain.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .fragment_shader(fs_post_bloom.main_entry_point(), ())
      .blend_alpha_blending()
      .depth_stencil_disabled()
      .render_pass(framebuffer::Subpass::from(gbuffer_renderpass.clone(), 2).unwrap())
      .build(device.clone())
      .unwrap());
  
  let (colour_attachment, normal_attachment, 
       position_attachment, uv_attachment,
       mr_attachment, ms_colour_attachment, 
       ms_depth_attachment, fullcolour_attachment, 
       bloom_attachment) = create_gbuffer_attachments(device.clone(), dim, samples);
  
  let gframebuffer = Arc::new({
      framebuffer::Framebuffer::start(gbuffer_renderpass.clone())
          .add(colour_attachment.clone()).unwrap()
          .add(normal_attachment.clone()).unwrap()
          .add(position_attachment.clone()).unwrap()
          .add(uv_attachment.clone()).unwrap()
          .add(mr_attachment.clone()).unwrap()
          .add(ms_colour_attachment.clone()).unwrap()
          .add(ms_depth_attachment.clone()).unwrap()
          .add(fullcolour_attachment.clone()).unwrap()
          .add(bloom_attachment.clone()).unwrap()
          .build().unwrap()
  });
  
  let mut gbuffer_customrenderpass = CustomRenderpass::new(vec!(colour_attachment, normal_attachment, 
                                                        position_attachment, uv_attachment, 
                                                        mr_attachment, ms_colour_attachment, 
                                                        ms_depth_attachment, fullcolour_attachment, 
                                                        bloom_attachment));
  
  gbuffer_customrenderpass.set_renderpass(gbuffer_renderpass);
  gbuffer_customrenderpass.set_pipelines(vec!(gbuffer_pipeline_0, gbuffer_pipeline_1, gbuffer_pipeline_2));
  gbuffer_customrenderpass.set_framebuffer(gframebuffer);
  
  gbuffer_customrenderpass
}

pub fn recreate_gbuffer(renderpass: &mut CustomRenderpass, device: Arc<Device>, dim: [u32; 2], samples: u32) {
  let (colour_attachment, normal_attachment, 
       position_attachment, uv_attachment,
       mr_attachment, ms_colour_attachment, 
       ms_depth_attachment, fullcolour_attachment, 
       bloom_attachment) = create_gbuffer_attachments(device.clone(), dim, samples);
  
  let gframebuffer = Arc::new({
      framebuffer::Framebuffer::start(renderpass.renderpass().clone())
          .add(colour_attachment.clone()).unwrap()
          .add(normal_attachment.clone()).unwrap()
          .add(position_attachment.clone()).unwrap()
          .add(uv_attachment.clone()).unwrap()
          .add(mr_attachment.clone()).unwrap()
          .add(ms_colour_attachment.clone()).unwrap()
          .add(ms_depth_attachment.clone()).unwrap()
          .add(fullcolour_attachment.clone()).unwrap()
          .add(bloom_attachment.clone()).unwrap()
          .build().unwrap()
  });
  
  renderpass.update_attachments(vec!(colour_attachment, normal_attachment, position_attachment, uv_attachment, mr_attachment, ms_colour_attachment, ms_depth_attachment, fullcolour_attachment, bloom_attachment));
  renderpass.set_framebuffer(gframebuffer);
}
