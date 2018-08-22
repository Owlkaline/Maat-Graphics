use font::GenericFont;
use window::VkWindow;
use drawcalls::DrawCall;
use math;
use graphics::Vertex2d;
use graphics::Vertex3d;
use graphics::CoreRender;
use settings::Settings;
use camera::Camera;
use gltf_interpreter::ModelDetails;
use vulkan::vulkan_2d;
use vulkan::vulkan_3d;
use vulkan::vulkan_draw;
use vulkan::vulkan_helper;
use vulkan::renderpass::CustomRenderpass;

use gltf::material::AlphaMode;

use image;
use winit;
use winit::dpi::LogicalSize;

use vulkano::image as vkimage;
use vulkano::sampler;

use vulkano::sync::now;
use vulkano::sync::GpuFuture;
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
use vulkano::pipeline::viewport::Viewport;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::pipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

use vulkano::format;
use vulkano::format::ClearValue;
use vulkano::image::ImmutableImage;
use vulkano::descriptor::descriptor_set;
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::swapchain::SwapchainCreationError;

use std::env;
use std::mem;
use std::cmp;
use std::time;
use std::marker::Sync;
use std::marker::Send;
use std::collections::HashMap;
use std::sync::Arc;

use cgmath;
use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;
use cgmath::SquareMatrix;

impl_vertex!(Vertex2d, position, uv);
impl_vertex!(Vertex3d, position, normal, tangent, uv, colour);

const SHADOW_ATTACHMENT: usize = 0;

const FORWARDBUFFER_MS_COLOUR: usize = 0;
const FORWARDBUFFER_MS_DEPTH: usize = 1;
const FORWARDBUFFER_FULLCOLOUR: usize = 2;
const FORWARDBUFFER_BLOOMCOLOUR: usize = 3;

const TEXTURE_PIPELINE: usize = 0;
const TEXT_PIPIELINE: usize = 1;

const TEXTURE_FULLCOLOUR: usize = 1;

pub const BLUR_DIM: u32 = 512;

pub mod vs_texture {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkTexture.vert"]
  struct Dummy;
}

pub mod fs_texture {
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

pub mod fs_text {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkText.frag"]
  struct Dummy;
}

pub mod vs_forwardbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkForward3D.vert"]
  struct Dummy;
}

pub mod fs_forwardbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkForward3D.frag"]
  struct Dummy;
}

pub mod vs_gbuffer_3d{
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkGBuffer3D.vert"]
  struct Dummy;
}

pub mod fs_gbuffer_3d {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkGBuffer3D.frag"]
  struct Dummy;
}

pub mod vs_plain {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkPlain.vert"]
  struct Dummy;
}

pub mod fs_lights {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkLight.frag"]
  struct Dummy;
}

pub mod fs_post_bloom {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkPostBloom.frag"]
  struct Dummy;
}

pub mod vs_shadow {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkShadow.vert"]
  struct Dummy;
}

pub mod fs_shadow {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkShadow.frag"]
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
  
  texture_renderpass: CustomRenderpass,
  
  uniform_buffer_texture: cpu_pool::CpuBufferPool<vs_texture::ty::Data>,
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
  
  shadowbuffer_renderpass: CustomRenderpass,
  shadow_uniform_buffer: cpu_pool::CpuBufferPool<vs_shadow::ty::Data>,
  
  forwardbuffer_subpass_vertex: Arc<BufferAccess + Send + Sync>,
  forwardbuffer_renderpass: CustomRenderpass,
  
  uniform_buffer: cpu_pool::CpuBufferPool<vs_forwardbuffer_3d::ty::Data>,
}

pub struct VKPOST {
  shadow_renderpass: CustomRenderpass,
  
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
  num_drawcalls: u32,
  shaders_loaded: bool,
  ready: bool, // finished loading models and textures
  fonts: HashMap<String, GenericFont>,
  textures: HashMap<String, Arc<ImmutableImage<format::R8G8B8A8Unorm>>>,
  diffuse_colours: HashMap<String, [f32; 3]>,
  texture_paths: HashMap<String, String>,
  model_paths: HashMap<String, ModelInfo>,
  
  clear_colour: Vector4<f32>,
  
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
  
  default_dynamic_state: DynamicState,
  recreate_swapchain: bool,
  window_actual_size: LogicalSize,
  
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
    let vsync = settings.vsync_enabled();
    let triple_buffer = settings.triple_buffer_enabled();
    
    let window = VkWindow::new(width, height, min_width, min_height, fullscreen, vsync, triple_buffer);
    
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
    let uniform_3d = cpu_pool::CpuBufferPool::<vs_forwardbuffer_3d::ty::Data>::new(window.get_device(), BufferUsage::uniform_buffer());
    let shadow_uniform_buffer = cpu_pool::CpuBufferPool::<vs_shadow::ty::Data>::new(window.get_device(), BufferUsage::uniform_buffer());
    let post_final_uniform = cpu_pool::CpuBufferPool::<vs_post_final::ty::Data>::new(window.get_device(), BufferUsage::uniform_buffer());
    
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
    
    let blur_ping_attachment = vkimage::AttachmentImage::sampled(window.get_device(), [BLUR_DIM, BLUR_DIM], window.get_swapchain().format()).unwrap();
    let blur_pong_attachment = vkimage::AttachmentImage::with_usage(window.get_device(), [BLUR_DIM, BLUR_DIM], window.get_swapchain().format(), src_usage).unwrap();
    
    let blur_downscale_attachment = vkimage::StorageImage::with_usage(window.get_device(), vkimage::Dimensions::Dim2d { width: BLUR_DIM, height: BLUR_DIM}, format::R8G8B8A8Unorm, dst_usage, window.get_queue_ref().family().physical_device().queue_families().find(|&q| {
          q.supports_graphics()
        }
      )).unwrap();
    
    let blur_upscale_attachment = vkimage::StorageImage::with_usage(window.get_device(), vkimage::Dimensions::Dim2d { width: dim[0], height: dim[1]}, format::R8G8B8A8Unorm, dst_usage, window.get_queue_ref().family().physical_device().queue_families().find(|&q| {
          q.supports_graphics()
        }
      )).unwrap();
    
    let lightpass_vertexbuffer = vulkan_3d::create_subpass_vertex(window.get_device());
    
    
    let dynamic_state = DynamicState {
                     line_width: None,
                     viewports: Some(vec![Viewport {
                       origin: [0.0, 0.0],
                       dimensions: [dim[0] as f32, dim[1] as f32],
                       depth_range: 0.0 .. 1.0,
                     }]),
                     scissors: None,
                   };
    RawVk {
      num_drawcalls: 0,
      shaders_loaded: false,
      ready: false,
      fonts: HashMap::new(),
      textures: HashMap::new(),
      diffuse_colours: HashMap::new(),
      texture_paths: HashMap::new(),
      model_paths: HashMap::new(),
      
      clear_colour: Vector4::new(0.0, 0.0, 0.0, 1.0),
      
      samples: samples,
      
      // 3D
      vk3d: VK3D {
        depth_buffer: None,
        
        models: HashMap::new(),
        
        pipeline: None,
        
        camera: Camera::default_vk(),
        
        projection: proj_3d,
        scale: scale,
        
        shadow_uniform_buffer: shadow_uniform_buffer,
        shadowbuffer_renderpass: CustomRenderpass::new_empty(),
        forwardbuffer_subpass_vertex: lightpass_vertexbuffer,
        forwardbuffer_renderpass: CustomRenderpass::new_empty(),
        
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
        
        texture_renderpass: CustomRenderpass::new_empty(),
        
        uniform_buffer_texture: texture_uniform,
        
        uniform_buffer_text: text_uniform,
        
        sampler: sampler,
      },
      
      // Post Processing
      vkpost: VKPOST {
        shadow_renderpass: CustomRenderpass::new_empty(),
        
        blur_uniformbuffer: post_blur_uniform,
        blur_ping_renderpass: CustomRenderpass::new(vec!(blur_ping_attachment)),
        blur_pong_renderpass: CustomRenderpass::new(vec!(blur_pong_attachment)),
        
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
      
      default_dynamic_state: dynamic_state,
      recreate_swapchain: false,
      window_actual_size: LogicalSize::new(dim[0] as f64, dim[1] as f64),
      
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
    debug_assert!(!self.model_paths.contains_key(&reference) && !self.vk3d.models.contains_key(&reference), 
                  "Attempted to create a static geometry with the reference: {}, that is already in use by a model", reference);
    debug_assert!(!self.texture_paths.contains_key(&reference) && !self.textures.contains_key(&reference), 
                  "Attempted to create a static geometry with the reference: {}, that is already in use by a texture", reference);
    debug_assert!(!self.fonts.contains_key(&reference), 
                  "Attempted to create a static geometry with the reference: {}, that is already in use by a font", reference);
    debug_assert!(!self.vk2d.custom_vao.contains_key(&reference), 
                  "Attempted to create a static geometry with the reference: {}, that is already in use by another static geometry", reference);
    debug_assert!(!self.vk2d.custom_dynamic_vao.contains_key(&reference), 
                  "Attempted to create a static geometry with the reference: {}, that is already in use by a dynamic geometry", reference);
    
    let (model, future) = vulkan_2d::create_static_custom_model(self.window.get_device(), self.window.get_queue(), vertices, indices);
    self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    self.vk2d.custom_vao.insert(reference, model);
  }
  
  fn load_dynamic_geometry(&mut self, reference: String, vertices: Vec<Vertex2d>, indices: Vec<u32>) {
    debug_assert!(!self.model_paths.contains_key(&reference) && !self.vk3d.models.contains_key(&reference), 
                  "Attempted to create a dynamic geometry with the reference: {}, that is already in use by a model", reference);
    debug_assert!(!self.texture_paths.contains_key(&reference) && !self.textures.contains_key(&reference), 
                  "Attempted to create a dynamic geometry with the reference: {}, that is already in use by a texture", reference);
    debug_assert!(!self.fonts.contains_key(&reference), 
                  "Attempted to create a dynamic geometry with the reference: {}, that is already in use by a font", reference);
    debug_assert!(!self.vk2d.custom_vao.contains_key(&reference), 
                  "Attempted to create a dynamic geometry with the reference: {}, that is already in use by a static geometry", reference);
    debug_assert!(!self.vk2d.custom_dynamic_vao.contains_key(&reference), 
                  "Attempted to create a dynamic geometry with the reference: {}, that is already in use by another dynamic geometry", reference);
    
    let model = vulkan_2d::create_dynamic_custom_model(self.window.get_device(), vertices, indices);
    self.vk2d.custom_dynamic_vao.insert(reference, model);
  }
  
  fn preload_model(&mut self, reference: String, directory: String) {
    self.load_model(reference.clone(), directory);
  }
  
  fn add_model(&mut self, reference: String, directory: String) {
    debug_assert!(!self.model_paths.contains_key(&reference) && !self.vk3d.models.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by another model", reference);
    debug_assert!(!self.texture_paths.contains_key(&reference) && !self.textures.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by a texture", reference);
    debug_assert!(!self.fonts.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by a font", reference);
    debug_assert!(!self.vk2d.custom_vao.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by a static geometry", reference);
    debug_assert!(!self.vk2d.custom_dynamic_vao.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by a dynamic geometry", reference);
    
    self.model_paths.insert(reference.clone(), ModelInfo {directory: directory.clone()});
  }
  
  fn load_model(&mut self, reference: String, directory: String) {
    debug_assert!(self.shaders_loaded, "load_model function called before shaders loaded.\n Please use add_model function instead.");
    
    debug_assert!(!self.vk3d.models.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by another model", reference);
    debug_assert!(!self.texture_paths.contains_key(&reference) && !self.textures.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by a texture", reference);
    debug_assert!(!self.fonts.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by a font", reference);
    debug_assert!(!self.vk2d.custom_vao.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by a static geometry", reference);
    debug_assert!(!self.vk2d.custom_dynamic_vao.contains_key(&reference), 
                  "Attempted to create a model with the reference: {}, that is already in use by a dynamic geometry", reference);
    
    let start_time = time::Instant::now();
    
    let mesh_data = ModelDetails::new(directory.clone());
    
    let mut mesh: Vec<Mesh> = Vec::new();
    
    let params_buffer = cpu_pool::CpuBufferPool::new(self.window.get_device().clone(), BufferUsage::uniform_buffer());
    let material_params = params_buffer.next(fs_forwardbuffer_3d::ty::MaterialParams {
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
      alpha_cutoff: 0.5,
      forced_alpha: 0,
      has_normals: 0,
      has_tangents: 0,
      _dummy0: [0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8],
    }).unwrap();
       
    let default_sampler = sampler::Sampler::new(self.window.get_device(), sampler::Filter::Linear,
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
    
    let default_descriptor_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.vk3d.forwardbuffer_renderpass.pipeline_subpass(0).clone(), 1)
              .add_buffer(material_params)
              .unwrap()
              .add_sampled_image(temp_tex.clone(), default_sampler.clone())
              .unwrap()
              .add_sampled_image(temp_tex.clone(), default_sampler.clone())
              .unwrap()
              .add_sampled_image(temp_tex.clone(), default_sampler.clone())
              .unwrap()
              .add_sampled_image(temp_tex.clone(), default_sampler.clone())
              .unwrap()
              .add_sampled_image(temp_tex.clone(), default_sampler.clone())
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
      let tangent = mesh_data.tangent(i);
      let index = mesh_data.index(i);
      let colour = mesh_data.colours(i);
      let alpha_mode = mesh_data.alphamode(i);
      let mut forced_alpha = 0;
      let mut alpha_cutoff = mesh_data.alphacutoff(i);
      let mut base_colour_texture = (temp_tex.clone(), -1, sampler::Sampler::simple_repeat_linear(self.window.get_device()));
      let mut metallic_roughness_texture = (temp_tex.clone(), -1, sampler::Sampler::simple_repeat_linear(self.window.get_device()));
      let mut normal_texture = (temp_tex.clone(), -1, sampler::Sampler::simple_repeat_linear(self.window.get_device()));
      let mut occlusion_texture = (temp_tex.clone(), -1, sampler::Sampler::simple_repeat_linear(self.window.get_device()));
      let mut emissive_texture = (temp_tex.clone(), -1, sampler::Sampler::simple_repeat_linear(self.window.get_device()));
      
      let has_normals = if mesh_data.has_normals(i) { 1 } else { 0 };
      let has_tangents = if mesh_data.has_tangents(i) { 1 } else { 0 };
      
      match alpha_mode {
        AlphaMode::Opaque => {
          forced_alpha = 1;
        },
        AlphaMode::Mask => {
          forced_alpha = 2;
        },
        AlphaMode::Blend => {
          forced_alpha = 0;
        },
      }
      
      let mut vertices: Vec<Vertex3d> = Vec::with_capacity(vertex.len());
      for j in 0..vertex.len() {
        let mut uv = [1.0, 1.0];
        if texcoord.len() > j {
          uv = texcoord[j];
        }
        let mut n_normal = [1.0, 1.0, 1.0];
        if normal.len() > j {
          n_normal = normal[j];
        }
        let mut t_tangent = [1.0, 1.0, 1.0, 1.0];
        if tangent.len() > j {
          t_tangent = tangent[j]
        }
        let mut c_colour = [1.0, 1.0, 1.0, 1.0];
        if colour.len() > j {
          c_colour = colour[j];
        }
        vertices.push(Vertex3d { position: vertex[j], normal: n_normal, tangent: t_tangent, uv: uv, colour: c_colour });
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
        
        let sampler = vulkan_3d::create_sampler_from_gltfsampler(self.window.get_device(), mesh_data.base_colour_sampler(i));
        
        base_colour_texture = (texture, 0, sampler);
      }
      
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.metallic_roughness_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
        
        let sampler = vulkan_3d::create_sampler_from_gltfsampler(self.window.get_device(), mesh_data.metallic_roughness_sampler(i));
        
        metallic_roughness_texture = (texture, 0, sampler);
      }
      
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.normal_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
        
        let sampler = vulkan_3d::create_sampler_from_gltfsampler(self.window.get_device(), mesh_data.normal_sampler(i));
        
        normal_texture = (texture, 0, sampler);
      }
      
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.occlusion_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
        
        let sampler = vulkan_3d::create_sampler_from_gltfsampler(self.window.get_device(), mesh_data.occlusion_sampler(i));
        
        occlusion_texture = (texture, 0, sampler);
      }
      
      let future_texture = vulkan_3d::create_texture_from_dynamicimage(self.window.get_queue(), mesh_data.emissive_texture(i));
      if future_texture.is_some() {
        let (texture, future) = future_texture.unwrap();
        self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
        
        let sampler = vulkan_3d::create_sampler_from_gltfsampler(self.window.get_device(), mesh_data.emissive_sampler(i));
        
        emissive_texture = (texture, 0, sampler);
      }
      
     // let params_buffer = cpu_pool::CpuBufferPool::new(self.window.get_device().clone(), BufferUsage::uniform_buffer());
      let material_params = params_buffer.next(fs_forwardbuffer_3d::ty::MaterialParams {
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
         alpha_cutoff: alpha_cutoff,
         forced_alpha: forced_alpha,
         has_normals: has_normals,
         has_tangents: has_tangents,
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
         Arc::new(descriptor_set::PersistentDescriptorSet::start(self.vk3d.forwardbuffer_renderpass.pipeline_subpass(0).clone(), 1)
              .add_buffer(material_params)
              .unwrap()
              .add_sampled_image(base_colour_texture.0, base_colour_texture.2)
              .unwrap()
              .add_sampled_image(metallic_roughness_texture.0, metallic_roughness_texture.2)
              .unwrap()
              .add_sampled_image(normal_texture.0, normal_texture.2)
              .unwrap()
              .add_sampled_image(occlusion_texture.0, occlusion_texture.2)
              .unwrap()
              .add_sampled_image(emissive_texture.0, emissive_texture.2)
              .unwrap()
              .build().unwrap());
              
        mesh[i].material_desctriptor = descriptor_set;
    }
    
    self.vk3d.models.insert(reference, mesh);
    
    let total_time = start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
    println!("{} ms,  {:?}", (total_time*1000f64) as f32, directory);
  }
  
  fn preload_texture(&mut self, reference: String, location: String) {
    self.load_texture(reference, location);
  }
  
  fn add_texture(&mut self, reference: String, location: String) {
    debug_assert!(!self.model_paths.contains_key(&reference) && !self.vk3d.models.contains_key(&reference), 
                  "Attempted to add a texture with the reference: {}, that is already in use by a model", reference);
    debug_assert!(!self.texture_paths.contains_key(&reference) && !self.textures.contains_key(&reference), 
                  "Attempted to add a texture with the reference: {}, that is already in use by another texture", reference);
    debug_assert!(!self.vk2d.custom_vao.contains_key(&reference), 
                  "Attempted to add a texture with the reference: {}, that is already in use by a static geometry", reference);
    debug_assert!(self.vk2d.custom_dynamic_vao.contains_key(&reference), 
                  "Attempted to add a texture with the reference: {}, that is already in use by a dynamic geometry", reference);
    self.texture_paths.insert(reference, location);
  }
  
  fn load_texture(&mut self, reference: String, location: String) {
    if location == String::from("") {
      return;
    }
    
    debug_assert!(!self.model_paths.contains_key(&reference) && !self.vk3d.models.contains_key(&reference), 
                  "Attempted to load a texture with the reference: {}, that is already in use by a model", reference);
    debug_assert!(!self.textures.contains_key(&reference), 
                  "Attempted to load a texture with the reference: {}, that is already in use by another texture", reference);
    debug_assert!(!self.vk2d.custom_vao.contains_key(&reference), 
                  "Attempted to load a texture with the reference: {}, that is already in use by a static geometry", reference);
    debug_assert!(!self.vk2d.custom_dynamic_vao.contains_key(&reference), 
                  "Attempted to load a texture with the reference: {}, that is already in use by a dynamic geometry", reference);
    
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
    self.add_texture(reference, font_texture);
  }
  
  fn load_font(&mut self, reference: String, font: &[u8]) {
    debug_assert!(!self.model_paths.contains_key(&reference) && !self.vk3d.models.contains_key(&reference), 
                  "Attempted to create a font with the reference: {}, that is already in use by a model", reference);
    debug_assert!(!self.fonts.contains_key(&reference), 
                  "Attempted to create a font with the reference: {}, that is already in use by another font", reference);
    debug_assert!(!self.vk2d.custom_vao.contains_key(&reference), 
                  "Attempted to create a font with the reference: {}, that is already in use by a static geometry", reference);
    debug_assert!(!self.vk2d.custom_dynamic_vao.contains_key(&reference), 
                  "Attempted to create a font with the reference: {}, that is already in use by a dynamic geometry", reference);
   let mut new_font = GenericFont::new();
    new_font.load_font(font);
    
    self.fonts.insert(reference.clone(), new_font);
  }
  
  /// Prepares shaders, pipelines, and vertex, index buffers
  /// # Warning
  /// You must call this function otherwise will result in crash
  fn load_shaders(&mut self) {
    debug_assert!(!self.shaders_loaded, "Error: Shaders already loaded");
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
    
    let vs_plain = vs_plain::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_forwardbuffer_3d = vs_forwardbuffer_3d::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_forwardbuffer_3d = fs_forwardbuffer_3d::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_texture = vs_texture::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_texture = fs_texture::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_text = vs_text::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_text = fs_text::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_post_final = vs_post_final::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_post_final = fs_post_final::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_post_blur = vs_post_blur::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_post_blur = fs_post_blur::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_shadow = vs_shadow::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_shadow = fs_shadow::Shader::load(self.window.get_device()).expect("failed to create shader module");
    
    self.vk3d.forwardbuffer_renderpass = vulkan_3d::create_forwardbuffer(self.window.get_device(), dimensions, self.samples);
    self.vk3d.shadowbuffer_renderpass = vulkan_3d::create_shadow_renderpass(self.window.get_device());
    self.vk2d.texture_renderpass = vulkan_2d::create_texturebuffer(self.window.get_device(), dimensions, self.samples);
    
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
    
    let blur_ping_attachment = self.vkpost.blur_ping_renderpass.attachment(0);
    self.vkpost.blur_ping_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(blur_ping_renderpass.clone())
                .add(blur_ping_attachment).unwrap()
                .build().unwrap()
    }));
    
    let blur_pong_attachment = self.vkpost.blur_pong_renderpass.attachment(0);
    self.vkpost.blur_pong_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(blur_pong_renderpass.clone())
                .add(blur_pong_attachment).unwrap()
                .build().unwrap()
    }));
    
    self.vkpost.blur_ping_renderpass.set_renderpass(blur_ping_renderpass);
    self.vkpost.blur_ping_renderpass.set_pipelines(vec!(blur_ping_pipeline));
    self.vkpost.blur_pong_renderpass.set_renderpass(blur_pong_renderpass);
    self.vkpost.blur_pong_renderpass.set_pipelines(vec!(blur_pong_pipeline));
    
    self.vk2d.uniform_buffer_texture = cpu_pool::CpuBufferPool::<vs_texture::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    self.vk2d.uniform_buffer_text = cpu_pool::CpuBufferPool::<vs_text::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    self.default_dynamic_state.viewports = Some(vec![Viewport {
                       origin: [0.0, 0.0],
                       dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                       depth_range: 0.0 .. 1.0,
                     }]);
    
    self.shaders_loaded = true;
  }
  
  /// Initalises some variables
  fn init(&mut self) {
    debug_assert!(self.shaders_loaded, "Error: Shaders not loaded");
    
    self.recreate_swapchain = false;
  }
  
  /// Loads the unloaded textures returning after ~16ms has passed to allow for 
  /// updates whilst still loading
  fn dynamic_load(&mut self) {
    debug_assert!(self.shaders_loaded, "Error: Shaders not loaded");
    
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
      self.load_model(reference.clone(), model.directory.clone());
      
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
    debug_assert!(self.shaders_loaded, "Error: Shaders not loaded");
    
    self.clear_screen();
    
    if self.recreate_swapchain {
      let mut dimensions = {
        let dim = self.window.get_dimensions();
       // [dim.width as u32, dim.height as u32]
        [self.window_actual_size.width as u32, self.window_actual_size.height as u32]
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
          println!("UnsupportedDimensions");
          return;
        },
        Err(err) => panic!("{:?}", err)
      };
      
      self.window.replace_swapchain(new_swapchain);
      self.window.replace_images(new_images);
      
      self.vkpost.final_framebuffer = None;
      self.recreate_swapchain = false;
      
      let device = self.window.get_device();
      let new_depth_buffer = vulkan_3d::create_depth_buffer(device.clone(), dimensions);
      self.vk3d.depth_buffer = new_depth_buffer;
      
      self.vk2d.projection = vulkan_2d::create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
      self.vk3d.projection = vulkan_3d::create_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
      
      let dst_usage = ImageUsage {
          transfer_destination: true,
          sampled: true,
          .. ImageUsage::none()
      };
      
      {
        let renderpass = &mut self.vk3d.forwardbuffer_renderpass;
        vulkan_3d::recreate_forwardbuffer(renderpass, device.clone(), dimensions, self.samples);
        
        let renderpass = &mut self.vk3d.shadowbuffer_renderpass;
        vulkan_3d::recreate_shadow_renderpass(renderpass, device.clone());
        
        let renderpass = &mut self.vk2d.texture_renderpass;
        vulkan_2d::recreate_texturebuffer(renderpass, device, dimensions, self.samples);
      }
      
      self.vkpost.blur_upscale_attachment = vkimage::StorageImage::with_usage(self.window.get_device(), vkimage::Dimensions::Dim2d { width: dimensions[0], height: dimensions[1]}, format::R8G8B8A8Unorm, dst_usage, self.window.get_queue_ref().family().physical_device().queue_families().find(|&q| {
          q.supports_graphics()
        }
      )).unwrap();
      
      self.default_dynamic_state.viewports = Some(vec![Viewport {
                       origin: [0.0, 0.0],
                       dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                       depth_range: 0.0 .. 1.0,
                     }]);
      
      // Attachment doesnt need updating as they dont resize?
     /* let blur_ping_renderpass = self.vkpost.blur_ping_renderpass.renderpass();
      let blur_ping_attachment = self.vkpost.blur_ping_renderpass.attachment(0);
      self.vkpost.blur_ping_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(blur_ping_renderpass)
                .add(blur_ping_attachment).unwrap()
                .build().unwrap()
      }));
      
      let blur_pong_renderpass = self.vkpost.blur_pong_renderpass.renderpass();
      let blur_pong_attachment = self.vkpost.blur_pong_renderpass.attachment(0);
      self.vkpost.blur_pong_renderpass.set_framebuffer(Arc::new({
        framebuffer::Framebuffer::start(blur_pong_renderpass)
                .add(blur_pong_attachment).unwrap()
                .build().unwrap()
      }));*/
    }
    
    if self.vkpost.final_framebuffer.is_none() {
      let new_framebuffers = 
        Some(self.window.get_images().iter().map( |image| {
             let fb = framebuffer::Framebuffer::start(self.vkpost.final_renderpass.clone().unwrap())
                      .add(image.clone()).unwrap()
                      .build().unwrap();
             Arc::new(fb) as Arc<framebuffer::FramebufferAbstract + Send + Sync>
             }).collect::<Vec<_>>());
      self.vkpost.final_framebuffer = new_framebuffers;
    }
  }
  
  /// Draws everything that is in the drawcall passed to this function
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
    debug_assert!(self.shaders_loaded, "Error: Shaders not loaded");
    
    self.num_drawcalls = 0;
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
      let texture_subpass = framebuffer::Subpass::from(self.vk2d.texture_renderpass.renderpass(), 0).unwrap();
      let model_subpass = framebuffer::Subpass::from(self.vk3d.forwardbuffer_renderpass.renderpass(), 0).unwrap();
      let shadow_subpass = framebuffer::Subpass::from(self.vk3d.shadowbuffer_renderpass.renderpass(), 0).unwrap();
      
      let mut texture_cmd_buffer: AutoCommandBufferBuilder = AutoCommandBufferBuilder::secondary_graphics_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family(), texture_subpass).unwrap();
      let mut model_cmd_buffer: AutoCommandBufferBuilder = AutoCommandBufferBuilder::secondary_graphics_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family(), model_subpass).unwrap();
      let mut shadow_cmd_buffer: AutoCommandBufferBuilder = AutoCommandBufferBuilder::secondary_graphics_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family(), shadow_subpass).unwrap();
      
      for draw in draw_calls {
        if draw.is_model() { // 3D
          let models = &self.vk3d.models;
          
          let depth_pipeline = self.vk3d.shadowbuffer_renderpass.pipeline_subpass(0);
          let uniform_buffer = self.vk3d.shadow_uniform_buffer.clone();
          let depth_subbuffer = vulkan_3d::create_shadow_subbuffer(draw.clone(), uniform_buffer);
          let dynamic_state = &self.default_dynamic_state;
          let (shadow_cmd, num_calls) = vulkan_draw::draw_3d_depth(shadow_cmd_buffer, draw, models, depth_pipeline, dynamic_state, depth_subbuffer, dimensions);
          
          let projection = self.vk3d.projection;
          let view_matrix = self.vk3d.camera.get_view_matrix();
          let pipeline = self.vk3d.forwardbuffer_renderpass.pipeline_subpass(0);
          let uniform_buffer = self.vk3d.uniform_buffer.clone();
          let device = self.window.get_device();
          let depth_attachment = self.vk3d.shadowbuffer_renderpass.attachment(SHADOW_ATTACHMENT);
          let dynamic_state = &self.default_dynamic_state;
          let subbuffer = vulkan_3d::create_3d_subbuffer(
                                               draw.clone(), 
                                               projection, 
                                               view_matrix, uniform_buffer
                                             );
          let (model_cmd, num_calls) = vulkan_draw::draw_3d(model_cmd_buffer, draw, models, projection, 
                                                            view_matrix, pipeline, dynamic_state, depth_attachment, subbuffer, device, dimensions);
          
          shadow_cmd_buffer = shadow_cmd;
          model_cmd_buffer = model_cmd;
          
          self.num_drawcalls += num_calls;
        } else {
          if draw.is_shape_update() {
            if let Some(reference) = draw.shape_name() {
              if self.vk2d.custom_dynamic_vao.contains_key(&reference) {
                if let Some((verts, index)) = draw.new_shape_details() {
                  
                  let new_model = vulkan_2d::create_dynamic_custom_model(self.window.get_device(), verts, index);
                  
                  if let Some(d_model) = self.vk2d.custom_dynamic_vao.get_mut(&reference) {
                    *d_model = new_model;
                  }
                }
              } else {
                println!("Error: Dynamic vao update doesn't exist: {:?}", reference);
                continue;
              }
            }
          } else if draw.is_text() {// Render Text
            let vao = &self.vk2d.vao;
            let sampler = self.vk2d.sampler.clone();
            let textures = &self.textures;
            let fonts = &self.fonts;
            let projection = self.vk2d.projection;
            let subpass = framebuffer::Subpass::from(self.vk2d.texture_renderpass.renderpass(), 0).unwrap();
            let pipeline = self.vk2d.texture_renderpass.pipeline_subpass(TEXT_PIPIELINE);
            let uniform_buffer = &self.vk2d.uniform_buffer_text;
            let queue_family = self.window.get_queue_ref().family();
            let device = self.window.get_device();
            let dynamic_state = &self.default_dynamic_state;
            
            let (texture_cmd, num_calls) = vulkan_draw::draw_text(texture_cmd_buffer, 
                                                                  draw, textures, 
                                                                  projection, vao, sampler, 
                                                                  uniform_buffer, pipeline, dynamic_state,
                                                                  fonts, subpass, device, 
                                                                  queue_family, dimensions);
            texture_cmd_buffer = texture_cmd;
            
            self.num_drawcalls += num_calls;
          } else { // 2D texture
            let uniform_subbuffer = vulkan_2d::create_texture_subbuffer(draw.clone(), self.vk2d.projection, self.vk2d.uniform_buffer_texture.clone());
            let vao = &self.vk2d.vao;
            let custom_vao = &self.vk2d.custom_vao;
            let custom_dynamic_vao = &self.vk2d.custom_dynamic_vao;
            let sampler = self.vk2d.sampler.clone();
            let textures = &self.textures;
            let projection = self.vk2d.projection;
            let subpass = framebuffer::Subpass::from(self.vk2d.texture_renderpass.renderpass(), 0).unwrap();
            let pipeline = self.vk2d.texture_renderpass.pipeline_subpass(TEXTURE_PIPELINE);
            let queue = self.window.get_queue();
            let queue_family = self.window.get_queue_ref().family();
            let device = self.window.get_device();
            let dynamic_state = &self.default_dynamic_state;
            let (texture_cmd, num_calls) = vulkan_draw::draw_texture(texture_cmd_buffer, draw, textures,
                                                       vao, custom_vao, 
                                                       custom_dynamic_vao, projection, 
                                                       sampler, uniform_subbuffer, 
                                                       pipeline, dynamic_state, subpass, device, 
                                                       queue, queue_family, dimensions);
            texture_cmd_buffer = texture_cmd;
            self.num_drawcalls += num_calls;
          }
        }
      }
      
      let shadow_cmd_buffer = shadow_cmd_buffer.build().unwrap();
      let model_cmd_buffer = model_cmd_buffer.build().unwrap();
      let texture_cmd_buffer = texture_cmd_buffer.build().unwrap();
      
      let mut tmp_cmd_buffer = AutoCommandBufferBuilder::primary_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family()).unwrap();
      let clear = [self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w];
      
      unsafe {
        let cb = tmp_cmd_buffer.begin_render_pass(self.vk3d.shadowbuffer_renderpass.framebuffer_ref(), true, vec![ClearValue::None]).unwrap();
        tmp_cmd_buffer = cb.execute_commands(shadow_cmd_buffer).unwrap();
      }
      
      let build_end = tmp_cmd_buffer.end_render_pass().unwrap();
      
      tmp_cmd_buffer = build_end.begin_render_pass(self.vk3d.forwardbuffer_renderpass.framebuffer_ref(), true, vec![ClearValue::Float(clear), ClearValue::Depth(1.0), ClearValue::None, ClearValue::None]).unwrap();
      
      unsafe {
        let cb = tmp_cmd_buffer.execute_commands(model_cmd_buffer).unwrap();
        tmp_cmd_buffer = cb;
      }
      
      let subpass_end = tmp_cmd_buffer.next_subpass(false).unwrap();
      tmp_cmd_buffer = subpass_end;
      
      {
        let pipeline = self.vk3d.forwardbuffer_renderpass.pipeline_subpass(1).clone();
        let vertex = self.vk3d.forwardbuffer_subpass_vertex.clone();
        let colour_attachment = self.vk3d.forwardbuffer_renderpass.attachment(FORWARDBUFFER_FULLCOLOUR);
        let dynamic_state = &self.default_dynamic_state;
        tmp_cmd_buffer = vulkan_draw::draw_bloompass(tmp_cmd_buffer, 
                                                     pipeline, 
                                                     dynamic_state,
                                                     vertex, 
                                                     colour_attachment, 
                                                     dimensions);
      }
      
      let build_end = tmp_cmd_buffer.end_render_pass().unwrap();
      
      let clear2D = [self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, 0.0];
      
      let build_start = build_end.begin_render_pass(self.vk2d.texture_renderpass.framebuffer_ref(), true, vec![ClearValue::Float(clear2D), ClearValue::None]).unwrap();
      tmp_cmd_buffer = build_start;
      // do stuff
      unsafe {
        let cb = tmp_cmd_buffer.execute_commands(texture_cmd_buffer).unwrap();
        tmp_cmd_buffer = cb;
      }
      
      let build_end = tmp_cmd_buffer.end_render_pass().unwrap();
      
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
                             .add_sampled_image(self.fullcolour_attachment.clone(), self.vk2d.sampler.clone()).unwrap()
                             .build().unwrap());
                          
      tmp_cmd_buffer = vulkan_helper::draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
      let build_end = tmp_cmd_buffer.end_render_pass().unwrap();
      /*      let build_start = tmp_cmd_buffer.clear_color_image(self.vkpost.blur_downscale_attachment.clone(), ClearValue::Float(clear.into())).unwrap()
                                      .clear_color_image(self.vkpost.blur_upscale_attachment.clone(), ClearValue::Float(clear.into())).unwrap();*/
      /*
      * Blur ping framebuffer
      */
      let build_start = build_end.blit_image(self.vkpost.bloom_renderpass.attachment(0), [0,0,0], [dimensions[0] as i32, dimensions[1] as i32, 1], 0, 0, 
                                             self.vkpost.blur_downscale_attachment.clone(), [0, 0, 0], [BLUR_DIM as i32, BLUR_DIM as i32, 1], 0, 0, 
                                             1, sampler::Filter::Linear).expect("Failed to scale down bloom image to blur image");
      
      let cb = build_start.begin_render_pass(self.vkpost.blur_ping_renderpass.framebuffer_ref(), false, vec![ClearValue::None]).unwrap();
      
      let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
      let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
      let pipeline = self.vkpost.blur_ping_renderpass.pipeline();
      
      let model = math::calculate_texture_model(Vector3::new(BLUR_DIM as f32 * 0.5, dimensions[1] as f32 - BLUR_DIM as f32 * 0.5, 0.0), Vector2::new(BLUR_DIM as f32, BLUR_DIM as f32), 90.0);
      
      let uniform_data = vs_post_blur::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        direction: Vector2::new(1.0, 0.0).into(),
      };
      
      let uniform_subbuffer = self.vkpost.blur_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.vkpost.blur_downscale_attachment.clone(), self.vk2d.sampler.clone()).unwrap()
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
      
      let model = math::calculate_texture_model(Vector3::new(BLUR_DIM as f32 * 0.5, dimensions[1] as f32 - BLUR_DIM as f32 * 0.5, 0.0), Vector2::new(BLUR_DIM as f32, BLUR_DIM as f32), 90.0);
      
      let uniform_data = vs_post_blur::ty::Data {
        projection: self.vk2d.projection.into(),
        model: model.into(),
        direction: Vector2::new(0.0, 1.0).into(),
      };
      
      let uniform_subbuffer = self.vkpost.blur_uniformbuffer.next(uniform_data).unwrap();
      
      let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .add_sampled_image(self.vkpost.blur_ping_renderpass.attachment(0), self.vk2d.sampler.clone()).unwrap()
                             .build().unwrap());
                          
      tmp_cmd_buffer = vulkan_helper::draw_immutable(cb, dimensions, pipeline, vertex_buffer, index_buffer, uniform_set);
      let build_end = tmp_cmd_buffer.end_render_pass().unwrap();
      
      /*
      * Final Framebuffer
      */
      
      let build_start = build_end.blit_image(self.vkpost.blur_pong_renderpass.attachment(0), [0,0,0], [BLUR_DIM as i32, BLUR_DIM as i32, 1], 0, 0, 
                                             self.vkpost.blur_upscale_attachment.clone(),  [0, 0, 0], [dimensions[0] as i32, dimensions[1] as i32, 1], 0, 0,
                                             1, sampler::Filter::Linear).expect("Failed to scale up blur image to final bloom image");*/
      //tmp_cmd_buffer = build_start.end_render_pass().unwrap();
      let cb = build_end.begin_render_pass(self.vkpost.final_framebuffer.as_ref().unwrap()[image_num].clone(), false, vec![ClearValue::None]).unwrap();
      
      
        let vertex_buffer = self.vk2d.vao.vertex_buffer.clone().unwrap();
        let index_buffer = self.vk2d.vao.index_buffer.clone().unwrap();
        let pipeline = self.vkpost.final_pipeline.clone().unwrap();
        
        let model = math::calculate_texture_model(Vector3::new(dimensions[0] as f32*0.5, dimensions[1] as f32*0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
        
        let uniform_data = vs_post_final::ty::Data {
          projection: self.vk2d.projection.into(),
          model: model.into(),
          bloom: 0.0,
        };
        
        let uniform_subbuffer = self.vkpost.final_uniformbuffer.next(uniform_data).unwrap();
        
        let fullcolour_3d_attachment = self.vk3d.forwardbuffer_renderpass.attachment(FORWARDBUFFER_FULLCOLOUR);
        let fullcolour_2d_attachment = self.vk2d.texture_renderpass.attachment(TEXTURE_FULLCOLOUR);
        let depth_attachment = self.vk3d.shadowbuffer_renderpass.attachment(SHADOW_ATTACHMENT);
        let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                               .add_buffer(uniform_subbuffer.clone()).unwrap()
                               .add_sampled_image(fullcolour_3d_attachment.clone(), self.vk2d.sampler.clone()).unwrap()
                               .add_sampled_image(fullcolour_2d_attachment.clone(), self.vk2d.sampler.clone()).unwrap()
                               .add_sampled_image(fullcolour_3d_attachment, self.vk2d.sampler.clone()).unwrap()
                               .build().unwrap());
        let dynamic_state = &self.default_dynamic_state;
        
        tmp_cmd_buffer = vulkan_helper::draw_immutable(cb, dimensions, pipeline, dynamic_state, vertex_buffer, index_buffer, uniform_set);
      
      
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
        future.wait(None).unwrap();
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
    self.window_actual_size = self.get_dimensions();
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
  
  fn set_cursor_position(&mut self, x: f32, y: f32) {
    self.window.set_cursor_position(x, y);
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
  
    fn num_drawcalls(&self) -> u32 {
      self.num_drawcalls
    }
  
  /// does nothing in vulkan
  fn post_draw(&self) {}
  /// does nothing in vulkan
  fn clean(&self) {}
  /// does nothing
  fn swap_buffers(&mut self) {}
}


