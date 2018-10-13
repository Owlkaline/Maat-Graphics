use graphics::CoreRender;
use graphics::Vertex2d;
use graphics::Vertex3d;
use drawcalls::DrawCall;
use drawcalls::DrawType;

use settings::Settings;

use vulkan::FinalShader;
use vulkan::TextureShader;
use vulkan::ResourceManager;

use camera::Camera;
use font::GenericFont;
use window::VkWindow;

use vulkano::sync::now;
use vulkano::sync::NowFuture;
use vulkano::sync::GpuFuture;
use vulkano::sync::FlushError;
use vulkano::format::ClearValue;
use vulkano::pipeline::viewport::Viewport;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::swapchain::SwapchainCreationError;

use vulkano::image as vkimage;
use vulkano::format;

use winit;
use winit::dpi::LogicalSize;

use cgmath::Vector4;
use cgmath::Matrix4;

use std::collections::HashMap;

impl_vertex!(Vertex2d, position, uv);
impl_vertex!(Vertex3d, position, normal, tangent, uv, colour);

// NEEDS TO BE MOVED WHEN 3D is a thing
use cgmath::perspective;
use cgmath::Deg;

pub fn create_3d_projection(width: f32, height: f32) -> Matrix4<f32> {
  perspective(Deg(45.0), { width as f32 / height as f32 }, 0.1, 100.0)
}

pub struct VkMaat {
  camera: Camera,
  
  texture_projection: Matrix4<f32>,
  _model_projection: Matrix4<f32>,
  
  resources: ResourceManager,
  
  texture_shader: TextureShader,
  final_shader: FinalShader,
  
  samples: u32,
  
  clear_colour: ClearValue,
  dynamic_state: DynamicState,
  
  recreate_swapchain: bool,
  previous_frame_end: Option<Box<GpuFuture>>,
  
  window: VkWindow,
}

impl VkMaat {
  pub fn new() -> VkMaat {
    
    let mut settings = Settings::load();
    let dim = settings.get_resolution();
    let min_dim = settings.get_minimum_resolution();
    let fullscreen = settings.is_fullscreen();
    let vsync = settings.vsync_enabled();
    let triple_buffer = settings.triple_buffer_enabled();
    let samples = settings.get_msaa();
    
    let window = VkWindow::new(dim[0] as f64, dim[1] as f64, min_dim[0], min_dim[1], fullscreen, vsync, triple_buffer);
    
    let dim = {
      let logic_dim = window.get_dimensions();
      [(logic_dim.width as f32 * window.get_dpi_scale() as f32) as u32, (logic_dim.height as f32 * window.get_dpi_scale() as f32) as u32]
    };
    
    let device = window.get_device();
    let queue = window.get_queue();
    let swapchain_format = window.get_swapchain_format();
    
    let texture_projection = TextureShader::create_projection(dim[0] as f32 / window.get_dpi_scale() as f32, dim[1] as f32 / window.get_dpi_scale() as f32);
    
    let (texture_shader, future_textures) = TextureShader::create(device.clone(), queue.clone(), dim, samples, texture_projection.clone());
    let (final_shader, final_futures) = FinalShader::create(device.clone(), queue.clone(), swapchain_format);
    
    let mut resources = ResourceManager::new();
    let (empty_texture, empty_future) = vkimage::immutable::ImmutableImage::from_iter([0u8, 0u8, 0u8, 0u8].iter().cloned(),
                                            vkimage::Dimensions::Dim2d { width: 1, height: 1 },
                                            format::R8G8B8A8Unorm, window.get_queue())
                                            .expect("Failed to create immutable image");
    resources.insert_texture("empty".to_string(), empty_texture);
    
    let mut previous_frame_end = Some(Box::new(now(device.clone())) as Box<GpuFuture>);
    
    for future in future_textures {
      previous_frame_end = Some(Box::new(future.join(Box::new(previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    }
    for future in final_futures {
      previous_frame_end = Some(Box::new(future.join(Box::new(previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    }
    
    previous_frame_end = Some(Box::new(empty_future.join(Box::new(previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    
    let dynamic_state = DynamicState {
                          line_width: None,
                          viewports: Some(vec![Viewport {
                            origin: [0.0, 0.0],
                            dimensions: [dim[0] as f32, dim[1] as f32],
                            depth_range: 0.0 .. 1.0,
                          }]),
                          scissors: None,
                        };
    
    VkMaat {
      camera: Camera::default_vk(),
      
      texture_projection: texture_projection,
      _model_projection: create_3d_projection(dim[0] as f32, dim[1] as f32),
      
      resources: resources,
      
      texture_shader: texture_shader,
      final_shader: final_shader,
      
      samples: samples,
      
      clear_colour: ClearValue::Float([0.0, 0.0, 0.0, 1.0]),
      dynamic_state: dynamic_state,
      
      recreate_swapchain: false,
      previous_frame_end: previous_frame_end,
      
      window: window,
    }
  }
  
  pub fn with_title(mut self, title: String) -> VkMaat {
    self.window.set_title(title);
    self
  }
  
  pub fn gather_futures(&mut self, futures: Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>) {
    for future in futures {
      self.previous_frame_end = Some(Box::new(future.join(Box::new(self.previous_frame_end.take().unwrap()) as Box<GpuFuture>)) as Box<GpuFuture>);
    }
  }
  
  pub fn draw_with_secondary_buffers(&mut self, draw_calls: &Vec<DrawCall>, image_num: usize) -> AutoCommandBuffer {
    // draw_calls
    let dimensions = {
      let dim = self.window.get_dimensions();
      [dim.width as u32, dim.height as u32]
    };
    
    let mut texture_command_buffer = {
      let device = self.window.get_device();
      let family = self.window.get_queue_ref().family();
      
      self.texture_shader.create_secondary_renderpass(device, family)
    };
    
    for draw in draw_calls {
      let black_and_white = draw.is_black_and_white();
      match draw.get_type() {
        DrawType::DrawFont(ref info) => {
          let (font, display_text, position, scale, colour, outline_colour, edge_width, _wrapped, wrap_length, centered) = info.clone(); 
          
          let texture_resource = self.resources.get_font(font.clone());
          if let Some(font_info) = texture_resource {
            texture_command_buffer = self.texture_shader.draw_text(texture_command_buffer, &self.dynamic_state, display_text, font, position, scale, colour, outline_colour, edge_width, wrap_length, centered, font_info);
          }
        },
        DrawType::DrawTextured(ref info) => {
          let (reference, position, scale, rotation, alpha) = info.clone(); 
          
          let texture_resource = self.resources.get_texture(reference.clone());
          if let Some(texture) = texture_resource {
            texture_command_buffer = self.texture_shader.draw_texture(texture_command_buffer, &self.dynamic_state, position, scale, rotation, Some(Vector4::new(0.0, 0.0, 0.0, alpha)), black_and_white, true, texture, false, None);
          }
        },
        DrawType::DrawColoured(ref info) => {
          let (position, scale, colour, rotation) = info.clone(); 
          
          let texture_resource = self.resources.get_texture("empty".to_string());
          if let Some(texture) = texture_resource {
            texture_command_buffer = self.texture_shader.draw_texture(texture_command_buffer, &self.dynamic_state, position, scale, rotation, Some(colour), black_and_white, false, texture, false, None);
          }
        },
        DrawType::DrawModel => {
          
        },
        DrawType::DrawCustomShapeTextured(ref info) => {
          let (reference, texture, position, scale, rotation) = info.clone(); 
          
          let shape_resource = self.resources.get_shape(reference.clone());
          let texture_resource = self.resources.get_texture(texture.clone());
            if let Some(texture) = texture_resource {
              texture_command_buffer = self.texture_shader.draw_texture(texture_command_buffer, &self.dynamic_state, position, scale, rotation, None, black_and_white, true, texture, true, shape_resource);
          }
        },
        DrawType::DrawCustomShapeColoured(ref info) => {
          let (reference, position, scale, colour, rotation) = info.clone(); 
          
          let shape_resource = self.resources.get_shape(reference.clone());
          let texture_resource = self.resources.get_texture("empty".to_string());
            if let Some(texture) = texture_resource {
              texture_command_buffer = self.texture_shader.draw_texture(texture_command_buffer, &self.dynamic_state, position, scale, rotation, Some(colour), black_and_white, false, texture, true, shape_resource);
          }
        },
        DrawType::DrawInstancedColoured => {},
        DrawType::DrawInstancedModel => {},
        DrawType::NewShape => {
          
        },
        DrawType::UpdateShape(ref info) => {
          let (reference, vertex, index) = info.clone();
          let futures = self.resources.update_shape(reference, vertex, index, self.window.get_queue());
          self.gather_futures(futures);
        },
        DrawType::RemoveShape => {
        //  if let Some(shape_name) = draw.shape_name() {
       //     self.resources.remove_object(shape_name);
       //   }
        },
        DrawType::NewDrawcallSet => {
          
        },
        DrawType::DrawDrawcallSet => {
          
        },
        DrawType::RemoveDrawcallSet => {
          
        },
        DrawType::NewTexture(ref _info) => {
          
        },
        DrawType::NewFont => {
          
        },
        DrawType::NewModel => {
          
        },
        DrawType::LoadTexture(ref info) => {
          let reference = info.clone();
          self.resources.load_texture_from_reference(reference, self.window.get_queue());
        },
        DrawType::LoadFont(ref _info) => {
//          let reference = info.clone();
//          self.resources.load_font(reference);
        },
        DrawType::LoadModel => {
          
        },
        DrawType::UnloadTexture(ref _info) => {
//          let reference = info.clone();
//          self.resources.unload_texture(reference);
        },
        DrawType::UnloadFont(ref _info) => {
//          let reference = info.clone();
//          self.resources.unload_font(reference);
        },
        DrawType::UnloadModel => {
          
        },
        DrawType::SetTextureScale(ref scale) => {
          self.texture_shader.set_scale(scale.clone(), self.texture_projection);
        },
        _ => {}
      }
    }
    
    let texture_cmd_buffer = texture_command_buffer.build().unwrap();
      
    let command_buffer: AutoCommandBuffer = {
      let mut tmp_cmd_buffer = AutoCommandBufferBuilder::primary_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family()).unwrap();
      
      tmp_cmd_buffer = self.texture_shader.begin_renderpass(tmp_cmd_buffer, true, self.clear_colour);
      unsafe {
        tmp_cmd_buffer = tmp_cmd_buffer.execute_commands(texture_cmd_buffer).unwrap();
      }
      tmp_cmd_buffer = self.texture_shader.end_renderpass(tmp_cmd_buffer);
      tmp_cmd_buffer = self.final_shader.begin_renderpass(tmp_cmd_buffer, false, image_num);
      
      let texture_image = self.texture_shader.get_texture_attachment();
      tmp_cmd_buffer = self.final_shader.draw(tmp_cmd_buffer, &self.dynamic_state, [dimensions[0] as f32, dimensions[1] as f32], self.texture_projection, texture_image);
      
      self.final_shader.end_renderpass(tmp_cmd_buffer)
          .build().unwrap() as AutoCommandBuffer
    };
    
    command_buffer
  }
  
  pub fn draw_without_secondary_buffers(&mut self, draw_calls: &Vec<DrawCall>, image_num: usize) -> AutoCommandBuffer {
    // draw_calls
    let command_buffer: AutoCommandBuffer = {
      let mut dimensions = {
        let dim = self.window.get_dimensions();
        [dim.width as u32, dim.height as u32]
      };
      
      /*let mut texture_command_buffer = {
        let device = self.window.get_device();
        let family = self.window.get_queue_ref().family();
        
        self.texture_shader.create_secondary_renderpass(device, family)
      };*/
      
     let mut tmp_cmd_buffer = AutoCommandBufferBuilder::primary_one_time_submit(self.window.get_device(), self.window.get_queue_ref().family()).unwrap();
      
      tmp_cmd_buffer = self.texture_shader.begin_renderpass(tmp_cmd_buffer, false, self.clear_colour);
      
      for draw in draw_calls {
        let black_and_white = draw.is_black_and_white();
        match draw.get_type() {
        DrawType::DrawFont(ref info) => {
          let (font, display_text, position, scale, colour, outline_colour, edge_width, _wrapped, wrap_length, centered) = info.clone(); 
          
          let texture_resource = self.resources.get_font(font.clone());
          if let Some(font_info) = texture_resource {
            tmp_cmd_buffer = self.texture_shader.draw_text(tmp_cmd_buffer, &self.dynamic_state, display_text, font, position, scale, colour, outline_colour, edge_width, wrap_length, centered, font_info);
          }
        },
        DrawType::DrawTextured(ref info) => {
          let (reference, position, scale, rotation, alpha) = info.clone(); 
          
          let texture_resource = self.resources.get_texture(reference.clone());
          if let Some(texture) = texture_resource {
            tmp_cmd_buffer = self.texture_shader.draw_texture(tmp_cmd_buffer, &self.dynamic_state, position, scale, rotation, Some(Vector4::new(0.0, 0.0, 0.0, alpha)), black_and_white, true, texture, false, None);
          }
        },
        DrawType::DrawColoured(ref info) => {
          let (position, scale, colour, rotation) = info.clone(); 
          
          let texture_resource = self.resources.get_texture("empty".to_string());
          if let Some(texture) = texture_resource {
            tmp_cmd_buffer = self.texture_shader.draw_texture(tmp_cmd_buffer, &self.dynamic_state, position, scale, rotation, Some(colour), black_and_white, false, texture, false, None);
          }
        },
        DrawType::DrawModel => {
          
        },
        DrawType::DrawCustomShapeTextured(ref info) => {
          let (reference, texture, position, scale, rotation) = info.clone(); 
          
          let shape_resource = self.resources.get_shape(reference.clone());
          let texture_resource = self.resources.get_texture(texture.clone());
            if let Some(texture) = texture_resource {
              tmp_cmd_buffer = self.texture_shader.draw_texture(tmp_cmd_buffer, &self.dynamic_state, position, scale, rotation, None, black_and_white, true, texture, true, shape_resource);
          }
        },
        DrawType::DrawCustomShapeColoured(ref info) => {
          let (reference, position, scale, colour, rotation) = info.clone(); 
          
          let shape_resource = self.resources.get_shape(reference.clone());
          let texture_resource = self.resources.get_texture("empty".to_string());
            if let Some(texture) = texture_resource {
              tmp_cmd_buffer = self.texture_shader.draw_texture(tmp_cmd_buffer, &self.dynamic_state, position, scale, rotation, Some(colour), black_and_white, false, texture, true, shape_resource);
          }
        },
        DrawType::DrawInstancedColoured => {},
        DrawType::DrawInstancedModel => {},
        DrawType::NewShape => {
          
        },
        DrawType::UpdateShape(ref info) => {
          let (reference, vertex, index) = info.clone();
          let futures = self.resources.update_shape(reference, vertex, index, self.window.get_queue());
          self.gather_futures(futures);
        },
        DrawType::RemoveShape => {
        //  if let Some(shape_name) = draw.shape_name() {
       //     self.resources.remove_object(shape_name);
       //   }
        },
        DrawType::NewDrawcallSet => {
          
        },
        DrawType::DrawDrawcallSet => {
          
        },
        DrawType::RemoveDrawcallSet => {
          
        },
        DrawType::NewTexture(ref _info) => {
          
        },
        DrawType::NewFont => {
          
        },
        DrawType::NewModel => {
          
        },
        DrawType::LoadTexture(ref info) => {
          let reference = info.clone();
          self.resources.load_texture_from_reference(reference, self.window.get_queue());
        },
        DrawType::LoadFont(ref _info) => {
//          let reference = info.clone();
//          self.resources.load_font(reference);
        },
        DrawType::LoadModel => {
          
        },
        DrawType::UnloadTexture(ref _info) => {
//          let reference = info.clone();
//          self.resources.unload_texture(reference);
        },
        DrawType::UnloadFont(ref _info) => {
//          let reference = info.clone();
//          self.resources.unload_font(reference);
        },
        DrawType::UnloadModel => {
          
        },
        DrawType::SetTextureScale(ref scale) => {
          self.texture_shader.set_scale(scale.clone(), self.texture_projection);
        },
        _ => {}
      }
      }
      
     // let texture_cmd_buffer = texture_command_buffer.build().unwrap();
      
      //unsafe {
      //  tmp_cmd_buffer = tmp_cmd_buffer.execute_commands(texture_cmd_buffer).unwrap();
      //}
      tmp_cmd_buffer = self.texture_shader.end_renderpass(tmp_cmd_buffer);
      tmp_cmd_buffer = self.final_shader.begin_renderpass(tmp_cmd_buffer, false, image_num);
      
      let texture_image = self.texture_shader.get_texture_attachment();
      tmp_cmd_buffer = self.final_shader.draw(tmp_cmd_buffer, &self.dynamic_state, [dimensions[0] as f32, dimensions[1] as f32], self.texture_projection, texture_image);
      
      self.final_shader.end_renderpass(tmp_cmd_buffer)
          .build().unwrap() as AutoCommandBuffer
    };
    
    command_buffer
  }
}

impl CoreRender for VkMaat {
  // Load 3D models
  fn preload_model(&mut self, _reference: String, _location: String) {
    
  }
  
  fn add_model(&mut self, _reference: String, _location: String) {
    
  }
  
  /**
  ** Blocks current thread until resource is loaded onto the GPU
  **/
  fn preload_texture(&mut self, reference: String, location: String) {
    let queue = self.window.get_queue();
    let future = self.resources.sync_load_texture(reference, location, queue);
    self.gather_futures(vec!(future));
  }
  
  /**
  ** Adds Texture details into list allowing easier loading with a drawcall command
  **/
  fn add_texture(&mut self, reference: String, location: String) {
    self.resources.insert_unloaded_texture(reference, location);
  }
  
  // Load fonts
  fn preload_font(&mut self, reference: String, font_texture: String, font: &[u8]) {
    let futures = self.resources.sync_load_font(reference, font_texture, font, self.window.get_queue());
    self.gather_futures(vec!(futures));
  }
  
  fn add_font(&mut self, reference: String, font_texture: String, font: &[u8]) {
    //self.load_font(reference, font_texture, font);
    self.resources.insert_unloaded_font(reference, font_texture, font);
  }
  
  // Load custom goemetry
  fn load_static_geometry(&mut self, reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>) {
    let queue = self.window.get_queue();
    self.resources.load_shape(reference, vertex, index, queue);
  }
  
  fn load_dynamic_geometry(&mut self, reference: String, vertex: Vec<Vertex2d>, index: Vec<u32>) {
    let queue = self.window.get_queue();
    self.resources.load_shape(reference, vertex, index, queue);
  }
  
  // Internal use until Custom Shaders are implemented
  fn load_shaders(&mut self) {
    
  }
  
  // Initalises everything
  fn init(&mut self) {
    
  }
  
  /**
  ** Clears the framebuffer should be called in 98% of cases
  **/
  fn clear_screen(&mut self) {
    self.previous_frame_end.as_mut().unwrap().cleanup_finished();
  }
  
  fn pre_draw(&mut self) {
    let futures = self.resources.recieve_objects();
    self.gather_futures(futures);
    
    if self.recreate_swapchain {
      let mut dimensions = {
        let dim = self.window.get_dimensions();
        [(dim.width * self.window.get_dpi_scale()) as u32 , (dim.height * self.window.get_dpi_scale()) as u32]
      };
      
      if dimensions[0] <= 0 {
        dimensions[0] = 1;
      }
      if dimensions[1] <= 0 {
        dimensions[1] = 1;
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
      
      let device = self.window.get_device();
      let samples = self.samples;
      self.texture_projection = TextureShader::create_projection(dimensions[0] as f32 / self.window.get_dpi_scale() as f32, dimensions[1] as f32 / self.window.get_dpi_scale() as f32);
      
      self.texture_shader.recreate_framebuffer(device, dimensions, samples, self.texture_projection);
      self.final_shader.empty_framebuffer();
      
      self.dynamic_state.viewports = Some(
        vec![Viewport {
          origin: [0.0, 0.0],
          dimensions: [dimensions[0] as f32, dimensions[1] as f32],
          depth_range: 0.0 .. 1.0,
        }]
      );
      
      self.recreate_swapchain = false;
    }
    
    let images = self.window.get_images();
    self.final_shader.recreate_framebuffer(images);
  }
  
  /**
  ** Secondary command buffer removed as on amd
  **/
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
    
    let command_buffer;
    if self.window.gpu_is_amd() {
      command_buffer = self.draw_without_secondary_buffers(draw_calls, image_num);
    } else {
      command_buffer = self.draw_with_secondary_buffers(draw_calls, image_num);
    }
    
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
  
  fn post_draw(&self) {
    
  }
  
  fn screen_resized(&mut self, _window_size: LogicalSize) {
    self.recreate_swapchain = true;
  }
  
  // Cleans up program
  fn clean(&self) {
    
  }
  
  // Getters and setters
  fn get_dimensions(&self) -> LogicalSize {
    self.window.get_dimensions()
  }
  
  fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn get_fonts(&self) -> HashMap<String, GenericFont> {
    HashMap::new()
  }
  
  fn get_dpi_scale(&self) -> f64 {
    self.window.get_dpi_scale()
  }
  
  fn is_ready(&self) -> bool {
    self.resources.pending_objects_loaded()
  }
  
  fn set_cursor_position(&mut self, x: f32, y: f32) {
    self.window.set_cursor_position(x, y);
  }
  
  fn show_cursor(&mut self) {
    self.window.show_cursor();
  }
  
  fn hide_cursor(&mut self) {
    self.window.hide_cursor();
  }
  
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
    self.clear_colour = ClearValue::Float([r, g, b, a]);
  }
  
  fn set_camera(&mut self, camera: Camera) {
    self.camera = camera;
  }
  
  fn get_camera(&self) -> Camera {
    self.camera.to_owned()
  }
  
  fn num_drawcalls(&self) -> u32 {
    0
  }
}


