use font::GenericFont;
use window::VkWindow;
use drawcalls;
use drawcalls::DrawCall;
use math;
use graphics::DEFAULT_TEXTURE;
use graphics::Vertex2d;
use graphics::Vertex3d;
use graphics::CoreRender;
use settings::Settings;
use camera::Camera;
use gltf_interpreter::ModelDetails;
use helperfunctions::convert_to_vertex3d;
use vulkan::vulkan_2d;
use vulkan::vulkan_3d;
use vulkan::vulkan_draw;
use vulkan::vulkan_helper;
use vulkan::renderpass::CustomRenderpass;

use image;
use winit;
use winit::dpi::LogicalSize;

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
use vulkano::image::ImageUsage;

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
use vulkano::descriptor::descriptor_set::DescriptorSet;
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

impl_vertex!(Vertex2d, position, uv);
impl_vertex!(Vertex3d, position, normal, uv);

pub const blur_dim: u32 = 512;

pub mod vs_texture {
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

pub mod vs_text {
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

pub mod vs_3d {
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

#[derive(Clone)]
pub struct ModelInfo {
  directory: String,
  model_name: String,
}

pub struct Model {
  pub vertex_buffer: Option<Vec<Arc<BufferAccess + Send + Sync>>>,
  pub index_buffer: Option<Arc<ImmutableBuffer<[u32]>>>,
}

pub struct DynamicModel {
  pub vertex_buffer: Option<Vec<Arc<BufferAccess + Send + Sync>>>,
  pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u32]>>>
}

pub struct Mesh {
  pub vertex_buffer: Option<Vec<Arc<BufferAccess + Send + Sync>>>,
  pub index_buffer: Option<Arc<ImmutableBuffer<[u32]>>>,
  pub material_desctriptor: Arc<DescriptorSet + Send + Sync>,
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
  
  sampler: Arc<sampler::Sampler>,
}

pub struct VK3D {
  depth_buffer: Option<Arc<vkimage::AttachmentImage<format::D16Unorm>>>,
  
  models: HashMap<String, Vec<Mesh>>,
  
  pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  
  camera: Camera,
  
  projection: Matrix4<f32>,
  scale: Matrix4<f32>,
  
  uniform_buffer: cpu_pool::CpuBufferPool<vs_3d::ty::Data>,
}

pub struct VKPOST {
  bloom_renderpass: CustomRenderpass,
  bloom_uniformbuffer: cpu_pool::CpuBufferPool<vs_post_bloom::ty::Data>,
  
  blur_ping_renderpass: CustomRenderpass,
  blur_pong_renderpass: CustomRenderpass,
  blur_uniformbuffer: cpu_pool::CpuBufferPool<vs_post_blur::ty::Data>,
  
  blur_upscale_attachment: Arc<vkimage::StorageImage<format::R8G8B8A8Unorm>>,
  blur_downscale_attachment: Arc<vkimage::StorageImage<format::R8G8B8A8Unorm>>,
  
  final_renderpass: Option<Arc<RenderPassAbstract + Send + Sync>>,
  final_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  final_framebuffer: Option<Vec<Arc<framebuffer::FramebufferAbstract + Send + Sync>>>,
  final_uniformbuffer: cpu_pool::CpuBufferPool<vs_post_final::ty::Data>,
}

pub struct RawVk {
  ready: bool,
  fonts: HashMap<String, GenericFont>,
  textures: HashMap<String, Arc<ImmutableImage<format::R8G8B8A8Unorm>>>,
  diffuse_colours: HashMap<String, [f32; 3]>,
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
    let width = settings.get_resolution()[0] as f64;
    let height = settings.get_resolution()[1] as f64;
    let min_width = settings.get_minimum_resolution()[0];
    let min_height = settings.get_minimum_resolution()[1];
    let fullscreen = settings.is_fullscreen();
    
    let window = VkWindow::new(width, height, min_width, min_height, fullscreen);
    
    let proj_2d = Matrix4::identity();
    let proj_3d = Matrix4::identity();
    
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
    
    let src_usage = ImageUsage {
        transfer_source: true,
        sampled: true,
        .. ImageUsage::none()
    };
    
    let dst_usage = ImageUsage {
        transfer_destination: true,
        sampled: true,
        .. ImageUsage::none()
    };
    
    let dim = window.get_dimensions();
    let dim = [dim.width as u32, dim.height as u32];
    let fullcolour_attachment = vkimage::AttachmentImage::sampled(window.get_device(), dim, format::Format::R16G16B16A16Unorm).unwrap();
    let bloom_attachment = vkimage::AttachmentImage::with_usage(window.get_device(), dim, format::Format::R16G16B16A16Unorm, src_usage).unwrap();
    
    let blur_ping_attachment = vkimage::AttachmentImage::sampled(window.get_device(), [blur_dim, blur_dim], window.get_swapchain().format()).unwrap();
    let blur_pong_attachment = vkimage::AttachmentImage::with_usage(window.get_device(), [blur_dim, blur_dim], window.get_swapchain().format(), src_usage).unwrap();
    
    let ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(window.get_device(), dim, samples, format::Format::R16G16B16A16Unorm).unwrap();
    let ms_depth_attachment = vkimage::AttachmentImage::transient_multisampled(window.get_device(), dim, samples, format::Format::D16Unorm).unwrap();
    
    let blur_downscale_attachment = vkimage::StorageImage::with_usage(window.get_device(), vkimage::Dimensions::Dim2d { width: blur_dim, height: blur_dim}, format::R8G8B8A8Unorm, dst_usage, window.get_queue_ref().family().physical_device().queue_families().find(|&q| {
          q.supports_graphics()
        }
      )).unwrap();
    
    let blur_upscale_attachment = vkimage::StorageImage::with_usage(window.get_device(), vkimage::Dimensions::Dim2d { width: dim[0], height: dim[1]}, format::R8G8B8A8Unorm, dst_usage, window.get_queue_ref().family().physical_device().queue_families().find(|&q| {
          q.supports_graphics()
        }
      )).unwrap();
    
    RawVk {
      ready: false,
      fonts: HashMap::new(),
      textures: HashMap::new(),
      diffuse_colours: HashMap::new(),
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
        
        camera: Camera::default_vk(),
        
        projection: proj_3d,
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
        
        sampler: sampler,
      },
      
      // Post Processing
      vkpost: VKPOST {
        bloom_renderpass: CustomRenderpass::new(bloom_attachment),
        bloom_uniformbuffer: post_bloom_uniform,
        
        blur_uniformbuffer: post_blur_uniform,
        blur_ping_renderpass: CustomRenderpass::new(blur_ping_attachment),
        blur_pong_renderpass: CustomRenderpass::new(blur_pong_attachment),
        
        blur_upscale_attachment: blur_upscale_attachment,
        blur_downscale_attachment: blur_downscale_attachment,
        
        final_renderpass: None,
        final_pipeline: None,
        final_framebuffer: None,
        final_uniformbuffer: post_final_uniform,
      },
      
      // Vk System
      min_dimensions: [min_width, min_height],
      window: window,
      
      recreate_swapchain: false,
      
      previous_frame_end: previous_frame_end,
    }
  }
  
  pub fn with_title(mut self, title: String) -> RawVk {
    self.window.set_title(title);
    self
  }
}

impl CoreRender for RawVk {
  fn load_instanced(&mut self, reference: String, max_instances: i32) {
    
  }
  
  fn load_static_geometry(&mut self, reference: String, vertices: Vec<Vertex2d>, indices: Vec<u32>) {
    let (model, future) = vulkan_2d::create_static_custom_model(self.window.get_device(), self.window.get_queue(), vertices, indices);
    self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    self.vk2d.custom_vao.insert(reference, model);
  }
  
  fn load_dynamic_geometry(&mut self, reference: String, vertices: Vec<Vertex2d>, indices: Vec<u32>) {
    let model = vulkan_2d::create_dynamic_custom_model(self.window.get_device(), vertices, indices);
    self.vk2d.custom_dynamic_vao.insert(reference, model);
  }
  
  fn preload_model(&mut self, reference: String, directory: String, texture: String) {
    self.load_model(reference.clone(), directory, texture.clone());
  }
  
  fn add_model(&mut self, reference: String, directory: String, model_name: String) {
    self.model_paths.insert(reference.clone(), ModelInfo {directory: directory.clone(), model_name: model_name.clone()});
  }
  
  fn load_model(&mut self, reference: String, directory: String, model_name: String) {
    let start_time = time::Instant::now();
    
    let mesh_data = ModelDetails::new(directory.clone()+&model_name.clone());
    
    let mut mesh: Vec<Mesh> = Vec::new();
    
    let params_buffer = cpu_pool::CpuBufferPool::new(self.window.get_device().clone(), BufferUsage::uniform_buffer());
    let material_params = params_buffer.next(fs_3d::ty::MaterialParams {
      base_colour_factor: [0.0, 0.0, 0.0, 0.0],
      base_colour_texture_tex_coord: -1,
      metallic_factor: 0.0,
      roughness_factor: 0.0,
      metallic_roughness_texture_tex_coord: -1,
      normal_texture_scale: 0.0,
      normal_texture_tex_coord: -1,
      occlusion_texture_strength: 0.0,
      occlusion_texture_tex_coord: -1,
      emissive_factor: [0.0, 0.0, 0.0],
      emissive_texture_tex_coord: -1,
      _dummy0: [0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8],
    }).unwrap();
       
    let sampler = sampler::Sampler::new(self.window.get_device(), sampler::Filter::Linear,
                                        sampler::Filter::Linear, 
                                        sampler::MipmapMode::Nearest,
                                        sampler::SamplerAddressMode::ClampToEdge,
                                        sampler::SamplerAddressMode::ClampToEdge,
                                        sampler::SamplerAddressMode::ClampToEdge,
                                        0.0, 1.0, 0.0, 0.0).unwrap();
    
    let (temp_tex, _) = vkimage::immutable::ImmutableImage::from_iter([0u8, 0u8, 0u8, 0u8].iter().cloned(),
                                            vkimage::Dimensions::Dim2d { width: 1, height: 1 },
                                            format::R8G8B8A8Unorm, self.window.get_queue())
                                            .expect("Failed to create immutable image");
    
    let default_descriptor_set =
      Arc::new(descriptor_set::PersistentDescriptorSet::start(self.vk3d.pipeline.clone().unwrap(), 1)
              .add_buffer(material_params)
              .unwrap()
              .add_sampled_image(temp_tex.clone(), sampler.clone())
              .unwrap()
              .add_sampled_image(temp_tex.clone(), sampler.clone())
              .unwrap()
              .add_sampled_image(temp_tex.clone(), sampler.clone())
              .unwrap()
              .add_sampled_image(temp_tex.clone(), sampler.clone())
              .unwrap()
              .add_sampled_image(temp_tex.clone(), sampler.clone())
              .unwrap()
              .build().unwrap());
    
    for i in 0..mesh_data.num_models() {
      mesh.push(Mesh {
        vertex_buffer: None,
        index_buffer: None,
        material_desctriptor: default_descriptor_set.clone(),
      });
      
      let vertex = mesh_data.vertex(i);
      let texcoord = mesh_data.texcoords(i);
      let normal = mesh_data.normal(i);
      let index = mesh_data.index(i);
      let mut base_colour_texture = (temp_tex.clone(), -1);
      let mut metallic_roughness_texture = (temp_tex.clone(), -1);
      let mut normal_texture = (temp_tex.clone(), -1);
      let mut occlusion_texture = (temp_tex.clone(), -1);
      let mut emissive_texture = (temp_tex.clone(), -1);
      
      
      let mut vertices: Vec<Vertex3d> = Vec::with_capacity(vertex.len());
      for j in 0..vertex.len() {
        let mut uv = [1.0, 1.0];
        if texcoord.len() > j {
          uv = texcoord[j];
        }
        vertices.push(Vertex3d { position: vertex[j], normal: normal[j], uv: uv });
      }
      
      let (idx_3d_buffer, future_3d_idx) = vulkan_3d::create_index(self.window.get_queue(), index.iter().cloned());
      mesh[i].vertex_buffer = Some(vec!(vulkan_3d::create_vertex(self.window.get_device(), vertices.iter().cloned())));
      mesh[i].index_buffer = Some(idx_3d_buffer);
            
      self.previous_frame_end = Some(Box::new(future_3d_idx.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
      
      let base_colour = mesh_data.base_colour(i);
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.base_colour_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);

        base_colour_texture = (texture, 0);
      }
      
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.metallic_roughness_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);

        metallic_roughness_texture = (texture, 0);
      }
      
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.normal_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);

        normal_texture = (texture, 0);
      }
      
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.occlusion_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);

        occlusion_texture = (texture, 0);
      }
      
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.emissive_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);

        emissive_texture = (texture, 0);
      }
      
     // let params_buffer = cpu_pool::CpuBufferPool::new(self.window.get_device().clone(), BufferUsage::uniform_buffer());
      let material_params = params_buffer.next(fs_3d::ty::MaterialParams {
         base_colour_factor: mesh_data.base_colour(i),
         base_colour_texture_tex_coord: base_colour_texture.1,
         metallic_factor: mesh_data.metallic_factor(i),
         roughness_factor: mesh_data.roughness_factor(i),
         metallic_roughness_texture_tex_coord: metallic_roughness_texture.1,
         normal_texture_scale: mesh_data.normal_texture_scale(i),
         normal_texture_tex_coord: normal_texture.1,
         occlusion_texture_strength: mesh_data.occlusion_texture_strength(i),
         occlusion_texture_tex_coord: occlusion_texture.1,
         emissive_factor: mesh_data.emissive_factor(i),
         emissive_texture_tex_coord: emissive_texture.1,
         _dummy0: [0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8],
       }).unwrap();
       /*
       let sampler = sampler::Sampler::new(self.window.get_device(), sampler::Filter::Linear,
                                           sampler::Filter::Linear, 
                                           sampler::MipmapMode::Nearest,
                                           sampler::SamplerAddressMode::ClampToEdge,
                                           sampler::SamplerAddressMode::ClampToEdge,
                                           sampler::SamplerAddressMode::ClampToEdge,
                                           0.0, 1.0, 0.0, 0.0).unwrap();*/
       
       
       let descriptor_set =
         Arc::new(descriptor_set::PersistentDescriptorSet::start(self.vk3d.pipeline.clone().unwrap(), 1)
              .add_buffer(material_params)
              .unwrap()
              .add_sampled_image(base_colour_texture.0, sampler.clone())
              .unwrap()
              .add_sampled_image(metallic_roughness_texture.0, sampler.clone())
              .unwrap()
              .add_sampled_image(normal_texture.0, sampler.clone())
              .unwrap()
              .add_sampled_image(occlusion_texture.0, sampler.clone())
              .unwrap()
              .add_sampled_image(emissive_texture.0, sampler.clone())
              .unwrap()
              .build().unwrap());
              
        mesh[i].material_desctriptor = descriptor_set;
    }
    
    self.vk3d.models.insert(reference, mesh);
    
    let total_time = start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (total_time*1000f64) as f32, directory+&model_name);
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
      let dim = self.window.get_dimensions();
      [dim.width as u32, dim.height as u32]
    };
    
    self.vk2d.projection = vulkan_2d::create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
    self.vk3d.projection = vulkan_3d::create_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
    
    let device = self.window.get_device();
    self.vk3d.depth_buffer = vulkan_3d::create_depth_buffer(device, dimensions);
    
    // 2D
    let vert_buffer = vulkan_2d::create_vertex(self.window.get_device());
    let (idx_buffer, future_idx) = vulkan_2d::create_index(self.window.get_queue());
    
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
          format: format::Format::R16G16B16A16Unorm,
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
          format: format::Format::R16G16B16A16Unorm,
          samples: 1,
        }
      },
      pass: {
        color: [multisample_colour],
        depth_stencil: {ms_depth_attachment},
        resolve: [resolve_fullcolour],
      }
    ).unwrap()));
    
    let bloom_renderpass = Arc::new(single_pass_renderpass!(self.window.get_device(),
      attachments: {
        out_colour: {
          load: DontCare,
          store: Store,
          format: format::Format::R16G16B16A16Unorm,
          samples: 1,
        }
      },
      pass: {
        color: [out_colour],
        depth_stencil: {},
        resolve: [],
      }
    ).unwrap());
    
    let blur_ping_renderpass = Arc::new(single_pass_renderpass!(self.window.get_device(),
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
    ).unwrap());
    
    let blur_pong_renderpass = Arc::new(single_pass_renderpass!(self.window.get_device(),
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
    ).unwrap());
    
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
        .vertex_input_single_buffer::<Vertex3d>()
        .vertex_shader(vs_3d.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_3d.main_entry_point(), ())
        .depth_clamp(true)
        .depth_stencil_simple_depth()
       // .cull_mode_front()
        .render_pass(framebuffer::Subpass::from(self.main_renderpass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));

    self.vk2d.pipeline_texture = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_texture.main_entry_point(), ())
       // .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_texture.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.main_renderpass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
        
    self.vk2d.pipeline_text = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_text.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_text.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.main_renderpass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
    
    let bloom_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_post_bloom.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_post_bloom.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(bloom_renderpass.clone(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap());
    
    let blur_ping_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_post_blur.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_post_blur.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(blur_ping_renderpass.clone() , 0).unwrap())
        .build(self.window.get_device())
        .unwrap());
        
    let blur_pong_pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_post_blur.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_post_blur.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(blur_pong_renderpass.clone() , 0).unwrap())
        .build(self.window.get_device())
        .unwrap());
        
    self.vkpost.final_pipeline = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
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
    
    let bloom_attachment = self.vkpost.bloom_renderpass.attachment();
    self.vkpost.bloom_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(bloom_renderpass.clone())
                .add(bloom_attachment).unwrap()
                .build().unwrap()
      }));
    
    let blur_ping_attachment = self.vkpost.blur_ping_renderpass.attachment();
    self.vkpost.blur_ping_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(blur_ping_renderpass.clone())
                .add(blur_ping_attachment).unwrap()
                .build().unwrap()
    }));
    
    let blur_pong_attachment = self.vkpost.blur_pong_renderpass.attachment();
    self.vkpost.blur_pong_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(blur_pong_renderpass.clone())
                .add(blur_pong_attachment).unwrap()
                .build().unwrap()
    }));
    
    self.vkpost.bloom_renderpass.set_renderpass(bloom_renderpass);
    self.vkpost.bloom_renderpass.set_pipeline(bloom_pipeline);
    self.vkpost.blur_ping_renderpass.set_renderpass(blur_ping_renderpass);
    self.vkpost.blur_ping_renderpass.set_pipeline(blur_ping_pipeline);
    self.vkpost.blur_pong_renderpass.set_renderpass(blur_pong_renderpass);
    self.vkpost.blur_pong_renderpass.set_pipeline(blur_pong_pipeline);
    
    self.vk2d.uniform_buffer_texture = cpu_pool::CpuBufferPool::<vs_texture::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    self.vk2d.uniform_buffer_text = cpu_pool::CpuBufferPool::<vs_text::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    // Terrain stuff
    let size = 800;
    let vertex_count = 128;
    
    let count = vertex_count*vertex_count;
    
    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(count);
    let mut uv: Vec<[f32; 2]> = Vec::with_capacity(count);
    let mut indices: Vec<u32> = Vec::with_capacity(6*(vertex_count-1)*(vertex_count-1));
    
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
      let top_left: u32 = ((gz*vertex_count)+gx) as u32;
        let top_right: u32 = (top_left + 1) as u32;
        let bottom_left: u32 = (((gz+1)*vertex_count)+gx) as u32;
        let bottom_right: u32 = (bottom_left + 1) as u32;
        indices.push(top_left);
        indices.push(bottom_left);
        indices.push(top_right);
        indices.push(top_right);
        indices.push(bottom_left);
        indices.push(bottom_right);
      }
    }
    
    let mut vertex: Vec<Vertex3d> = Vec::new();
    
    for i in 0..vertices.len() {
      vertex.push(Vertex3d {
          position: vertices[i],
          normal: normals[i],
          uv: uv[i],
        }
      );
    }
    
    let vert3d_buffer = vulkan_3d::create_vertex(self.window.get_device(), vertex.iter().cloned());
    let (idx_3d_buffer, future_3d_idx) = vulkan_3d::create_index(self.window.get_queue(), indices.iter().cloned()); 
    /*
    let model = Mesh {
      vertex_buffer: Some(vec!(vert3d_buffer)),
      index_buffer: Some(idx_3d_buffer),
      material_desctriptor: None,
    };*/
    
   // self.vk3d.models.insert(String::from("terrain"), vec!(model));
    
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
      self.load_model(reference.clone(), model.directory.clone(), model.model_name.clone());
      
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
        let dim = self.window.get_dimensions();
        [dim.width as u32, dim.height as u32]
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
      
      let device = self.window.get_device();
      let new_depth_buffer = vulkan_3d::create_depth_buffer(device, dimensions);
      mem::replace(&mut self.vk3d.depth_buffer, new_depth_buffer);
      
      self.vk2d.projection = vulkan_2d::create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
      self.vk3d.projection = vulkan_3d::create_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
      
      let src_usage = ImageUsage {
          transfer_source: true,
          sampled: true,
          .. ImageUsage::none()
      };
      
      let dst_usage = ImageUsage {
          transfer_destination: true,
          sampled: true,
          .. ImageUsage::none()
      };
      
      self.fullcolour_attachment = vkimage::AttachmentImage::sampled(self.window.get_device(), dimensions, format::Format::R16G16B16A16Unorm).unwrap();
      self.vkpost.bloom_renderpass.set_attachment(vkimage::AttachmentImage::with_usage(self.window.get_device(), dimensions, format::Format::R16G16B16A16Unorm, src_usage).unwrap());
      
      self.ms_colour_attachment = vkimage::AttachmentImage::transient_multisampled(self.window.get_device(), dimensions, self.samples, format::Format::R16G16B16A16Unorm).unwrap();
      self.ms_depth_attachment = vkimage::AttachmentImage::transient_multisampled(self.window.get_device(), dimensions, self.samples, format::Format::D16Unorm).unwrap();
      
      self.vkpost.blur_upscale_attachment = vkimage::StorageImage::with_usage(self.window.get_device(), vkimage::Dimensions::Dim2d { width: dimensions[0], height: dimensions[1]}, format::R8G8B8A8Unorm, dst_usage, self.window.get_queue_ref().family().physical_device().queue_families().find(|&q| {
          q.supports_graphics()
        }
      )).unwrap();
      
      let main_renderpass = self.main_renderpass.clone().unwrap();
      self.framebuffers = Some(Arc::new({
        framebuffer::Framebuffer::start(main_renderpass.clone())
                .add(self.ms_colour_attachment.clone()).unwrap()
                .add(self.ms_depth_attachment.clone()).unwrap()
                .add(self.fullcolour_attachment.clone()).unwrap()
                .build().unwrap()
      }));
      
      let bloom_renderpass = self.vkpost.bloom_renderpass.renderpass();
      let bloom_attachment = self.vkpost.bloom_renderpass.attachment();
      self.vkpost.bloom_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(bloom_renderpass)
                .add(bloom_attachment).unwrap()
                .build().unwrap()
      }));
      
      let blur_ping_renderpass = self.vkpost.blur_ping_renderpass.renderpass();
      let blur_ping_attachment = self.vkpost.blur_ping_renderpass.attachment();
      self.vkpost.blur_ping_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(blur_ping_renderpass)
                .add(blur_ping_attachment).unwrap()
                .build().unwrap()
      }));
      
      let blur_pong_renderpass = self.vkpost.blur_pong_renderpass.renderpass();
      let blur_pong_attachment = self.vkpost.blur_pong_renderpass.attachment();
      self.vkpost.blur_pong_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(blur_pong_renderpass)
                .add(blur_pong_attachment).unwrap()
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
      let dim = self.window.get_dimensions();
      [dim.width as u32, dim.height as u32]
    };
    
    let command_buffer: AutoCommandBuffer = {
      let mut tmp_cmd_buffer = AutoCommandBufferBuilder::primary_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family()).unwrap();
      
      let build_start = tmp_cmd_buffer.clear_color_image(self.vkpost.blur_downscale_attachment.clone(), ClearValue::Float([0.0, 0.0, 0.0, 1.0])).unwrap()
                                      .clear_color_image(self.vkpost.blur_upscale_attachment.clone(), ClearValue::Float([0.0, 0.0, 0.0, 1.0])).unwrap();
      
      let clear = [self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w];
      tmp_cmd_buffer = build_start.begin_render_pass(self.framebuffers.as_ref().unwrap().clone(), false, vec![ClearValue::Float(clear.into()), ClearValue::Depth(1.0), ClearValue::None]).unwrap();
      for draw in draw_calls {
        if draw.is_3d_model() {
          let models = &self.vk3d.models;
          let projection = self.vk3d.projection;
          let view_matrix = self.vk3d.camera.get_view_matrix();
          let uniform_buffer = self.vk3d.uniform_buffer.clone();
          let pipeline = &self.vk3d.pipeline;
          let subbuffer = vulkan_3d::create_3d_subbuffer(
                                               draw.clone(), 
                                               projection, 
                                               view_matrix, uniform_buffer
                                             );
          
          tmp_cmd_buffer = vulkan_draw::draw_3d(tmp_cmd_buffer, draw, models, projection, view_matrix, pipeline, 
                               subbuffer, dimensions);
        } else {
          if draw.is_vao_update() {
            let reference = draw.get_text().clone();
            
            if self.vk2d.custom_dynamic_vao.contains_key(&reference) {
              let mut verts = draw.get_new_vertices();
              let mut index = draw.get_new_indices();
              
              let new_model = vulkan_2d::create_dynamic_custom_model(self.window.get_device(), verts, index);
              
              if let Some(d_model) = self.vk2d.custom_dynamic_vao.get_mut(&reference) {
                *d_model = new_model;
              }
              
            } else {
              println!("Error: Dynamic vao update doesn't exist: {:?}", reference);
              continue;
            }
            
          } else if draw.is_text() {// Render Text
            let vao = &self.vk2d.vao;
            let sampler = self.vk2d.sampler.clone();
            let textures = &self.textures;
            let fonts = &self.fonts;
            let projection = self.vk2d.projection;
            let pipeline = &self.vk2d.pipeline_text;
            let uniform_buffer = &self.vk2d.uniform_buffer_text;
            tmp_cmd_buffer = vulkan_draw::draw_text(tmp_cmd_buffer, draw, textures, 
                                                    projection, vao, sampler, 
                                                    uniform_buffer, pipeline, 
                                                    fonts, dimensions);
          } else { // 2D texture
            let uniform_subbuffer = vulkan_2d::create_texture_subbuffer(draw.clone(), self.vk2d.projection, self.vk2d.uniform_buffer_texture.clone());
            let vao = &self.vk2d.vao;
            let custom_vao = &self.vk2d.custom_vao;
            let custom_dynamic_vao = &self.vk2d.custom_dynamic_vao;
            let sampler = self.vk2d.sampler.clone();
            let textures = &self.textures;
            let projection = self.vk2d.projection;
            let pipeline = &self.vk2d.pipeline_texture;
            let queue = self.window.get_queue();
            tmp_cmd_buffer = vulkan_draw::draw_texture(tmp_cmd_buffer, draw, textures,
                                                       vao, custom_vao, 
                                                       custom_dynamic_vao, projection, 
                                                       sampler, uniform_subbuffer, 
                                                       pipeline, queue, dimensions);
          }
        }
      }
      /*
      /*
      * Bloom framebuffer
      */
      let build_start = tmp_cmd_buffer.end_render_pass().unwrap();
      let cb = build_start.begin_render_pass(self.vkpost.bloom_renderpass.framebuffer_ref(), false, vec![ClearValue::None]).unwrap();
      
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.bloom_renderpass.pipeline();
      
      let model = math::calculate_texture_model(Vector3::new(dimensions[0] as f32 * 0.5, dimensions[1] as f32 * 0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
      
      let uniform_data = vs_post_bloom::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
      };
      
      let uniform_subbuffer = self.vkpost.bloom_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.fullcolour_attachment.clone(), self.sampler.clone()).unwrap()
                             .build().unwrap());
                          
      tmp_cmd_buffer = vulkan_helper::draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
      let build_end = tmp_cmd_buffer.end_render_pass().unwrap();
      
      /*
      * Blur ping framebuffer
      */
      let build_start = build_end.blit_image(self.vkpost.bloom_renderpass.attachment(), [0,0,0], [dimensions[0] as i32, dimensions[1] as i32, 1], 0, 0, 
                                             self.vkpost.blur_downscale_attachment.clone(), [0, 0, 0], [blur_dim as i32, blur_dim as i32, 1], 0, 0, 
                                             1, sampler::Filter::Linear).expect("Failed to scale down bloom image to blur image");
      
      let cb = build_start.begin_render_pass(self.vkpost.blur_ping_renderpass.framebuffer_ref(), false, vec![ClearValue::None]).unwrap();
      
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.blur_ping_renderpass.pipeline();
      
      let model = math::calculate_texture_model(Vector3::new(blur_dim as f32 * 0.5, dimensions[1] as f32 - blur_dim as f32 * 0.5, 0.0), Vector2::new(blur_dim as f32, blur_dim as f32), 90.0);
      
      let uniform_data = vs_post_blur::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        direction: Vector2::new(1.0, 0.0).into(),
      };
      
      let uniform_subbuffer = self.vkpost.blur_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.vkpost.blur_downscale_attachment.clone(), self.sampler.clone()).unwrap()
                             .build().unwrap());
                          
      tmp_cmd_buffer = vulkan_helper::draw_immutable(cb, dimensions, pipeline.clone(), vertex_buffer.clone(), index_buffer.clone(), uniform_set);
      
      /*
      * Blur pong framebuffer
      */
      let build_start = tmp_cmd_buffer.end_render_pass().unwrap();
      let cb = build_start.begin_render_pass(self.vkpost.blur_pong_renderpass.framebuffer_ref(), false, vec![ClearValue::None]).unwrap();
      
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.blur_pong_renderpass.pipeline();
      
      let model = math::calculate_texture_model(Vector3::new(blur_dim as f32 * 0.5, dimensions[1] as f32 - blur_dim as f32 * 0.5, 0.0), Vector2::new(blur_dim as f32, blur_dim as f32), 90.0);
      
      let uniform_data = vs_post_blur::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        direction: Vector2::new(0.0, 1.0).into(),
      };
      
      let uniform_subbuffer = self.vkpost.blur_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.vkpost.blur_ping_renderpass.attachment(), self.sampler.clone()).unwrap()
                             .build().unwrap());
                          
      tmp_cmd_buffer = vulkan_helper::draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
      let build_end = tmp_cmd_buffer.end_render_pass().unwrap();
      
      /*
      * Final Framebuffer
      */
      
      let build_start = build_end.blit_image(self.vkpost.blur_pong_renderpass.attachment(), [0,0,0], [blur_dim as i32, blur_dim as i32, 1], 0, 0, 
                                             self.vkpost.blur_upscale_attachment.clone(),  [0, 0, 0], [dimensions[0] as i32, dimensions[1] as i32, 1], 0, 0,
                                             1, sampler::Filter::Linear).expect("Failed to scale up blur image to final bloom image");*/
      let build_start = tmp_cmd_buffer.end_render_pass().unwrap();
      let clear = [self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w];
      let cb = build_start.begin_render_pass(self.vkpost.final_framebuffer.as_ref().unwrap()[image_num].clone(), false, vec![ClearValue::None]).unwrap();
      
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.final_pipeline.clone().unwrap();
      
      let model = math::calculate_texture_model(Vector3::new(dimensions[0] as f32*0.5, dimensions[1] as f32*0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
      
      let uniform_data = vs_post_final::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        bloom: 2.0,
      };
      
      let uniform_subbuffer = self.vkpost.final_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.fullcolour_attachment.clone(), self.vk2d.sampler.clone()).unwrap()
                             .add_sampled_image(self.fullcolour_attachment.clone(), self.vk2d.sampler.clone()).unwrap()
                             .build().unwrap());
      
      tmp_cmd_buffer = vulkan_helper::draw_immutable(cb, dimensions, pipeline.clone(), vertex_buffer.clone(), index_buffer.clone(), uniform_set);
      
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
  fn screen_resized(&mut self, window_size: LogicalSize) {
    self.recreate_swapchain = true;
  }
  
  /// Returns the dimensions of the drawing window as u32
  fn get_dimensions(&self) -> LogicalSize {
    self.window.get_dimensions()
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
  fn get_dpi_scale(&self) -> f64 {
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
  
  // Sets the clear colour for the window to display if nothing is draw in an area
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
    self.clear_colour = Vector4::new(r,g,b,a);
  }
  
  /// Sets the camera location, rotation, view given a Camera object
  fn set_camera(&mut self, camera: Camera) {
    self.vk3d.camera = camera;
  }
  
  fn get_camera(&self) -> Camera {
    self.vk3d.camera.to_owned()
  }
  
  /// does nothing in vulkan
  fn post_draw(&self) {}
  /// does nothing in vulkan
  fn clean(&self) {}
  /// does nothing
  fn swap_buffers(&mut self) {}
}


