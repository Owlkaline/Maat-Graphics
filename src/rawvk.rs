use font::GenericFont;
use window::VkWindow;
use drawcalls::DrawCall;
use drawcalls::DrawMath;
use graphics::CoreRender;
use settings::Settings;

use image;
use winit;

use vulkano::image as vkimage;
use vulkano::sampler;

use vulkano::sync::now;
use vulkano::sync::GpuFuture;

use vulkano::swapchain;
use vulkano::swapchain::AcquireError;

use vulkano::buffer::cpu_pool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferAccess;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::ImmutableBuffer;

use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;

use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipelineAbstract;

use vulkano::format;
use vulkano::image::ImmutableImage;
use vulkano::descriptor::descriptor_set;
use vulkano::swapchain::SwapchainCreationError;

use std::mem;
use std::time;
use std::marker::Sync;
use std::marker::Send;
use std::collections::HashMap;
use std::sync::Arc;

use cgmath;
use cgmath::Matrix4;
use cgmath::SquareMatrix;

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

pub struct RawVk {
  ready: bool,
  fonts: HashMap<String, GenericFont>,
  textures: HashMap<String, Arc<ImmutableImage<format::R8G8B8A8Unorm>>>,
  texture_paths: HashMap<String, String>,

  framebuffers: Option<Vec<Arc<framebuffer::FramebufferAbstract + Send + Sync>>>,
  render_pass: Option<Arc<RenderPassAbstract + Send + Sync>>,

  projection: Matrix4<f32>,

  text_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,
  texture_pipeline: Option<Arc<GraphicsPipelineAbstract + Send + Sync>>,

  vertex_buffer: Option<Vec<Arc<BufferAccess + Send + Sync>>>,
  index_buffer: Option<Arc<ImmutableBuffer<[u16]>>>,

  texture_uniform_buffer: cpu_pool::CpuBufferPool<vs_texture::ty::Data>,
  text_uniform_buffer: cpu_pool::CpuBufferPool<vs_text::ty::Data>,

  pub window: VkWindow,
  sampler: Arc<sampler::Sampler>,

  recreate_swapchain: bool,
  
  previous_frame_end: Option<Box<GpuFuture>>,
  
  empty_buffer: Option<Vec<Arc<BufferAccess + Send + Sync>>>,
}

impl RawVk {
  pub fn new() -> RawVk {
    let mut settings = Settings::load();
    let width = settings.get_resolution()[0];
    let height = settings.get_resolution()[1];
    let min_width = settings.get_minimum_resolution()[0];
    let min_height = settings.get_minimum_resolution()[1];
    let fullscreen = settings.is_fullscreen();
    
    let window = VkWindow::new(width, height, min_width, min_height, fullscreen);
    
    let proj = Matrix4::identity();//create_2d_projection(width, height);
        
    let sampler = sampler::Sampler::new(window.get_device(), sampler::Filter::Linear,
                                                   sampler::Filter::Linear, 
                                                   sampler::MipmapMode::Nearest,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   0.0, 1.0, 0.0, 0.0).unwrap();
 
    let text_uniform = cpu_pool::CpuBufferPool::new(window.get_device(), BufferUsage::uniform_buffer());
    let texture_uniform = cpu_pool::CpuBufferPool::new(window.get_device(), BufferUsage::uniform_buffer());
    let previous_frame_end = Some(Box::new(now(window.get_device())) as Box<GpuFuture>);
    
    RawVk {
      ready: false,
      fonts: HashMap::new(),
      textures: HashMap::new(),
      texture_paths: HashMap::new(),

      projection: proj,

      window: window,
      
      vertex_buffer: None,
      empty_buffer: None,
      index_buffer: None,

      texture_uniform_buffer: texture_uniform,
      text_uniform_buffer: text_uniform,
      texture_pipeline: None,
      text_pipeline: None,

      render_pass: None,
      sampler: sampler,

      recreate_swapchain: false,
      framebuffers: None,
      previous_frame_end: previous_frame_end,
    }
  }
  
  pub fn create_2d_projection(&self, width: f32, height: f32) -> Matrix4<f32> {
    cgmath::ortho(0.0, width, height, 0.0, -1.0, 1.0)
  }
}

impl CoreRender for RawVk {
  fn add_texture(&mut self, reference: String, location: String) {
    self.texture_paths.insert(reference, location);
  }
  
  fn add_font(&mut self, reference: String, font: &[u8], font_texture: String) {
    self.load_font(reference.clone(), font);
    self.texture_paths.insert(reference, font_texture);
  }
  
  fn pre_load_texture(&mut self, reference: String, location: String) {
    self.load_texture(reference, location);
  }
  
  fn pre_load_font(&mut self, reference: String, font: &[u8], font_texture: String) {
    self.load_font(reference.clone(), font);    
    self.load_texture(reference, font_texture);
  }
  
  fn load_texture(&mut self, reference: String, location: String) {
    let texture_start_time = time::Instant::now();
    
    let (texture, tex_future) = {
      let image = image::open(&location).unwrap().to_rgba(); 
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
  
  fn load_font(&mut self, reference: String, font: &[u8]) {
   let mut new_font = GenericFont::new();
    new_font.load_font(font);
    
    self.fonts.insert(reference.clone(), new_font);
  }
  
  fn load_shaders(&mut self) {
    let dimensions = {
      self.window.get_dimensions()
    };
    
    self.projection = self.create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
    
    let square = {
      [
          Vertex { position: [  0.5 ,   0.5 ], uv: [1.0, 0.0] },
          Vertex { position: [ -0.5,    0.5 ], uv: [0.0, 0.0] },
          Vertex { position: [ -0.5,   -0.5 ], uv: [0.0, 1.0] },
          Vertex { position: [  0.5 ,  -0.5 ], uv: [1.0, 1.0] },
      ]
    };
    
    let empty = {
      [   Vertex { position: [  0.0 ,   0.0 ], uv: [0.0, 0.0] },]
    };
    
    let empty_buffer = {
      CpuAccessibleBuffer::from_iter(self.window.get_device(), BufferUsage::vertex_buffer(), empty.iter().cloned()).expect("failed to create vertex buffer")
    };    
    /*
    for i in [0, 1].iter().cloned() { // [0, 1, 0, 1] will prevent issue
      let vector: Vec<Vertex> = vec!(
          Vertex { position: [  0.5 ,   0.5 ], uv: [1.0, 0.0] },
          Vertex { position: [ -0.5,    0.5 ], uv: [0.0, 0.0] },
          Vertex { position: [ -0.5,   -0.5 ], uv: [0.0, 1.0] },
          Vertex { position: [  0.5 ,  -0.5 ], uv: [1.0, 1.0] },
        ).iter().cycle().take(i * 4).cloned().collect();*/
            
    let indicies: [u16; 6] = [1, 2, 3, 0, 3, 1];
    
    let vert_buffer = {
      CpuAccessibleBuffer::from_iter(self.window.get_device(), BufferUsage::vertex_buffer(), square.iter().cloned()).expect("failed to create vertex buffer")
    };    
/*    let (vert_buffer, future_vertex) = {
      ImmutableBuffer::from_iter(square.iter().cloned(), BufferUsage::vertex_buffer(), self.window.get_queue()).expect("failed to create immutable vertex buffer")
    };*/
    
    let (idx_buffer, future_idx) = {
      ImmutableBuffer::from_iter(indicies.iter().cloned(), BufferUsage::index_buffer(), self.window.get_queue()).expect("failed to create immutable index buffer")
    };
    
    self.vertex_buffer = Some(vec!(vert_buffer));
    self.index_buffer = Some(idx_buffer);
    self.empty_buffer = Some(vec!(empty_buffer));
    
    self.previous_frame_end = Some(Box::new(future_idx.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    
    let vs_texture = vs_texture::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_texture = fs_texture::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let vs_text = vs_text::Shader::load(self.window.get_device()).expect("failed to create shader module");
    let fs_text = fs_text::Shader::load(self.window.get_device()).expect("failed to create shader module");
    
    self.render_pass = Some(Arc::new(single_pass_renderpass!(self.window.get_device(),
      attachments: {
        colour: {
          load: Clear,
          store: Store,
          format: self.window.get_swapchain_format(),
          samples: 1,
        }
      },
      pass: {
        color: [colour],
        depth_stencil: {}
      }
    ).unwrap()));
     
    self.texture_pipeline = Some(Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs_texture.main_entry_point(), ())
        .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_texture.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.render_pass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
        
    self.text_pipeline = Some(Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs_text.main_entry_point(), ())
        .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_text.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.render_pass.clone().unwrap(), 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
   
    self.texture_uniform_buffer = cpu_pool::CpuBufferPool::<vs_texture::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
    
    self.text_uniform_buffer = cpu_pool::CpuBufferPool::<vs_text::ty::Data>::new(self.window.get_device(), BufferUsage::uniform_buffer());
  }
  
  fn init(&mut self) {    
    self.framebuffers = None;
    
    self.recreate_swapchain = false;
  }
  
  fn dynamic_load(&mut self) {
    let mut delta_time;
    let frame_start_time = time::Instant::now();
  
    let mut loaded_a_image = false;
    
    let cloned_paths = self.texture_paths.clone();
    
    for (reference, path) in &cloned_paths {
      self.load_texture(reference.clone(), path.clone());
      
      self.texture_paths.remove(reference);
      loaded_a_image = true;
      
      delta_time = frame_start_time.elapsed().subsec_nanos() as f64 / 1000000000.0 as f64;
      if (delta_time*1000f64) > 9.0 {
        break;
      } 
    }
    
    if !loaded_a_image {
      self.ready = true;
    }
  }
  
  fn clear_screen(&mut self) {
    self.previous_frame_end.as_mut().unwrap().cleanup_finished();
  }
  
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
      
      self.projection = self.create_2d_projection(dimensions[0] as f32, dimensions[1] as f32);
    }
    
    if self.framebuffers.is_none() {
      let new_framebuffers = 
        Some(self.window.get_images().iter().map( |image| {
             let fb = framebuffer::Framebuffer::start(self.render_pass.clone().unwrap())
                      .add(image.clone()).unwrap()
                      .build().unwrap();
             Arc::new(fb) as Arc<framebuffer::FramebufferAbstract + Send + Sync>
             }).collect::<Vec<_>>());
      mem::replace(&mut self.framebuffers, new_framebuffers);
    }
  }
  
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
        
      tmp_cmd_buffer = build_start.begin_render_pass(self.framebuffers.as_ref().unwrap()[image_num].clone(), false, vec![[0.2, 0.3, 0.3, 1.0].into(), 1f32.into()]).unwrap();
      
      // fill first chunk with empty
    /*  {
        let mut cb = tmp_cmd_buffer;
        
        let model = DrawMath::calculate_texture_model(Vector3::new(0.0, 0.0, 0.0), Vector2::new(0.0, 0.0));
          
        let uniform_buffer_subbuffer = {
          let uniform_data = vs_texture::ty::Data {
            colour: Vector4::new(0.0, 0.0, 0.0, 0.0).into(),
            model: model.into(),
            projection: self.projection.into(),
          };
          self.texture_uniform_buffer.next(uniform_data).unwrap()
        };
          
        // No Texture
        let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.texture_pipeline.clone().unwrap(), 0)
                                   .add_sampled_image(self.textures.get("Candara").unwrap().clone(), self.sampler.clone()).unwrap()
                                   .add_buffer(uniform_buffer_subbuffer.clone()).unwrap()
                                   .build().unwrap());
                                   
        tmp_cmd_buffer = cb.draw(self.texture_pipeline.clone().unwrap(),
                                            DynamicState {
                                                    line_width: None,
                                                    viewports: Some(vec![Viewport {
                                                      origin: [0.0, 0.0],
                                                      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                                      depth_range: 0.0 .. 1.0,
                                                    }]),
                                                    scissors: None,
                                            },
                                            self.empty_buffer.clone().unwrap(),
                                            uniform_set.clone(), ()).unwrap();
      }*/
      
      for draw in draw_calls {
        let mut cb = tmp_cmd_buffer;
        
        if draw.get_text() != "" {         
          let wrapped_draw = DrawMath::setup_correct_wrapping(draw.clone(), self.fonts.clone());
          let size = draw.get_x_size();
          
          for letter in wrapped_draw {
            let cmd_tmp = cb;
            
            let char_letter = {
              letter.get_text().as_bytes()[0] 
            };
            
            let c = self.fonts.get(draw.get_texture()).unwrap().get_character(char_letter as i32);

            let model = DrawMath::calculate_text_model(letter.get_translation(), size, &c.clone(), char_letter);
            let letter_uv = DrawMath::calculate_text_uv(&c.clone());
            let colour = letter.get_colour();
            let outline = letter.get_outline_colour();
            let edge_width = letter.get_edge_width(); 
             
            let text_uniform_buffer_subbuffer = {
              let uniform_data = vs_text::ty::Data {
                outlineColour: outline.into(),
                colour: colour.into(),
                edge_width: edge_width.into(),
                letter_uv: letter_uv.into(),
                model: model.into(),
                projection: self.projection.into(),
              };
              self.text_uniform_buffer.next(uniform_data).unwrap()
             };
            
            let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.text_pipeline.clone().unwrap(), 0)
                                       .add_sampled_image(self.textures.get(draw.get_texture()).unwrap().clone(), self.sampler.clone()).unwrap()
                                       .add_buffer(text_uniform_buffer_subbuffer.clone()).unwrap()
                                       .build().unwrap());
            
            cb = cmd_tmp.draw_indexed(self.text_pipeline.clone().unwrap(),
                                            DynamicState {
                                                    line_width: None,
                                                    viewports: Some(vec![Viewport {
                                                      origin: [0.0, 0.0],
                                                      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                                      depth_range: 0.0 .. 1.0,
                                                    }]),
                                                    scissors: None,
                                            },
                                            self.vertex_buffer.clone().unwrap(),
                                            self.index_buffer.clone().unwrap(),
                                            uniform_set.clone(), ()).unwrap();
          }
          tmp_cmd_buffer = cb;
        } else {
          
          let model = DrawMath::calculate_texture_model(draw.get_translation(), draw.get_size());
          
          let uniform_buffer_subbuffer = {
            let uniform_data = vs_texture::ty::Data {
              colour: draw.get_colour().into(),
              model: model.into(),
              projection: self.projection.into(),
            };
            self.texture_uniform_buffer.next(uniform_data).unwrap()
          };
          
          // No Texture
          if draw.get_texture() == &String::from("") {
            let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.texture_pipeline.clone().unwrap(), 0)
                                       .add_sampled_image(self.textures.get("Candara").unwrap().clone(), self.sampler.clone()).unwrap()
                                       .add_buffer(uniform_buffer_subbuffer.clone()).unwrap()
                                       .build().unwrap());
            
            tmp_cmd_buffer = cb.draw_indexed(self.texture_pipeline.clone().unwrap(),
                                            DynamicState {
                                                    line_width: None,
                                                    viewports: Some(vec![Viewport {
                                                      origin: [0.0, 0.0],
                                                      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                                      depth_range: 0.0 .. 1.0,
                                                    }]),
                                                    scissors: None,
                                            },
                                            self.vertex_buffer.clone().unwrap(),
                                            self.index_buffer.clone().unwrap(),
                                            uniform_set.clone(), ()).unwrap();
          } else {
            // Texture
            let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(self.texture_pipeline.clone().unwrap(), 0)
                                    .add_sampled_image(self.textures.get(draw.get_texture()).expect("Unknown Texture").clone(), self.sampler.clone()).unwrap()
                                    .add_buffer(uniform_buffer_subbuffer.clone()).unwrap()
                                    .build().unwrap());
          
            tmp_cmd_buffer = cb.draw_indexed(self.texture_pipeline.clone().unwrap(),
                                            DynamicState {
                                                    line_width: None,
                                                    viewports: Some(vec![Viewport {
                                                      origin: [0.0, 0.0],
                                                      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                                      depth_range: 0.0 .. 1.0,
                                                    }]),
                                                    scissors: None,
                                            },
                                            self.vertex_buffer.clone().unwrap(),
                                            self.index_buffer.clone().unwrap(),
                                            uniform_set.clone(), ()).unwrap();
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
      
      
    self.previous_frame_end = Some(Box::new(future) as Box<_>);
  }
  
  fn screen_resized(&mut self) {
    self.recreate_swapchain = true;
  }
  
  fn get_dimensions(&self) -> [u32; 2] {
    let dimensions: [u32; 2] = self.window.get_dimensions();
    dimensions
  }
  
  fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn get_fonts(&self) -> HashMap<String, GenericFont> {
    self.fonts.clone()
  }
  
  fn get_dpi_scale(&self) -> f32 {
    self.window.get_dpi_scale()
  }
  
  fn is_ready(&self) -> bool {
    self.ready
  }
  
  fn show_cursor(&mut self) {
    self.window.show_cursor();
  }
  
  fn hide_cursor(&mut self) {
    self.window.hide_cursor();
  }
  
  fn post_draw(&self) {}
  fn clean(&self) {}
  fn swap_buffers(&mut self) {}
}


