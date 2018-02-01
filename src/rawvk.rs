use font::GenericFont;
use window::VkWindow;
use drawcalls::DrawCall;
use drawcalls::DrawMath;
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
use vulkano::image::ImmutableImage;
use vulkano::descriptor::descriptor_set;
use vulkano::swapchain::SwapchainCreationError;

use std::env;
use std::mem;
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

#[derive(Debug, Clone)]
struct Vertex { position: [f32; 2], uv: [f32; 2] }

impl_vertex!(Vertex, position, uv);

mod vs_texture {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/VkTexture.vert"]
  struct Dummy;
}

mod fs_texture {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/VkTexture.frag"]
  struct Dummy;
}

mod vs_text {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/VkText.vert"]
  struct Dummy;
}

mod fs_text {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/VkText.frag"]
  struct Dummy;
}

mod vs_3d {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/Vk3D.vert"]
  struct Dummy;
}

mod fs_3d {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/Vk3D.frag"]
  struct Dummy;
}

#[derive(Clone)]
pub struct ModelInfo {
  location: String,
  texture: String,
}

pub struct Model {
  vertex_buffer: Vec<Arc<BufferAccess + Send + Sync>>,
  index_buffer: Arc<ImmutableBuffer<[u16]>>,
}

pub struct RawVk {
  ready: bool,
  fonts: HashMap<String, GenericFont>,
  textures: HashMap<String, Arc<ImmutableImage<format::R8G8B8A8Unorm>>>,
  texture_paths: HashMap<String, String>,
  model_paths: HashMap<String, ModelInfo>,
  
  clear_colour: Vector4<f32>,
  
  framebuffers: Option<Vec<Arc<framebuffer::FramebufferAbstract + Send + Sync>>>,
  render_pass: Option<Arc<RenderPassAbstract + Send + Sync>>,

  depth_buffer: Option<Arc<vkimage::AttachmentImage<format::D16Unorm>>>,
  
  //3D
  models: HashMap<String, Model>,
  
  pipeline_3d: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  
  projection_3d: Matrix4<f32>,
  view: Matrix4<f32>,
  scale: Matrix4<f32>,
  
  uniform_buffer_3d: cpu_pool::CpuBufferPool<vs_3d::ty::Data>,

  //2D
  vertex_buffer_2d: Option<Vec<Arc<BufferAccess + Send + Sync>>>,
  index_buffer_2d: Option<Arc<ImmutableBuffer<[u16]>>>,

  pipeline_text: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  pipeline_texture: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,

  projection_2d: Matrix4<f32>,

  uniform_buffer_texture: cpu_pool::CpuBufferPool<vs_texture::ty::Data>,
  uniform_buffer_text: cpu_pool::CpuBufferPool<vs_text::ty::Data>,

  // Vk System stuff
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
    let previous_frame_end = Some(Box::new(now(window.get_device())) as Box<GpuFuture>);
    
    RawVk {
      ready: false,
      fonts: HashMap::new(),
      textures: HashMap::new(),
      texture_paths: HashMap::new(),
      model_paths: HashMap::new(),

      clear_colour: Vector4::new(0.0, 0.0, 0.0, 1.0),

      framebuffers: None,
      render_pass: None,

      depth_buffer: None,

      // 3D
      models: HashMap::new(),
      
      pipeline_3d: None,
      
      projection_3d: proj_3d,
      view: view,
      scale: scale,

      uniform_buffer_3d: uniform_3d,

      //2D
      vertex_buffer_2d: None,
      index_buffer_2d: None,
      
      pipeline_texture: None,
      pipeline_text: None,
      
      projection_2d: proj_2d,
            
      uniform_buffer_texture: texture_uniform,
      uniform_buffer_text: text_uniform,

      // Vk System
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
  
  pub fn create_2d_vertex(&self) -> Arc<BufferAccess + Send + Sync> {
    #[derive(Debug, Clone)]
    struct Vertex { position: [f32; 2], uv: [f32; 2] }

    impl_vertex!(Vertex, position, uv);
    
    let square = {
      [
          Vertex { position: [  0.5 ,   0.5 ], uv: [1.0, 0.0] },
          Vertex { position: [ -0.5,    0.5 ], uv: [0.0, 0.0] },
          Vertex { position: [ -0.5,   -0.5 ], uv: [0.0, 1.0] },
          Vertex { position: [  0.5 ,  -0.5 ], uv: [1.0, 1.0] },
      ]
    };
    
    CpuAccessibleBuffer::from_iter(self.window.get_device(), 
                                   BufferUsage::vertex_buffer(), 
                                   square.iter().cloned())
                                   .expect("failed to create vertex buffer")
  }
  
  pub fn create_2d_index(&self) -> (Arc<ImmutableBuffer<[u16]>>,
                                    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
    //Flickering at top and sides
    //let indicies: [u16; 6] = [1, 2, 3, 0, 3, 1];
    
    // Better, no flickering at top but worse flickering in corner
    //let indicies: [u16; 6] = [0, 3, 1, 1, 2, 3];
    
    //let indicies: [u16; 6] = [2, 3, 1, 0, 1, 3];
    let indicies: [u16; 6] = [0, 1, 2, 2, 3, 0];
    ImmutableBuffer::from_iter(indicies.iter().cloned(), 
                               BufferUsage::index_buffer(), 
                               self.window.get_queue())
                               .expect("failed to create immutable index buffer")
  }
  
  pub fn create_vertex(&self, verticies: iter::Cloned<slice::Iter<model_data::Vertex>>) -> Arc<BufferAccess + Send + Sync> {
      CpuAccessibleBuffer::from_iter(self.window.get_device(), 
                                     BufferUsage::vertex_buffer(), 
                                     verticies)
                                     .expect("failed to create vertex buffer")
  }
  
  pub fn create_index(&self, indicies: iter::Cloned<slice::Iter<u16>>) -> (Arc<ImmutableBuffer<[u16]>>,
                                                                           CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
      ImmutableBuffer::from_iter(indicies, BufferUsage::index_buffer(), 
                                 self.window.get_queue())
                                 .expect("failed to create immutable teapot index buffer")
  }
  
  pub fn create_texture_subbuffer(&self, draw: DrawCall) -> cpu_pool::CpuBufferPoolSubbuffer<vs_texture::ty::Data,
                                                                      Arc<memory::pool::StdMemoryPool>> {
    
    let model = DrawMath::calculate_texture_model(draw.get_translation(), draw.get_size(), -draw.get_x_rotation());
    
    let has_texture = {
      let mut value = 1.0;
      if draw.get_texture() == &String::from("") {
        value = 0.0;
      }
      value
    };
    
    let uniform_data = vs_texture::ty::Data {
      projection: self.projection_2d.into(),
      model: model.into(),
      colour: draw.get_colour().into(),
      has_texture: Vector4::new(has_texture, 0.0, 0.0, 0.0).into(),
    };
    self.uniform_buffer_texture.next(uniform_data).unwrap()
  }
  
  pub fn create_3d_subbuffer(&self, draw: DrawCall) -> cpu_pool::CpuBufferPoolSubbuffer<vs_3d::ty::Data, 
                                                                 Arc<memory::pool::StdMemoryPool>> {
    
    let rotation_x: Matrix4<f32> = Matrix4::from_angle_x(Deg(draw.get_x_rotation()));
    let rotation_y: Matrix4<f32> = Matrix4::from_angle_y(Deg(draw.get_y_rotation()));
    let rotation_z: Matrix4<f32> = Matrix4::from_angle_z(Deg(draw.get_z_rotation()));
         
    let transformation: Matrix4<f32> = (cgmath::Matrix4::from_translation(draw.get_translation())* cgmath::Matrix4::from_scale(draw.get_size().x)) * (rotation_x*rotation_y*rotation_z);
    
    let lighting_position: Matrix4<f32> =
      Matrix4::from_cols(
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
      // (R, G, B, n/a)
      Matrix4::from_cols(
        Vector4::new(0.0, 0.0, 1.0, 10.0), // colour + shinedamper
        Vector4::new(1.0, 0.0, 0.0, reflectivity),  // colour + reflectivity
        Vector4::new(0.4, 0.4, 0.4, -1.0), //sun
        Vector4::new(0.0, 0.0, 0.0, -1.0)
      );
    
    // (Intensity, 1)
    let attenuation: Matrix4<f32> =
      Matrix4::from_cols(
        Vector4::new(0.1, 0.25, 0.25, -1.0),
        Vector4::new(0.1, 0.25, 0.25, -1.0),
        Vector4::new(0.5, 0.0, 0.0, -1.0),
        Vector4::new(0.0, 0.0, 0.0, -1.0)
      );
    
    let uniform_data = vs_3d::ty::Data {
      transformation: transformation.into(),
      view : (self.view * self.scale).into(),
      proj : self.projection_3d.into(),
      lightpositions: lighting_position.into(),
      lightcolours: lighting_colour.into(),
      attenuations: attenuation.into(),
    };

    self.uniform_buffer_3d.next(uniform_data).unwrap()
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
    let (idx_3d_buffer, future_3d_idx) = self.create_index(model.get_indicies().iter().cloned()); 
    
    let model = Model {
      vertex_buffer: vec!(vert3d_buffer),
      index_buffer: idx_3d_buffer,
    };
    self.models.insert(reference, model);
    
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
    
    self.projection_2d = self.create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
    self.projection_3d = self.create_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
    
    self.depth_buffer = self.create_depth_buffer();
    
    // 2D
    let vert_buffer = self.create_2d_vertex();
    let (idx_buffer, future_idx) = self.create_2d_index();
    
    self.vertex_buffer_2d = Some(vec!(vert_buffer));
    self.index_buffer_2d = Some(idx_buffer);
    
    self.previous_frame_end = Some(Box::new(future_idx.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    
    let vs_3d = vs_3d::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_3d = fs_3d::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_texture = vs_texture::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_texture = fs_texture::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_text = vs_text::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_text = fs_text::Shader::load(self.window.get_device()).expect("failed to create shader module");
    
    self.render_pass = Some(Arc::new(single_pass_renderpass!(self.window.get_device(),
      attachments: {
        colour: {
          load: Clear,
          store: Store,
          format: self.window.get_swapchain().format(),
          samples: 1,
        },
        depth: {
          load: Clear,
          store: DontCare,
          format: format::Format::D16Unorm,
          samples: 1,
        }
      },
      pass: {
        color: [colour],
        depth_stencil: {depth}
      }
    ).unwrap()));
   
    self.pipeline_3d = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<model_data::Vertex>()
        .vertex_shader(vs_3d.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_3d.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .cull_mode_front()
        .render_pass(framebuffer::Subpass::from(self.render_pass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));

    self.pipeline_texture = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs_texture.main_entry_point(), ())
        .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_texture.main_entry_point(), ())
        //.blend_collective(pipeline::blend::AttachmentBlend::alpha_blending())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.render_pass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
        
    self.pipeline_text = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs_text.main_entry_point(), ())
        .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_text.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.render_pass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
   
    self.uniform_buffer_texture = cpu_pool::CpuBufferPool::<vs_texture::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    self.uniform_buffer_text = cpu_pool::CpuBufferPool::<vs_text::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    
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
      vertex_buffer: vec!(vert3d_buffer),
      index_buffer: idx_3d_buffer,
    };
    
    self.models.insert(String::from("terrain"), model);
    
    self.previous_frame_end = Some(Box::new(future_3d_idx.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
  }
  
  /// Initalises some variables
  fn init(&mut self) {    
    self.framebuffers = None;
    
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
    if self.recreate_swapchain {
      let dimensions = {
        self.window.get_dimensions()
      };
      
      let (new_swapchain, new_images) = match self.window.recreate_swapchain(dimensions) {
        Ok(r) => r,
        Err(SwapchainCreationError::UnsupportedDimensions) => {
          return;
        },
        Err(err) => panic!("{:?}", err)
      };
      
      self.window.replace_swapchain(new_swapchain);
      self.window.replace_images(new_images);
      
      self.framebuffers = None;
      self.recreate_swapchain = false;
      
      let new_depth_buffer = self.create_depth_buffer();
      mem::replace(&mut self.depth_buffer, new_depth_buffer);
      
      self.projection_2d = self.create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
      self.projection_3d = self.create_3d_projection(dimensions[0] as f32, dimensions[1] as f32);
    }
    
    if self.framebuffers.is_none() {
      let depth_buffer = self.depth_buffer.clone();
      
      let new_framebuffers = 
        Some(self.window.get_images().iter().map( |image| {
             let fb = framebuffer::Framebuffer::start(self.render_pass.clone().unwrap())
                      .add(image.clone()).unwrap()
                      .add(depth_buffer.clone().unwrap()).unwrap()
                      .build().unwrap();
             Arc::new(fb) as Arc<framebuffer::FramebufferAbstract + Send + Sync>
             }).collect::<Vec<_>>());
      mem::replace(&mut self.framebuffers, new_framebuffers);
    }
  }
  
  /// Draws everything that is in the drawcall passed to this function
  fn draw(&mut self, draw_calls: &Vec<DrawCall>) {
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
        
    //  tmp_cmd_buffer = build_start.begin_render_pass(self.framebuffers.as_ref().unwrap()[image_num].clone(), false, vec![[0.2, 0.3, 0.3, 1.0].into(), 1f32.into()]).unwrap(); 
      let clear = [self.clear_colour.x, self.clear_colour.y, self.clear_colour.z, self.clear_colour.w];   
      tmp_cmd_buffer = build_start.begin_render_pass(self.framebuffers.as_ref().unwrap()[image_num].clone(), false, vec![clear.into(), 1f32.into()]).unwrap();
      for draw in draw_calls {
        
        if draw.is_3d_model() {
          
          let uniform_buffer_subbuffer = self.create_3d_subbuffer(draw.clone());
          
          let mut texture: String = String::from("default");
          if self.textures.contains_key(draw.get_texture()) {
            texture = draw.get_texture().clone();
          }
          
          let set_3d = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.pipeline_3d.clone().unwrap(), 0)
                .add_buffer(uniform_buffer_subbuffer).unwrap()
                .add_sampled_image(self.textures.get(&texture).unwrap().clone(), self.sampler.clone()).unwrap()
                .build().unwrap()
          );
          
          {
            let cb = tmp_cmd_buffer;

            tmp_cmd_buffer = cb.draw_indexed(
                  self.pipeline_3d.clone().unwrap(),
                  DynamicState {
                        line_width: None,
                        viewports: Some(vec![pipeline::viewport::Viewport {
                            origin: [0.0, 0.0],
                            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                            depth_range: 0.0 .. 1.0,
                        }]),
                        scissors: None,
                  },
                  self.models.get(draw.get_texture()).expect("Invalid model name").vertex_buffer.clone(),
                  self.models.get(draw.get_texture()).expect("Invalid model name").index_buffer.clone(), set_3d.clone(), ()).unwrap();
          }
        } else {
          // Render Text
          if draw.get_text() != "" {
            let wrapped_draw = DrawMath::setup_correct_wrapping(draw.clone(), self.fonts.clone());
            let size = draw.get_x_size();
            
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
                  projection: self.projection_2d.into(),
                };
                self.uniform_buffer_text.next(uniform_data).unwrap()
              };
              
              let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.pipeline_text.clone().unwrap(), 0)
                                         .add_sampled_image(self.textures.get(draw.get_texture()).unwrap().clone(), self.sampler.clone()).unwrap()
                                         .add_buffer(uniform_buffer_text_subbuffer.clone()).unwrap()
                                         .build().unwrap());
              
              {
                let cb = tmp_cmd_buffer;
                tmp_cmd_buffer = cb.draw_indexed(self.pipeline_text.clone().unwrap(),
                                              DynamicState {
                                                      line_width: None,
                                                      viewports: Some(vec![Viewport {
                                                        origin: [0.0, 0.0],
                                                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                                        depth_range: 0.0 .. 1.0,
                                                      }]),
                                                      scissors: None,
                                              },
                                              self.vertex_buffer_2d.clone().unwrap(),
                                              self.index_buffer_2d.clone().unwrap(),
                                              uniform_set.clone(), ()).unwrap();
              
              
              }
            }
          } else {
            let uniform_buffer_texture_subbuffer = self.create_texture_subbuffer(draw.clone());
            
            // No Texture
            if draw.get_texture() == &String::from("") {
              let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.pipeline_texture.clone().unwrap(), 0)
                                         .add_sampled_image(self.textures.get("Arial").expect("Default texture not loaded!").clone(), self.sampler.clone()).unwrap()
                                         .add_buffer(uniform_buffer_texture_subbuffer.clone()).unwrap()
                                         .build().unwrap());
              
              {
                let cb = tmp_cmd_buffer;
                
                tmp_cmd_buffer = cb.draw_indexed(self.pipeline_texture.clone().unwrap(),
                                              DynamicState {
                                                      line_width: None,
                                                      viewports: Some(vec![Viewport {
                                                        origin: [0.0, 0.0],
                                                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                                        depth_range: 0.0 .. 1.0,
                                                      }]),
                                                      scissors: None,
                                              },
                                              self.vertex_buffer_2d.clone().unwrap(),
                                              self.index_buffer_2d.clone().unwrap(),
                                              uniform_set.clone(), ()).unwrap();
              }
            } else {
              // Texture
              let texture = draw.get_texture();
              
              let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.pipeline_texture.clone().unwrap(), 0)
                                      .add_sampled_image(self.textures.get(draw.get_texture()).expect(&("Unknown Texture".to_string() + texture)).clone(), self.sampler.clone()).unwrap()
                                      .add_buffer(uniform_buffer_texture_subbuffer.clone()).unwrap()
                                      .build().unwrap());
              
              {
                let cb = tmp_cmd_buffer;

                tmp_cmd_buffer = cb.draw_indexed(self.pipeline_texture.clone().unwrap(),
                                              DynamicState {
                                                      line_width: None,
                                                      viewports: Some(vec![Viewport {
                                                        origin: [0.0, 0.0],
                                                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                                        depth_range: 0.0 .. 1.0,
                                                      }]),
                                                      scissors: None,
                                              },
                                              self.vertex_buffer_2d.clone().unwrap(),
                                              self.index_buffer_2d.clone().unwrap(),
                                              uniform_set.clone(), ()).unwrap();
              }
            }
          }
        }
      }
      
      tmp_cmd_buffer.end_render_pass()
        .unwrap()
        .build().unwrap() as AutoCommandBuffer
    };
  
    let future = self.previous_frame_end.take().unwrap().join(acquire_future)
      .then_execute(self.window.get_queue(), command_buffer).unwrap()
      .then_swapchain_present(self.window.get_queue(), self.window.get_swapchain(), image_num)
      .then_signal_fence_and_flush().unwrap();
    
    // <hacks>
    /*    match future.wait(Some(Duration::from_millis(100))) {
            Ok(x) => x,  // type unit
            Err(err) => println!("err: {:?}", err), // never see this
        }*/
    // </hacks>
    
    self.previous_frame_end = Some(Box::new(future) as Box<_>);
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
    
    self.view = cgmath::Matrix4::look_at(cgmath::Point3::new(camera.x, camera.y, camera.z), cgmath::Point3::new(camera.x+x_rot, camera.y, camera.z+z_rot), cgmath::Vector3::new(0.0, -1.0, 0.0));
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


