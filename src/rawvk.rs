use font::GenericFont;
use window::VkWindow;
use drawcalls::DrawCall;
use drawcalls::DrawMath;
use graphics;
use graphics::CoreRender;
use settings::Settings;
use camera::Camera;
use model_data;

use image;
use winit;

use vulkano::image as vkimage;
use vulkano::sampler;

use vulkano::memory;

use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::sync::NowFuture;
use vulkano::sync::FlushError;

use vulkano::swapchain;
use vulkano::swapchain::AcquireError;

use vulkano::buffer::cpu_pool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferAccess;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::ImmutableBuffer;

use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;

use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::pipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::viewport::Scissor;
use vulkano::pipeline::GraphicsPipelineAbstract;

use vulkano::format;
use vulkano::format::ClearValue;
use vulkano::image::ImmutableImage;
use vulkano::descriptor;
use vulkano::descriptor::descriptor_set;
use vulkano::descriptor::pipeline_layout;
use vulkano::swapchain::SwapchainCreationError;

use std::env;
use std::mem;
use std::cmp;
use std::time;
use std::iter;
use std::slice;
use std::f32::consts;
use std::marker::Sync;
use std::marker::Send;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;

use cgmath;
use cgmath::Deg;
use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix3;
use cgmath::Matrix4;
use cgmath::SquareMatrix;
use cgmath::Rotation3;
use cgmath::InnerSpace;

impl_vertex!(graphics::Vertex2d, position, uv);

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

mod vs_3d {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/Vk3D.vert"]
  struct Dummy;
}

mod fs_3d {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/Vk3D.frag"]
  struct Dummy;
}

mod vs_post_blur {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkPostBlur.vert"]
  struct Dummy;
}

mod fs_post_blur {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkPostBlur.frag"]
  struct Dummy;
}

mod vs_post_bloom {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkPostBloom.vert"]
  struct Dummy;
}

mod fs_post_bloom {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkPostBloom.frag"]
  struct Dummy;
}

mod vs_post_final {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkPostFinal.vert"]
  struct Dummy;
}

mod fs_post_final {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkPostFinal.frag"]
  struct Dummy;
}

pub fn draw_dynamic(cb: AutoCommandBufferBuilder, 
                    dimensions: [u32; 2], 
                    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>, 
                    vertex_buffer: Vec<Arc<BufferAccess + Send + Sync>>, 
                    index_buffer: Arc<CpuAccessibleBuffer<[u16]>>, 
                    uniform_buffer: Arc<descriptor::DescriptorSet + Send + Sync>) -> AutoCommandBufferBuilder {
  cb.draw_indexed(pipeline,
                  DynamicState {
                    line_width: None,
                    viewports: Some(vec![Viewport {
                      origin: [0.0, 0.0],
                      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                      depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                  },
                  vertex_buffer,
                  index_buffer,
                  uniform_buffer, ()).unwrap()
}

pub fn draw_immutable(cb: AutoCommandBufferBuilder, 
                      dimensions: [u32; 2], 
                      pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>, 
                      vertex_buffer: Vec<Arc<BufferAccess + Send + Sync>>, 
                      index_buffer: Arc<ImmutableBuffer<[u16]>>, 
                      uniform_buffer: Arc<descriptor::DescriptorSet + Send + Sync>) -> AutoCommandBufferBuilder {
  cb.draw_indexed(pipeline,
                  DynamicState {
                    line_width: None,
                    viewports: Some(vec![Viewport {
                      origin: [0.0, 0.0],
                      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                      depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                  },
                  vertex_buffer,
                  index_buffer,
                  uniform_buffer, ()).unwrap()
}

#[derive(Clone)]
pub struct ModelInfo {
  location: String,
  texture: String,
}

pub struct Model {
  vertex_buffer: Option<Vec<Arc<BufferAccess + Send + Sync>>>,
  index_buffer: Option<Arc<ImmutableBuffer<[u16]>>>,
}

pub struct DynamicModel {
  vertex_buffer: Option<Vec<Arc<BufferAccess + Send + Sync>>>,
  index_buffer: Option<Arc<CpuAccessibleBuffer<[u16]>>>
}

pub struct VK2D {
  vao: Model,
  custom_vao: HashMap<String, Model>,
  custom_dynamic_vao: HashMap<String, DynamicModel>,
  projection: Matrix4<f32>,
  
  pipeline_texture: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  uniform_buffer_texture: cpu_pool::CpuBufferPool<vs_texture::ty::Data>,
  
  pipeline_text: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  uniform_buffer_text: cpu_pool::CpuBufferPool<vs_text::ty::Data>,
}

pub struct VK3D {
  depth_buffer: Option<Arc<vkimage::AttachmentImage<format::D16Unorm>>>,
  
  models: HashMap<String, Model>,
  
  pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  
  projection: Matrix4<f32>,
  view: Matrix4<f32>,
  scale: Matrix4<f32>,
  
  uniform_buffer: cpu_pool::CpuBufferPool<vs_3d::ty::Data>,
}

pub struct VKPOST {
  bloom_renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
  bloom_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  bloom_framebuffer: Option<Arc<framebuffer::FramebufferAbstract + Send + Sync>>,
  bloom_uniformbuffer: cpu_pool::CpuBufferPool<vs_post_bloom::ty::Data>,
  bloom_attachment: Arc<vkimage::AttachmentImage>,
  
  blur_uniformbuffer: cpu_pool::CpuBufferPool<vs_post_blur::ty::Data>,
  blur_ping_renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
  blur_ping_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  blur_ping_framebuffer: Option<Arc<framebuffer::FramebufferAbstract + Send + Sync>>,
  blur_ping_attachment: Arc<vkimage::AttachmentImage>,
  blur_pong_renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
  blur_pong_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  blur_pong_framebuffer: Option<Arc<framebuffer::FramebufferAbstract + Send + Sync>>,
  blur_pong_attachment: Arc<vkimage::AttachmentImage>,
  
  final_renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
  final_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  final_framebuffer: Option<Vec<Arc<framebuffer::FramebufferAbstract + Send + Sync>>>,
  final_uniformbuffer: cpu_pool::CpuBufferPool<vs_post_final::ty::Data>,
}

pub struct RawVk {
  ready: bool,
  fonts: HashMap<String, GenericFont>,
  textures: HashMap<String, Arc<ImmutableImage<format::R8G8B8A8Unorm>>>,
  texture_paths: HashMap<String, String>,
  model_paths: HashMap<String, ModelInfo>,
  
  clear_colour: Vector4<f32>,
  
  framebuffers: Option<Arc<framebuffer::FramebufferAbstract + Send + Sync>>,
  main_renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
  fullcolour_attachment: Arc<vkimage::AttachmentImage>,
  ms_colour_attachment: Arc<vkimage::AttachmentImage>,
  ms_depth_attachment: Arc<vkimage::AttachmentImage>,
  samples: u32,
  
  //3D
  vk3d: VK3D,
  
  //2D
  vk2d: VK2D,
  
  // Post Processing
  vkpost: VKPOST,
  
  // Vk System stuff
  min_dimensions: [u32; 2],
  pub window: VkWindow,
  sampler: Arc<sampler::Sampler>,
  
  recreate_swapchain: bool,
  
  previous_frame_end: Option<Box<GpuFuture>>,
}

impl RawVk {
  pub fn new() -> RawVk {
    //avoid_winit_wayland_hack
    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    env::set_var("WINIT_UNIX_BACKEND", "x11");
    println!("Forcing x11");
    
    let mut settings = Settings::load();
    let width = settings.get_resolution()[0];
    let height = settings.get_resolution()[1];
    let min_width = settings.get_minimum_resolution()[0];
    let min_height = settings.get_minimum_resolution()[1];
    let fullscreen = settings.is_fullscreen();
    
    let window = VkWindow::new(width, height, min_width, min_height, fullscreen);
    
    let proj_2d = Matrix4::identity();
    let proj_3d = Matrix4::identity();
    
    let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0.0, 0.0, -1.0), cgmath::Point3::new(0.0, 0.0, 0.0), cgmath::Vector3::new(0.0, -1.0, 0.0));
    let scale = cgmath::Matrix4::from_scale(0.1);
    
    let sampler = sampler::Sampler::new(window.get_device(), sampler::Filter::Linear,
                                                   sampler::Filter::Linear, 
                                                   sampler::MipmapMode::Nearest,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   0.0, 1.0, 0.0, 0.0).unwrap();
 
    let text_uniform = cpu_pool::CpuBufferPool::new(window.get_device(), BufferUsage::uniform_buffer());
    let texture_uniform = cpu_pool::CpuBufferPool::new(window.get_device(), BufferUsage::uniform_buffer());
    let uniform_3d = cpu_pool::CpuBufferPool::<vs_3d::ty::Data>::new(window.get_device(), BufferUsage::uniform_buffer());
    let post_final_uniform = cpu_pool::CpuBufferPool::<vs_post_final::ty::Data>::new(window.get_device(), BufferUsage::uniform_buffer());
    let post_bloom_uniform = cpu_pool::CpuBufferPool::<vs_post_bloom::ty::Data>::new(window.get_device(), BufferUsage::uniform_buffer());
    let post_blur_uniform = cpu_pool::CpuBufferPool::<vs_post_blur::ty::Data>::new(window.get_device(), BufferUsage::uniform_buffer());
    let previous_frame_end = Some(Box::new(now(window.get_device())) as Box<GpuFuture>);
    
    let mut msaa_samples = settings.get_msaa();
    if msaa_samples <= 0 {
      msaa_samples = 1;
    }
    
    let mut max_samples = window.get_max_msaa();
    
    println!("Max MSAA: x{}", max_samples);
    
    let samples = cmp::min(msaa_samples, max_samples as u32);
    println!("Current MSAA: x{}\n", samples);
    
    let dim = window.get_dimensions();
    let fullcolour_attachment = vkimage::AttachmentImage::sampled(window.get_device(), dim, window.get_swapchain().format()).unwrap();
    let bloom_attachment = vkimage::AttachmentImage::sampled(window.get_device(), dim, window.get_swapchain().format()).unwrap();
    let blur_ping_attachment = vkimage::AttachmentImage::sampled(window.get_device(), dim, window.get_swapchain().format()).unwrap();
    let blur_pong_attachment = vkimage::AttachmentImage::sampled(window.get_device(), dim, window.get_swapchain().format()).unwrap();
    let ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(window.get_device(), dim, samples, window.get_swapchain().format()).unwrap();
    let ms_depth_attachment = vkimage::AttachmentImage::transient_multisampled(window.get_device(), dim, samples, format::Format::D16Unorm).unwrap();
    
    RawVk {
      ready: false,
      fonts: HashMap::new(),
      textures: HashMap::new(),
      texture_paths: HashMap::new(),
      model_paths: HashMap::new(),

      clear_colour: Vector4::new(0.0, 0.0, 0.0, 1.0),

      framebuffers: None,
      main_renderpass: None,
      fullcolour_attachment: fullcolour_attachment,
      ms_colour_attachment: ms_colour_attachment,
      ms_depth_attachment: ms_depth_attachment,
      samples: samples,

      // 3D
      vk3d: VK3D {
        depth_buffer: None,
        
        models: HashMap::new(),
        
        pipeline: None,
        
        projection: proj_3d,
        view: view,
        scale: scale,

        uniform_buffer: uniform_3d,
      },
      
      //2D
      vk2d: VK2D {
        vao: Model {
          vertex_buffer: None,
          index_buffer: None,
        },
        custom_vao: HashMap::new(),
        custom_dynamic_vao: HashMap::new(),
        
        projection: proj_2d,
        
        pipeline_texture: None,
        uniform_buffer_texture: texture_uniform,
        
        pipeline_text: None,
        uniform_buffer_text: text_uniform,
      },
      
      // Post Processing
      vkpost: VKPOST {
        bloom_renderpass: None,
        bloom_pipeline: None,
        bloom_framebuffer: None,
        bloom_uniformbuffer: post_bloom_uniform,
        bloom_attachment: bloom_attachment,
        
        blur_uniformbuffer: post_blur_uniform,
        blur_ping_renderpass: None,
        blur_ping_pipeline: None,
        blur_ping_framebuffer: None,
        blur_ping_attachment: blur_ping_attachment,
        blur_pong_renderpass: None,
        blur_pong_pipeline: None,
        blur_pong_framebuffer: None,
        blur_pong_attachment: blur_pong_attachment,
        
        final_renderpass: None,
        final_pipeline: None,
        final_framebuffer: None,
        final_uniformbuffer: post_final_uniform,
      },
      
      // Vk System
      min_dimensions: [min_width, min_height],
      window: window,
      sampler: sampler,

      recreate_swapchain: false,
      
      previous_frame_end: previous_frame_end,
    }
  }
  
  pub fn with_title(mut self, title: String) -> RawVk {
    self.window.set_title(title);
    self
  }
  
  pub fn with_samples(mut self, samples: u32) -> RawVk {
    self.samples = samples;
    let dim = self.window.get_dimensions();
    let ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(self.window.get_device(), dim, samples, self.window.get_swapchain().format()).unwrap();
    let ms_depth_attachment = vkimage::AttachmentImage::transient_multisampled(self.window.get_device(), dim, samples, format::Format::D16Unorm).unwrap();
    
    self.ms_colour_attachment = ms_colour_attachment;
    self.ms_depth_attachment = ms_depth_attachment;
    
    self
  }
  
  pub fn create_2d_vertex(&self) -> Arc<BufferAccess + Send + Sync> {
    let square = {
      [
          graphics::Vertex2d { position: [  0.5 ,   0.5 ], uv: [1.0, 0.0] },
          graphics::Vertex2d { position: [ -0.5,    0.5 ], uv: [0.0, 0.0] },
          graphics::Vertex2d { position: [ -0.5,   -0.5 ], uv: [0.0, 1.0] },
          graphics::Vertex2d { position: [  0.5 ,  -0.5 ], uv: [1.0, 1.0] },
      ]
    };
    
    CpuAccessibleBuffer::from_iter(self.window.get_device(), 
                                   BufferUsage::vertex_buffer(), 
                                   square.iter().cloned())
                                   .expect("failed to create vertex buffer")
  }
  
  pub fn create_2d_index(&self) -> (Arc<ImmutableBuffer<[u16]>>,
                                    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    
    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
    ImmutableBuffer::from_iter(indices.iter().cloned(), 
                               BufferUsage::index_buffer(), 
                               self.window.get_queue())
                               .expect("failed to create immutable index buffer")
  }
  
  pub fn create_dynamic_custom_2d_model(&mut self, mut verts: Vec<graphics::Vertex2d>, indices: Vec<u16>) -> DynamicModel {
    for i in 0..verts.len() {
      verts[i].position[0] *= -1.0;
      verts[i].position[1] *= -1.0;
    }
    
    let vert =  CpuAccessibleBuffer::from_iter(self.window.get_device(), 
                                   BufferUsage::vertex_buffer(), 
                                   verts.iter().cloned())
                                   .expect("Vulkan failed to create custom vertex buffer");
    let idx = CpuAccessibleBuffer::from_iter(self.window.get_device(),
                                   BufferUsage::index_buffer(), 
                                   indices.iter().cloned())
                                   .expect("Vulkan failed to create custom index buffer");
    
    DynamicModel {
      vertex_buffer: Some(vec!(vert)),
      index_buffer: Some(idx),
    }
  }
  
  pub fn create_static_custom_2d_model(&mut self, mut verts: Vec<graphics::Vertex2d>, indices: Vec<u16>) -> Model {
    for i in 0..verts.len() {
      verts[i].position[1] *= -1.0;
    }
    
    let vert =  CpuAccessibleBuffer::from_iter(self.window.get_device(), 
                                   BufferUsage::vertex_buffer(), 
                                   verts.iter().cloned())
                                   .expect("Vulkan failed to create custom vertex buffer");
    
    let (idx_buffer, future_idx) = ImmutableBuffer::from_iter(indices.iter().cloned(), 
                               BufferUsage::index_buffer(), 
                               self.window.get_queue())
                               .expect("failed to create immutable index buffer");
    
    self.previous_frame_end = Some(Box::new(future_idx.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    
    Model {
      vertex_buffer: Some(vec!(vert)),
      index_buffer: Some(idx_buffer),
    }
  }
  
  pub fn create_vertex(&self, verticies: iter::Cloned<slice::Iter<model_data::Vertex>>) -> Arc<BufferAccess + Send + Sync> {
      CpuAccessibleBuffer::from_iter(self.window.get_device(), 
                                     BufferUsage::vertex_buffer(), 
                                     verticies)
                                     .expect("failed to create vertex buffer")
  }
  
  pub fn create_index(&self, indices: iter::Cloned<slice::Iter<u16>>) -> (Arc<ImmutableBuffer<[u16]>>,
                                                                           CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
      ImmutableBuffer::from_iter(indices, BufferUsage::index_buffer(), 
                                 self.window.get_queue())
                                 .expect("failed to create immutable teapot index buffer")
  }
  
  pub fn create_texture_subbuffer(&self, draw: DrawCall) -> cpu_pool::CpuBufferPoolSubbuffer<vs_texture::ty::Data,
                                                                      Arc<memory::pool::StdMemoryPool>> {
    let model = DrawMath::calculate_texture_model(draw.get_translation(), draw.get_size(), -draw.get_x_rotation() -180.0);
    
    let has_texture = {
      let mut value = 1.0;
      if draw.get_texture() == &String::from("") {
        value = 0.0;
      }
      value
    };
    
    let uniform_data = vs_texture::ty::Data {
      projection: self.vk2d.projection.into(),
      model: model.into(),
      colour: draw.get_colour().into(),
      has_texture: Vector4::new(has_texture, 0.0, 0.0, 0.0).into(),
    };
    self.vk2d.uniform_buffer_texture.next(uniform_data).unwrap()
  }
  
  pub fn create_3d_subbuffer(&self, draw: DrawCall) -> cpu_pool::CpuBufferPoolSubbuffer<vs_3d::ty::Data, Arc<memory::pool::StdMemoryPool>> {
    
    let rotation_x: Matrix4<f32> = Matrix4::from_angle_x(Deg(draw.get_x_rotation()));
    let rotation_y: Matrix4<f32> = Matrix4::from_angle_y(Deg(draw.get_y_rotation()));
    let rotation_z: Matrix4<f32> = Matrix4::from_angle_z(Deg(draw.get_z_rotation()));
         
    let transformation: Matrix4<f32> = (cgmath::Matrix4::from_translation(draw.get_translation())* cgmath::Matrix4::from_scale(draw.get_size().x)) * (rotation_x*rotation_y*rotation_z);
    
    let point_light = 2.0;
    let directional_light = 0.0;
    let metallic = 1.0;
    let roughness = 1.0;
    
    let lighting_position: Matrix4<f32> =
      Matrix4::from_cols(
        // (x, y, z, n/a)
        Vector4::new(0.0, -0.6, 25.0, -1.0),
        Vector4::new(7.0, -0.6, 25.0, -1.0),
        Vector4::new(-2000000.0, 1000000.0, -2000000.0, -1.0),
        Vector4::new(0.0, 0.0, 0.0, -1.0)
      );
    
    let reflectivity = {
      let mut temp = 1.0;
      if draw.get_texture() == "terrain" {
        temp = 0.0;
      }
      temp
    };
    
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
      view : (self.vk3d.view * self.vk3d.scale).into(),
      proj : self.vk3d.projection.into(),
      lightpositions: lighting_position.into(),
      lightcolours: lighting_colour.into(),
      attenuations: attenuation.into(),
    };

    self.vk3d.uniform_buffer.next(uniform_data).unwrap()
  }
  
  pub fn create_2d_projection(&self, width: f32, height: f32) -> Matrix4<f32> {
    cgmath::ortho(0.0, width, height, 0.0, -1.0, 1.0)
  }
  
  pub fn create_3d_projection(&self, width: f32, height: f32) -> Matrix4<f32> {
    cgmath::perspective(/*cgmath::Rad(consts::FRAC_PI_4)*/cgmath::Deg(45.0), { width as f32 / height as f32 }, 0.01, 100.0)
  }
  
  pub fn create_depth_buffer(&self) -> Option<Arc<vkimage::AttachmentImage<format::D16Unorm>>> {
    Some(vkimage::attachment::AttachmentImage::transient(
                                self.window.get_device().clone(),
                                self.window.get_dimensions(),
                                format::D16Unorm)
                                .unwrap())
  }
}

impl CoreRender for RawVk {
  fn load_instanced(&mut self, reference: String, max_instances: i32) {
    
  }
  
  fn load_static_geometry(&mut self, reference: String, vertices: Vec<graphics::Vertex2d>, indices: Vec<u16>) {
    let model = self.create_static_custom_2d_model(vertices, indices);
    self.vk2d.custom_vao.insert(reference, model);
  }
  
  fn load_dynamic_geometry(&mut self, reference: String, vertices: Vec<graphics::Vertex2d>, indices: Vec<u16>) {
    let model = self.create_dynamic_custom_2d_model(vertices, indices);
    self.vk2d.custom_dynamic_vao.insert(reference, model);
  }
  
  fn preload_model(&mut self, reference: String, location: String, texture: String) {
    self.load_model(reference.clone(), location, texture.clone());
    self.load_texture(reference, texture);
  }
  
  fn add_model(&mut self, reference: String, location: String, texture: String) {
    self.model_paths.insert(reference.clone(), ModelInfo {location: location, texture: texture.clone()});
    self.add_texture(reference, texture);
  }
  
  fn load_model(&mut self, reference: String, location: String, texture: String) {
    let start_time = time::Instant::now();
    
    let model = model_data::Loader::load_opengex(location.clone(), texture);
    
    let vert3d_buffer = self.create_vertex(model.get_verticies().iter().cloned());
    let (idx_3d_buffer, future_3d_idx) = self.create_index(model.get_indices().iter().cloned()); 
    
    let model = Model {
      vertex_buffer: Some(vec!(vert3d_buffer)),
      index_buffer: Some(idx_3d_buffer),
    };
    self.vk3d.models.insert(reference, model);
    
    self.previous_frame_end = Some(Box::new(future_3d_idx.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    
    let total_time = start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (total_time*1000f64) as f32, location);
  }
  
  fn preload_texture(&mut self, reference: String, location: String) {
    self.load_texture(reference, location);
  }
  
  fn add_texture(&mut self, reference: String, location: String) {
    self.texture_paths.insert(reference, location);
  }
  
  fn load_texture(&mut self, reference: String, location: String) {
    if location == String::from("") {
      return;
    }
    
    let texture_start_time = time::Instant::now();
    
    let (texture, tex_future) = {
      let texture = location.clone();
      let image = image::open(&location).expect(&("No file or Directory at: ".to_string() + &texture)).to_rgba(); 
      let (width, height) = image.dimensions();
      let image_data = image.into_raw().clone();

      vkimage::immutable::ImmutableImage::from_iter(
              image_data.iter().cloned(),
              vkimage::Dimensions::Dim2d { width: width, height: height },
              format::R8G8B8A8Unorm,
               self.window.get_queue()).unwrap()
    };
    self.previous_frame_end = Some(Box::new(tex_future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    self.textures.insert(reference.clone(), texture);
   
    let texture_time = texture_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (texture_time*1000f64) as f32, location);
  }
  
  fn preload_font(&mut self, reference: String, font: &[u8], font_texture: String) {
    self.load_font(reference.clone(), font);
    self.load_texture(reference, font_texture);
  }
  
  fn add_font(&mut self, reference: String, font: &[u8], font_texture: String) {
    self.load_font(reference.clone(), font);
    self.texture_paths.insert(reference, font_texture);
  }
  
  fn load_font(&mut self, reference: String, font: &[u8]) {
   let mut new_font = GenericFont::new();
    new_font.load_font(font);
    
    self.fonts.insert(reference.clone(), new_font);
  }
  
  /// Prepares shaders, pipelines, and vertex, index buffers
  /// # Warning
  /// You must call this function otherwise will result in crash
  fn load_shaders(&mut self) {
    let dimensions = {
      self.window.get_dimensions()
    };
    
    self.vk2d.projection = self.create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
    self.vk3d.projection = self.create_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
    
    self.vk3d.depth_buffer = self.create_depth_buffer();
    
    // 2D
    let vert_buffer = self.create_2d_vertex();
    let (idx_buffer, future_idx) = self.create_2d_index();
    
    self.vk2d.vao.vertex_buffer = Some(vec!(vert_buffer));
    self.vk2d.vao.index_buffer = Some(idx_buffer);
    
    self.previous_frame_end = Some(Box::new(future_idx.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    
    let vs_3d = vs_3d::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_3d = fs_3d::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_texture = vs_texture::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_texture = fs_texture::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_text = vs_text::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_text = fs_text::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_post_final = vs_post_final::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_post_final = fs_post_final::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_post_bloom = vs_post_bloom::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_post_bloom = fs_post_bloom::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_post_blur = vs_post_blur::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_post_blur = fs_post_blur::Shader::load(self.window.get_device()).expect("failed to create shader module");
    
    self.main_renderpass = Some(Arc::new(single_pass_renderpass!(self.window.get_device(),
      attachments: {
        multisample_colour: {
          load: Clear,
          store: DontCare,
          format: self.window.get_swapchain().format(),
          samples: self.samples,
        },
        ms_depth_attachment: {
          load: Clear,
          store: DontCare,
          format: format::Format::D16Unorm,
          samples: self.samples,
        },
        resolve_fullcolour: {
          load: DontCare,
          store: Store,
          format: self.window.get_swapchain().format(),
          samples: 1,
        }
      },
      pass: {
        color: [multisample_colour],
        depth_stencil: {ms_depth_attachment},
        resolve: [resolve_fullcolour],
      }
    ).unwrap()));
    
    self.vkpost.bloom_renderpass = Some(Arc::new(single_pass_renderpass!(self.window.get_device(),
      attachments: {
        out_colour: {
          load: DontCare,
          store: Store,
          format: self.window.get_swapchain().format(),
          samples: 1,
        }
      },
      pass: {
        color: [out_colour],
        depth_stencil: {},
        resolve: [],
      }
    ).unwrap()));
    
    self.vkpost.blur_ping_renderpass = Some(Arc::new(single_pass_renderpass!(self.window.get_device(),
      attachments: {
        out_colour: {
          load: DontCare,
          store: Store,
          format: self.window.get_swapchain().format(),
          samples: 1,
        }
      },
      pass: {
        color: [out_colour],
        depth_stencil: {},
        resolve: [],
      }
    ).unwrap()));
    
    self.vkpost.blur_pong_renderpass = Some(Arc::new(single_pass_renderpass!(self.window.get_device(),
      attachments: {
        out_colour: {
          load: DontCare,
          store: Store,
          format: self.window.get_swapchain().format(),
          samples: 1,
        }
      },
      pass: {
        color: [out_colour],
        depth_stencil: {},
        resolve: [],
      }
    ).unwrap()));
    
    self.vkpost.final_renderpass = Some(Arc::new(single_pass_renderpass!(self.window.get_device(),
      attachments: {
        out_colour: {
          load: DontCare,
          store: Store,
          format: self.window.get_swapchain().format(),
          samples: 1,
        }
      },
      pass: {
        color: [out_colour],
        depth_stencil: {},
        resolve: [],
      }
    ).unwrap()));
   
    self.vk3d.pipeline = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<model_data::Vertex>()
        .vertex_shader(vs_3d.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_3d.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .cull_mode_front()
        .render_pass(framebuffer::Subpass::from(self.main_renderpass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));

    self.vk2d.pipeline_texture = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<graphics::Vertex2d>()
        .vertex_shader(vs_texture.main_entry_point(), ())
       // .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_texture.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.main_renderpass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
        
    self.vk2d.pipeline_text = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<graphics::Vertex2d>()
        .vertex_shader(vs_text.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_text.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.main_renderpass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
        
   self.vkpost.bloom_pipeline = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<graphics::Vertex2d>()
        .vertex_shader(vs_post_bloom.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_post_bloom.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.vkpost.bloom_renderpass.clone().unwrap() , 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
        
   self.vkpost.blur_ping_pipeline = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<graphics::Vertex2d>()
        .vertex_shader(vs_post_blur.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_post_blur.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.vkpost.blur_ping_renderpass.clone().unwrap() , 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
        
   self.vkpost.blur_pong_pipeline = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<graphics::Vertex2d>()
        .vertex_shader(vs_post_blur.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_post_blur.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.vkpost.blur_pong_renderpass.clone().unwrap() , 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
        
   self.vkpost.final_pipeline = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<graphics::Vertex2d>()
        .vertex_shader(vs_post_final.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_post_final.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.vkpost.final_renderpass.clone().unwrap() , 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
    
    let main_renderpass = self.main_renderpass.clone().unwrap();
    
    self.framebuffers = Some(Arc::new({
        framebuffer::Framebuffer::start(main_renderpass.clone())
            .add(self.ms_colour_attachment.clone()).unwrap()
            .add(self.ms_depth_attachment.clone()).unwrap()
            .add(self.fullcolour_attachment.clone()).unwrap()
            .build().unwrap()
    }));
    
    let bloom_renderpass = self.vkpost.bloom_renderpass.clone().unwrap();
    self.vkpost.bloom_framebuffer = Some(Arc::new({
        framebuffer::Framebuffer::start(bloom_renderpass.clone())
                .add(self.vkpost.bloom_attachment.clone()).unwrap()
                .build().unwrap()
    }));
    
    let blur_ping_renderpass = self.vkpost.blur_ping_renderpass.clone().unwrap();
    self.vkpost.blur_ping_framebuffer = Some(Arc::new({
        framebuffer::Framebuffer::start(blur_ping_renderpass.clone())
                .add(self.vkpost.blur_ping_attachment.clone()).unwrap()
                .build().unwrap()
    }));
    
    let blur_pong_renderpass = self.vkpost.blur_pong_renderpass.clone().unwrap();
    self.vkpost.blur_pong_framebuffer = Some(Arc::new({
        framebuffer::Framebuffer::start(blur_pong_renderpass.clone())
                .add(self.vkpost.blur_pong_attachment.clone()).unwrap()
                .build().unwrap()
    }));
    
    self.vk2d.uniform_buffer_texture = cpu_pool::CpuBufferPool::<vs_texture::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    self.vk2d.uniform_buffer_text = cpu_pool::CpuBufferPool::<vs_text::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    // Terrain stuff
    let size = 800;
    let vertex_count = 128;
    
    let count = vertex_count*vertex_count;
    
    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(count);
    let mut uv: Vec<[f32; 2]> = Vec::with_capacity(count);
    let mut indices: Vec<u16> = Vec::with_capacity(6*(vertex_count-1)*(vertex_count-1));
    
    for i in 0..vertex_count {
      for j in 0..vertex_count {
        vertices.push(
          [j as f32/(vertex_count - 1) as f32 * size as f32 - size as f32*0.5,
          0.0,
          i as f32/(vertex_count - 1) as f32 * size as f32 - size as f32*0.5]);
        normals.push([0.0, 1.0, 0.0]);
        uv.push([j as f32/(vertex_count - 1) as f32,
                 i as f32/(vertex_count - 1) as f32]);
      }
    }
    
    for gz in 0..vertex_count-1 {
      for gx in 0..vertex_count-1 {
      let top_left: u16 = ((gz*vertex_count)+gx) as u16;
        let top_right: u16 = (top_left + 1) as u16;
        let bottom_left: u16 = (((gz+1)*vertex_count)+gx) as u16;
        let bottom_right: u16 = (bottom_left + 1) as u16;
        indices.push(top_left);
        indices.push(bottom_left);
        indices.push(top_right);
        indices.push(top_right);
        indices.push(bottom_left);
        indices.push(bottom_right);
      }
    }
    
    let mut vertex: Vec<model_data::Vertex> = Vec::new();
    
    for i in 0..vertices.len() {      
      vertex.push(model_data::Vertex {
          position: vertices[i],
          normal: normals[i],
          uv: uv[i],
        }
      );
    }
    
    let vert3d_buffer = self.create_vertex(vertex.iter().cloned());
    let (idx_3d_buffer, future_3d_idx) = self.create_index(indices.iter().cloned()); 
    
    let model = Model {
      vertex_buffer: Some(vec!(vert3d_buffer)),
      index_buffer: Some(idx_3d_buffer),
    };
    
    self.vk3d.models.insert(String::from("terrain"), model);
    
    self.previous_frame_end = Some(Box::new(future_3d_idx.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
  }
  
  /// Initalises some variables
  fn init(&mut self) {
   // self.framebuffers = None;
    
    self.recreate_swapchain = false;
  }
  
  /// Loads the unloaded textures returning after ~16ms has passed to allow for 
  /// updates whilst still loading
  fn dynamic_load(&mut self) {
    let time_limit = 12.0;
    
    let mut delta_time;
    let frame_start_time = time::Instant::now();
  
    let mut still_loading = false;
    //let mut to_be_removed: Vec<String> = Vec::new();
    
    let texture_paths_clone = self.texture_paths.clone();
    
    for (reference, path) in &texture_paths_clone {
      self.load_texture(reference.clone(), path.clone());
      
      self.texture_paths.remove(reference);
      still_loading = true;
      
      delta_time = frame_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
      if (delta_time*1000f64) > time_limit {
        break;
      } 
    }
    
    delta_time = frame_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    if (delta_time*1000f64) > time_limit {
      return;
    }
    
    let model_paths_clone = self.model_paths.clone();
    
    for (reference, model) in &model_paths_clone {
      self.load_model(reference.clone(), model.location.clone(), model.texture.clone());
      
      self.model_paths.remove(reference);
      still_loading = true;
      
      delta_time = frame_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
      if (delta_time*1000f64) > time_limit {
        break;
      } 
    }
    
    if !still_loading {
      self.ready = true;
    }
  }
  
  /// Clears the screen
  fn clear_screen(&mut self) {
    self.previous_frame_end.as_mut().unwrap().cleanup_finished();
  }
  
  /// Settings up drawing variables before the drawing commences
  fn pre_draw(&mut self) {
    self.clear_screen();
    
    if self.recreate_swapchain {
      let mut dimensions = {
        self.window.get_dimensions()
      };
      
      if dimensions[0] <= 0 {
        dimensions[0] = self.min_dimensions[0];
      }
      if dimensions[1] <= 0 {
        dimensions[1] = self.min_dimensions[1];
      }
      
      let (new_swapchain, new_images) = match self.window.recreate_swapchain(dimensions) {
        Ok(r) => r,
        Err(SwapchainCreationError::UnsupportedDimensions) => {
          return;
        },
        Err(err) => panic!("{:?}", err)
      };
      
      self.window.replace_swapchain(new_swapchain);
      self.window.replace_images(new_images);
      
      //self.framebuffers = None;
      self.vkpost.final_framebuffer = None;
      self.recreate_swapchain = false;
      
      let new_depth_buffer = self.create_depth_buffer();
      mem::replace(&mut self.vk3d.depth_buffer, new_depth_buffer);
      
      self.vk2d.projection = self.create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
      self.vk3d.projection = self.create_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
      
      self.ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(self.window.get_device(), dimensions, self.samples, self.window.get_swapchain().format()).unwrap();
      self.ms_depth_attachment = vkimage::AttachmentImage::transient_multisampled(self.window.get_device(), dimensions, self.samples, format::Format::D16Unorm).unwrap();
      self.fullcolour_attachment = vkimage::AttachmentImage::sampled(self.window.get_device(), dimensions, self.window.get_swapchain().format()).unwrap();
      self.vkpost.bloom_attachment = vkimage::AttachmentImage::sampled(self.window.get_device(), dimensions, self.window.get_swapchain().format()).unwrap();
      self.vkpost.blur_ping_attachment = vkimage::AttachmentImage::sampled(self.window.get_device(), dimensions, self.window.get_swapchain().format()).unwrap();
     self.vkpost.blur_pong_attachment = vkimage::AttachmentImage::sampled(self.window.get_device(), dimensions, self.window.get_swapchain().format()).unwrap();
     
      let main_renderpass = self.main_renderpass.clone().unwrap();
      self.framebuffers = Some(Arc::new({
        framebuffer::Framebuffer::start(main_renderpass.clone())
                .add(self.ms_colour_attachment.clone()).unwrap()
                .add(self.ms_depth_attachment.clone()).unwrap()
                .add(self.fullcolour_attachment.clone()).unwrap()
                .build().unwrap()
      }));
      
      let bloom_renderpass = self.vkpost.bloom_renderpass.clone().unwrap();
      self.vkpost.bloom_framebuffer = Some(Arc::new({
        framebuffer::Framebuffer::start(bloom_renderpass.clone())
                .add(self.vkpost.bloom_attachment.clone()).unwrap()
                .build().unwrap()
      }));
      
      let blur_ping_renderpass = self.vkpost.blur_ping_renderpass.clone().unwrap();
      self.vkpost.blur_ping_framebuffer = Some(Arc::new({
        framebuffer::Framebuffer::start(blur_ping_renderpass.clone())
                .add(self.vkpost.blur_ping_attachment.clone()).unwrap()
                .build().unwrap()
      }));
      
      let blur_pong_renderpass = self.vkpost.blur_pong_renderpass.clone().unwrap();
      self.vkpost.blur_pong_framebuffer = Some(Arc::new({
        framebuffer::Framebuffer::start(blur_pong_renderpass.clone())
                .add(self.vkpost.blur_pong_attachment.clone()).unwrap()
                .build().unwrap()
      }));
    }
    
    if self.vkpost.final_framebuffer.is_none() {
      let new_framebuffers = 
        Some(self.window.get_images().iter().map( |image| {
             let fb = framebuffer::Framebuffer::start(self.vkpost.final_renderpass.clone().unwrap())
                      .add(image.clone()).unwrap()
                      .build().unwrap();
             Arc::new(fb) as Arc<framebuffer::FramebufferAbstract + Send + Sync>
             }).collect::<Vec<_>>());
      mem::replace(&mut self.vkpost.final_framebuffer, new_framebuffers);
    }
  }
  
  /// Draws everything that is in the drawcall passed to this function
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
    if self.recreate_swapchain == true {
      return;
    }
    
    let (image_num, acquire_future) = match swapchain::acquire_next_image(self.window.get_swapchain(), None) {
      Ok(r) => r,
      Err(AcquireError::OutOfDate) => {
        self.recreate_swapchain = true;
        return;
      },
      Err(err) => panic!("{:?}", err)
    };
    
    let dimensions = {
      self.window.get_dimensions()
    };
    
    let command_buffer: AutoCommandBuffer = {
      let mut tmp_cmd_buffer = AutoCommandBufferBuilder::primary_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family()).unwrap();
      
      let build_start = tmp_cmd_buffer;
      
      let clear = [self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w];
      tmp_cmd_buffer = build_start.begin_render_pass(self.framebuffers.as_ref().unwrap().clone(), false, vec![ClearValue::Float(clear.into()), ClearValue::Depth(1.0), ClearValue::None]).unwrap();
      for draw in draw_calls {
        
        if draw.is_3d_model() {
          
          let uniform_buffer_subbuffer = self.create_3d_subbuffer(draw.clone());
          
          let mut texture: String = String::from(graphics::DEFAULT_TEXTURE);
          if self.textures.contains_key(draw.get_texture()) {
            texture = draw.get_texture().clone();
          }
          if draw.get_texture() == "terrain" {
            texture = String::from("oakfloor");
          }
          
          if !self.textures.contains_key(&texture.clone()) {
            println!("Error: Model doesn't exist {}", texture.clone());
            continue;
          }
          
          let set_3d = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.vk3d.pipeline.clone().unwrap(), 0)
                .add_buffer(uniform_buffer_subbuffer).unwrap()
                .add_sampled_image(self.textures.get(&texture).unwrap().clone(), self.sampler.clone()).unwrap()
                .build().unwrap()
          );
          
          {
            let cb = tmp_cmd_buffer;

            tmp_cmd_buffer = cb.draw_indexed(
                  self.vk3d.pipeline.clone().unwrap(),
                  DynamicState {
                        line_width: None,
                        viewports: Some(vec![pipeline::viewport::Viewport {
                            origin: [0.0, 0.0],
                            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                            depth_range: 0.0 .. 1.0,
                        }]),
                        scissors: None,
                  },
                  self.vk3d.models.get(draw.get_texture()).expect("Invalid model name").vertex_buffer.clone().unwrap(),
                  self.vk3d.models.get(draw.get_texture()).expect("Invalid model name").index_buffer.clone().unwrap(), set_3d.clone(), ()).unwrap();
          }
        } else {
          if draw.is_vao_update() {
            let reference = draw.get_text().clone();
            
            if self.vk2d.custom_dynamic_vao.contains_key(&reference) {
              let mut verts = draw.get_new_vertices();
              let mut index = draw.get_new_indices();
              
              let new_model = self.create_dynamic_custom_2d_model(verts, index);
              
              if let Some(d_model) = self.vk2d.custom_dynamic_vao.get_mut(&reference) {
                *d_model = new_model;
              }
              
            } else {
              println!("Error: Dynamic vao update doesn't exist: {:?}", reference);
              continue;
            }
            
          } else if draw.is_text() {// Render Text
            let wrapped_draw = DrawMath::setup_correct_wrapping(draw.clone(), self.fonts.clone());
            let size = draw.get_x_size();
            
            if !self.fonts.contains_key(draw.get_texture()) || !self.textures.contains_key(draw.get_texture()) {
              println!("Error: text couldn't draw, Texture: {:?}", draw.get_texture());
              continue;
            }
            
            let pipeline = self.vk2d.pipeline_text.clone().unwrap();
            let vertex_buffer = self.vk2d
                                    .vao.vertex_buffer.clone()
                                    .expect("Error: Unwrapping text vertex buffer failed!");
            let index_buffer = self.vk2d
                                    .vao.index_buffer.clone()
                                    .expect("Error: Unwrapping text index buffer failed!");
            
            for letter in wrapped_draw {
              let char_letter = {
                letter.get_text().as_bytes()[0] 
              };
              
              let c = self.fonts.get(draw.get_texture()).unwrap().get_character(char_letter as i32);

              let model = DrawMath::calculate_text_model(letter.get_translation(), size, &c.clone(), char_letter);
              let letter_uv = DrawMath::calculate_text_uv(&c.clone());
              let colour = letter.get_colour();
              let outline = letter.get_outline_colour();
              let edge_width = letter.get_edge_width(); 
               
              let uniform_buffer_text_subbuffer = {
                let uniform_data = vs_text::ty::Data {
                  outlineColour: outline.into(),
                  colour: colour.into(),
                  edge_width: edge_width.into(),
                  letter_uv: letter_uv.into(),
                  model: model.into(),
                  projection: self.vk2d.projection.into(),
                };
                self.vk2d.uniform_buffer_text.next(uniform_data).unwrap()
              };
              
              let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.vk2d.pipeline_text.clone().unwrap(), 0)
                                         .add_sampled_image(self.textures.get(draw.get_texture()).unwrap().clone(), self.sampler.clone()).unwrap()
                                         .add_buffer(uniform_buffer_text_subbuffer.clone()).unwrap()
                                         .build().unwrap());
              
              {
                let cb = tmp_cmd_buffer;
                tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline.clone(), vertex_buffer.clone(), index_buffer.clone(), uniform_set);
              }
            }
          } else {
            let uniform_buffer_texture_subbuffer = self.create_texture_subbuffer(draw.clone());
            
            // No Texture
            if draw.get_texture() == &String::from("") {
              let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.vk2d.pipeline_texture.clone().unwrap(), 0)
                                         .add_sampled_image(self.textures.get(graphics::DEFAULT_TEXTURE).expect("Default texture not loaded!").clone(), self.sampler.clone()).unwrap()
                                         .add_buffer(uniform_buffer_texture_subbuffer.clone()).unwrap()
                                         .build().unwrap());
              
              {
                let cb = tmp_cmd_buffer;
                
                let pipeline = self.vk2d.pipeline_texture.clone().unwrap();
                if draw.is_custom_vao() {
                  if self.vk2d.custom_vao.contains_key(draw.get_text()) {
                    let vertex_buffer = self.vk2d
                                        .custom_vao.get(draw.get_text()).unwrap()
                                        .vertex_buffer.clone()
                                        .expect("Error: Unwrapping static custom vertex buffer failed!");
                    let index_buffer = self.vk2d
                                        .custom_vao.get(draw.get_text()).unwrap()
                                        .index_buffer.clone()
                                        .expect("Error: Unwrapping static custom index buffer failed!");
                    
                    tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
                  } else if self.vk2d.custom_dynamic_vao.contains_key(draw.get_text()) {
                    let vertex_buffer = self.vk2d
                                        .custom_dynamic_vao.get(draw.get_text()).unwrap()
                                        .vertex_buffer.clone()
                                        .expect("Error: Unwrapping static custom vertex buffer failed!");
                    let index_buffer = self.vk2d
                                        .custom_dynamic_vao.get(draw.get_text()).unwrap()
                                        .index_buffer.clone()
                                        .expect("Error: Unwrapping static custom index buffer failed!");
                    
                    tmp_cmd_buffer = draw_dynamic(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
                  } else {
                    println!("Error: custom vao {:?} does not exist!", draw.get_text());
                    tmp_cmd_buffer = cb;
                    continue;
                  }
                } else {
                  let vertex_buffer = self.vk2d
                                          .vao.vertex_buffer.clone()
                                          .expect("Error: Unwrapping main vertex buffer failed!");
                  let index_buffer = self.vk2d
                                          .vao.index_buffer.clone()
                                         .expect("Error: Unwrapping main index buffer failed!");
                  tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
                }
              }
            } else {
              // Texture
              let default_texture = String::from(graphics::DEFAULT_TEXTURE);
              let mut texture = draw.get_texture(); 
              
              if !self.textures.contains_key(texture) {
                println!("Texture not found: {}", texture);
                texture = &default_texture;
              }
              
              let pipeline = self.vk2d.pipeline_texture.clone().unwrap();
              let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                                      .add_sampled_image(self.textures.get(texture).unwrap().clone(), self.sampler.clone()).unwrap()
                                      .add_buffer(uniform_buffer_texture_subbuffer.clone()).unwrap()
                                      .build().unwrap());
              
              {
                let cb = tmp_cmd_buffer;
                
                if draw.is_custom_vao() {
                  if self.vk2d.custom_vao.contains_key(draw.get_text()) {
                    let vertex_buffer = self.vk2d
                                            .custom_vao.get(draw.get_text()).unwrap()
                                            .vertex_buffer.clone()
                                            .expect("Error: Unwrapping static custom vertex buffer failed!");
                    let index_buffer = self.vk2d
                                           .custom_vao.get(draw.get_text()).unwrap()
                                           .index_buffer.clone()
                                           .expect("Error: Unwrapping static custom index buffer failed!");
                    
                    tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
                  } else if self.vk2d.custom_dynamic_vao.contains_key(draw.get_text()) {
                    let vertex_buffer = self.vk2d
                                        .custom_dynamic_vao.get(draw.get_text()).unwrap()
                                        .vertex_buffer.clone()
                                        .expect("Error: Unwrapping static custom vertex buffer failed!");
                    let index_buffer = self.vk2d
                                        .custom_dynamic_vao.get(draw.get_text()).unwrap()
                                        .index_buffer.clone()
                                        .expect("Error: Unwrapping static custom index buffer failed!");
                    
                    tmp_cmd_buffer = draw_dynamic(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
                  } else {
                    println!("Error: custom vao {:?} does not exist!", draw.get_text());
                    tmp_cmd_buffer = cb;
                    continue;
                  }
                } else {
                  let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
                  let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
                  tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
                }
              }
            }
          }
        }
      }
      
      /*
      * Bloom framebuffer
      */
      let build_start = tmp_cmd_buffer.end_render_pass().unwrap();
      let cb = build_start.begin_render_pass(self.vkpost.bloom_framebuffer.as_ref().unwrap().clone(), false, vec![ClearValue::None]).unwrap();
      
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.bloom_pipeline.clone().unwrap();
      
      let model = DrawMath::calculate_texture_model(Vector3::new(dimensions[0] as f32 * 0.5, dimensions[1] as f32 * 0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
      
      let uniform_data = vs_post_bloom::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
      };
      
      let uniform_subbuffer = self.vkpost.bloom_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.fullcolour_attachment.clone(), self.sampler.clone()).unwrap()
                             .build().unwrap());
                          
      tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
      
      
      /*
      * Blur ping framebuffer
      */
      let build_start = tmp_cmd_buffer.end_render_pass().unwrap();
      let cb = build_start.begin_render_pass(self.vkpost.blur_ping_framebuffer.as_ref().unwrap().clone(), false, vec![ClearValue::None]).unwrap();
      /*
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.blur_ping_pipeline.clone().unwrap();
      
      let model = DrawMath::calculate_texture_model(Vector3::new(dimensions[0] as f32 * 0.5, dimensions[1] as f32 * 0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
      
      let uniform_data = vs_post_blur::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        direction: Vector2::new(1.0, 0.0).into(),
      };
      
      let uniform_subbuffer = self.vkpost.blur_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.vkpost.bloom_attachment.clone(), self.sampler.clone()).unwrap()
                             .build().unwrap());
                          
      tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline.clone(), vertex_buffer.clone(), index_buffer.clone(), uniform_set);*/
      tmp_cmd_buffer = cb;
      
      /*
      * Blur pong framebuffer
      */
      let build_start = tmp_cmd_buffer.end_render_pass().unwrap();
      let cb = build_start.begin_render_pass(self.vkpost.blur_pong_framebuffer.as_ref().unwrap().clone(), false, vec![ClearValue::None]).unwrap();
      /*
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.blur_pong_pipeline.clone().unwrap();
      
      let model = DrawMath::calculate_texture_model(Vector3::new(dimensions[0] as f32 * 0.5, dimensions[1] as f32 * 0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
      
      let uniform_data = vs_post_blur::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        direction: Vector2::new(0.0, 1.0).into(),
      };
      
      let uniform_subbuffer = self.vkpost.blur_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.vkpost.blur_ping_attachment.clone(), self.sampler.clone()).unwrap()
                             .build().unwrap());
                          
      tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
      */
      
      tmp_cmd_buffer = cb;
      
      /*
      * Final Framebuffer
      */
      let build_start = tmp_cmd_buffer.end_render_pass().unwrap();
      
      let clear = [self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w];
      let cb = build_start.begin_render_pass(self.vkpost.final_framebuffer.as_ref().unwrap()[image_num].clone(), false, vec![ClearValue::None]).unwrap();
      
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.final_pipeline.clone().unwrap();
      
      let model = DrawMath::calculate_texture_model(Vector3::new(dimensions[0] as f32*0.5, dimensions[1] as f32*0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
      
      let uniform_data = vs_post_final::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        bloom: 0.0,
      };
      
      let uniform_subbuffer = self.vkpost.final_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.fullcolour_attachment.clone(), self.sampler.clone()).unwrap()
                             .add_sampled_image(self.vkpost.blur_ping_attachment.clone(), self.sampler.clone()).unwrap()
                             .build().unwrap());
      
      tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline.clone(), vertex_buffer.clone(), index_buffer.clone(), uniform_set);
     
      let cb = tmp_cmd_buffer;
      
      let model = DrawMath::calculate_texture_model(Vector3::new(dimensions[0] as f32 * 0.25, dimensions[1] as f32 * 0.25, 0.0), Vector2::new(dimensions[0] as f32 * 0.125, dimensions[1] as f32 * 0.125), 90.0);
      
      let uniform_data = vs_post_final::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        bloom: 0.0,
      };
      
      let uniform_subbuffer = self.vkpost.final_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.vkpost.blur_ping_attachment.clone(), self.sampler.clone()).unwrap()
                             .add_sampled_image(self.vkpost.blur_ping_attachment.clone(), self.sampler.clone()).unwrap()
                             .build().unwrap());
      tmp_cmd_buffer = draw_immutable(cb, dimensions, pipeline.clone(), vertex_buffer.clone(), index_buffer.clone(), uniform_set);
      
      tmp_cmd_buffer.end_render_pass()
        .unwrap()
        .build().unwrap() as AutoCommandBuffer
    };
  
    let future = self.previous_frame_end.take().unwrap().join(acquire_future)
      .then_execute(self.window.get_queue(), command_buffer).expect("future")
      .then_swapchain_present(self.window.get_queue(), self.window.get_swapchain(), image_num)
      .then_signal_fence_and_flush();
    
    match future {
      Ok(future) => {
        self.previous_frame_end = Some(Box::new(future) as Box<_>);
      }
      Err(FlushError::OutOfDate) => {
        self.recreate_swapchain = true;
        self.previous_frame_end = Some(Box::new(now(self.window.get_device())) as Box<_>);
      }
      Err(e) => {
        println!("{:?}", e);
        self.previous_frame_end = Some(Box::new(now(self.window.get_device())) as Box<_>);
      }
    }
  }
  
  /// Tells engine it needs to update as window resize has occured
  fn screen_resized(&mut self) {
    self.recreate_swapchain = true;
  }
  
  /// Returns the dimensions of the drawing window as u32
  fn get_dimensions(&self) -> [u32; 2] {
    let dimensions: [u32; 2] = self.window.get_dimensions();
    dimensions
  }
  
  /// Returns a reference to the events loop
  fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  /// Returns all loaded fonts in a HashMap
  fn get_fonts(&self) -> HashMap<String, GenericFont> {
    self.fonts.clone()
  }
  
  /// Returns the current dpi scale factor
  ///
  /// Needed to solve issues with Hidpi monitors
  fn get_dpi_scale(&self) -> f32 {
    self.window.get_dpi_scale()
  }
  
  /// Queries if the engine has loaded all textures and models
  fn is_ready(&self) -> bool {
    self.ready
  }
  
  /// Enables the cursor to be drawn whilst over the window
  fn show_cursor(&mut self) {
    self.window.show_cursor();
  }
  
  /// Disables the cursor from being drawn whilst over the window
  fn hide_cursor(&mut self) {
    self.window.hide_cursor();
  }
  
  /// Sets camera location based on a position Vector3 and rotation in the x 
  /// and y axis with a Vector2
  fn set_camera_location(&mut self, camera: Vector3<f32>, camera_rot: Vector2<f32>) {

    //let (x_rot, z_rot) = DrawMath::calculate_y_rotation(camera_rot.y);
    let (x_rot, z_rot) = DrawMath::rotate(camera_rot.y);
    
    self.vk3d.view = cgmath::Matrix4::look_at(cgmath::Point3::new(camera.x, camera.y, camera.z), cgmath::Point3::new(camera.x+x_rot, camera.y, camera.z+z_rot), cgmath::Vector3::new(0.0, -1.0, 0.0));
  }
  
  // Sets the clear colour for the window to display if nothing is draw in an area
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
    self.clear_colour = Vector4::new(r,g,b,a);
  }
  
  /// Sets the camera location, rotation, view given a Camera object
  fn set_camera(&mut self, camera: Camera) {}
  
  /// does nothing in vulkan
  fn post_draw(&self) {}
  /// does nothing in vulkan
  fn clean(&self) {}
  /// does nothing
  fn swap_buffers(&mut self) {}
}


