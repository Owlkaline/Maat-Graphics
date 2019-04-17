use vk;
use winit;
use cgmath::{Vector4};
use winit::dpi::LogicalSize;

//use crate::imgui_sys;
//use crate::imgui::{ImGui, Ui, ImFontConfig};
use crate::imgui::*;
use crate::imgui_winit_support;

use crate::ResourceManager;
use crate::camera::Camera;
use crate::drawcalls::DrawCall; 
use crate::drawcalls::DrawType;
use crate::graphics::CoreRender;
use crate::font::GenericFont;
use crate::TextureShader;
use crate::ModelShader;
use crate::FinalShader;
use crate::graphics;
use crate::Settings;

use crate::vulkan::vkenums::{ImageType, ImageViewType, ImageTiling, SampleCount, Filter, AddressMode, 
                             MipmapMode, VkBool, CommandBufferLevel, BorderColour};

use crate::vulkan::{ClearValues, VkWindow, Device, ImageAttachment, Sampler, SamplerBuilder, Compute};
use crate::vulkan::pool::{CommandPool, DescriptorPool, DescriptorPoolBuilder};
use crate::vulkan::sync::{Semaphore, Fence};
use crate::vulkan::buffer::{CommandBuffer, CommandBufferBuilder};
use crate::vulkan::check_errors;

use cgmath::{Vector2, Vector3};

use std::mem;
use std::ptr;
use std::sync::Arc;
use std::collections::HashMap;

pub struct CoreMaat {
  window: VkWindow,
  window_dimensions: vk::Extent2D,
  recreate_swapchain: bool,
  fences: Vec<Fence>,
  semaphore_image_available: Vec<Semaphore>,
  semaphore_render_finished: Vec<Semaphore>,
  command_pool: CommandPool,
  command_buffers: Vec<Arc<CommandBuffer>>,
  descriptor_set_pool: DescriptorPool,
  
  texture_clear_colour: Vec<vk::ClearValue>,
  model_clear_colour: Vec<vk::ClearValue>,
  final_clear_colour: Vec<vk::ClearValue>,
  
  dummy_image: ImageAttachment,
  
  sampler: Sampler,
  
  compute_shader: Option<Compute>,
  texture_shader: TextureShader,
  model_shader: ModelShader,
  final_shader: FinalShader,
  
  settings: Settings,
  resources: ResourceManager,
  
  current_frame: usize,
  max_frames: usize,
  
  image_from_draw: Option<ImageAttachment>,
  
  imgui_sampler: Option<Sampler>,
  imgui: Option<ImGui>,
  
  mouse_position: Vector2<f32>,
  dpi: f32,
}

impl CoreMaat {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> CoreMaat {
    let mut settings = Settings::load(Vector2::new(800, 600), Vector2::new(width as i32, height as i32));
    let window = VkWindow::new(app_name, app_version, should_debug, &settings);
    
    let resource_manager = ResourceManager::new();
    
    let fences: Vec<Fence>;
    let mut semaphore_image_available: Vec<Semaphore> = Vec::new();
    let mut semaphore_render_finished: Vec<Semaphore> = Vec::new();
    let command_pool: CommandPool;
    let command_buffers: Vec<Arc<CommandBuffer>>;
    let descriptor_set_pool: DescriptorPool;
    
    let texture_shader: TextureShader;
    let model_shader: ModelShader;
    let final_shader: FinalShader;
    
    let sampler: Sampler;
    
    let current_extent = window.get_current_extent();
    
    let dummy_image;
    
    let desired_texture_msaa = settings.get_texture_msaa();
    let desired_model_msaa = settings.get_model_msaa();
    
    let texture_msaa;
    let model_msaa;
    
    let mut compute_shader = None;
    
    {
      let instance = window.instance();
      let device = window.device();
      let format = window.swapchain_format();
      let graphics_family = window.get_graphics_family();
      let graphics_queue = window.get_graphics_queue();
      let image_views = window.swapchain_image_views();
      
      let max_msaa = window.get_max_mssa();
      println!("Max Msaa possible: {}", max_msaa);
      
      if desired_texture_msaa < max_msaa {
        texture_msaa = SampleCount::from(desired_texture_msaa);
      } else {
        texture_msaa = SampleCount::from(max_msaa);
      }
      
      if desired_model_msaa < max_msaa {
        model_msaa = SampleCount::from(desired_model_msaa);
      } else {
        model_msaa = SampleCount::from(max_msaa);
      }
      
      settings.set_texture_msaa(texture_msaa.to_bits());
      settings.set_model_msaa(model_msaa.to_bits());
      println!("Using Msaa: {}x for 2D", texture_msaa.to_bits());
      println!("Using Msaa: {}x for 3D", model_msaa.to_bits());
      
      for _ in 0..image_views.len() {
        semaphore_image_available.push(Semaphore::new(Arc::clone(&device)));
        semaphore_render_finished.push(Semaphore::new(Arc::clone(&device)));
      }
      
      fences = CoreMaat::create_fences(Arc::clone(&device), image_views.len() as u32);
      command_pool = CommandPool::new(Arc::clone(&device), graphics_family);
      command_buffers = command_pool.create_command_buffers(Arc::clone(&device), image_views.len() as u32);
      
      //descriptor_set_pool = DescriptorPool::new(Arc::clone(&device), image_views.len() as u32, 80, 80);
      descriptor_set_pool = DescriptorPoolBuilder::new()
                              .add_combined_image_samplers(80)
                              .add_uniform_buffers(80)
                              .add_storage_images(2)
                              .build(Arc::clone(&device), image_views.len() as u32);
      
      dummy_image = ImageAttachment::create_dummy_texture(Arc::clone(&instance), Arc::clone(&device), &ImageType::Type2D, &ImageTiling::Optimal, &SampleCount::OneBit, &ImageViewType::Type2D, vk::FORMAT_R8G8B8A8_UNORM, &command_pool, graphics_queue);
      
      sampler = SamplerBuilder::new()
                       .min_filter(Filter::Linear)
                       .mag_filter(Filter::Linear)
                       .address_mode(AddressMode::ClampToEdge)
                       .mipmap_mode(MipmapMode::Nearest)
                       .anisotropy(VkBool::True)
                       .max_anisotropy(8.0)
                       .build(Arc::clone(&device));
      
     // compute_shader = Compute::new(Arc::clone(&instance), Arc::clone(&device), &dummy_image, &descriptor_set_pool, image_views.len() as u32);
      
      texture_shader = TextureShader::new(Arc::clone(&instance), Arc::clone(&device), &current_extent, &format, &sampler, image_views, &dummy_image, &descriptor_set_pool, &command_pool, graphics_queue, &texture_msaa);
      model_shader = ModelShader::new(Arc::clone(&instance), Arc::clone(&device), &current_extent, &format, &sampler, image_views, &dummy_image, &descriptor_set_pool, &command_pool, graphics_queue, &model_msaa);
      final_shader = FinalShader::new(Arc::clone(&instance), Arc::clone(&device), &current_extent, &format, &sampler, image_views, &dummy_image, &descriptor_set_pool, &command_pool, graphics_queue);
      
      let mut model_images = Vec::with_capacity(image_views.len());
      for i in 0..image_views.len() {
        model_images.push(model_shader.get_texture_ref(i));
      }
      //compute_shader.build(Arc::clone(&device), *graphics_queue as u32, model_images);
    }
    
    let max_frames = fences.len();
    
    let default_clear_colour = vec!(vk::ClearValue {
      color: vk::ClearColorValue { float32: [0.0, 0.0, 0.2, 1.0] }
    });
    
    let model_clear_colour = if model_msaa != SampleCount::OneBit {
      vec!(
        vk::ClearValue {
          color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] }
        },
        vk::ClearValue {
          color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] }
        },
        vk::ClearValue {
          depthStencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 }
        }
        
      )
    } else {
      vec!(
        vk::ClearValue {
          color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] }
        },
        vk::ClearValue {
          depthStencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 }
        }
      )
    };
    
    CoreMaat {
      window: window,
      window_dimensions: current_extent,
      recreate_swapchain: false,
      fences,
      semaphore_image_available,
      semaphore_render_finished,
      command_pool,
      command_buffers,
      descriptor_set_pool,
      
      texture_clear_colour:
        vec!(
          vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] }
          },
          vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] }
          },
        ),
      model_clear_colour,
      final_clear_colour: default_clear_colour,
      
      dummy_image,
      sampler,
      
      compute_shader,
      texture_shader,
      model_shader,
      final_shader,
      
      settings,
      resources: resource_manager,
      
      current_frame: 0,
      max_frames,
      
      image_from_draw: None,
      
      imgui_sampler: None,
      imgui: None,
      mouse_position: Vector2::new(0.0, 0.0),
      dpi: 1.0,
    }
  }
  
  pub fn use_imgui(mut self) -> CoreMaat {
    let instance = self.window.instance();
    let device = self.window.device();
    let graphics_queue = self.window.get_graphics_queue();
    
    let mut imgui = ImGui::init();
    {
      fn imgui_gamma_to_linear(col: ImVec4) -> ImVec4 {
        let x = col.x.powf(2.2);
        let y = col.y.powf(2.2);
        let z = col.z.powf(2.2);
        let w = 1.0 - (1.0 - col.w).powf(2.2);
        
        ImVec4::new(x, y, z, w)
      }
      
      let style = imgui.style_mut();
      style.colors[9] = ImVec4{ x: 1.0, y: 0.0, z: 0.0, w: 0.6 };
      style.colors[11] = ImVec4{ x: 1.0, y: 0.0, z: 0.0, w: 0.8 };
      style.colors[12] = ImVec4{ x: 1.0, y: 0.0, z: 0.0, w: 0.4 };
      style.colors[24] = ImVec4{ x: 1.0, y: 0.0, z: 0.0, w: 0.4 };
      style.colors[18] = ImVec4{ x: 0.0, y: 1.0, z: 0.0, w: 1.0 };
      
      for col in 0..style.colors.len() {
        style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
      }
    }
    
    imgui.set_ini_filename(None);
    
    imgui.fonts().add_default_font_with_config(
        ImFontConfig::new()
          //  .oversample_h(1)
          //  .pixel_snap_h(true)
           // .size_pixels(13.0),
    );
    //imgui.igStyleColorsDark();
    imgui.set_font_global_scale(1.2);
    
    self.texture_shader.load_imgui(Arc::clone(&instance), Arc::clone(&device), self.max_frames as u32);
    
    self.resources.load_imgui(Arc::clone(&instance), Arc::clone(&device), &mut imgui, &self.command_pool, *graphics_queue);
    
    imgui_winit_support::configure_keys(&mut imgui);
    
    let sampler = SamplerBuilder::new()
                       .min_filter(Filter::Linear)
                       .mag_filter(Filter::Linear)
                       .address_mode(AddressMode::ClampToEdge)
                       .mipmap_mode(MipmapMode::Linear)
                       .anisotropy(VkBool::False)
                       .max_anisotropy(1.0)
                       .border_colour(BorderColour::FloatOpaqueWhite)
                       .build(Arc::clone(&device));
    
    self.imgui = Some(imgui);
    self
  }
  
  fn create_fences(device: Arc<Device>, num_fences: u32) -> Vec<Fence> {
    let mut fences: Vec<Fence> = Vec::with_capacity(num_fences as usize);
    
    for _ in 0..num_fences {
      let fence: Fence = Fence::new(Arc::clone(&device));
      fences.push(fence);
    }
    
    fences
  }
}

impl CoreRender for CoreMaat {
  fn preload_model(&mut self, reference: String, location: String) {
    self.resources.sync_load_model(reference, location);
  }
  
  fn add_model(&mut self, reference: String, location: String) {
    self.resources.insert_unloaded_model(reference, location);
  }
  
  fn preload_texture(&mut self, reference: String, location: String) {
    let graphics_queue = self.window.get_graphics_queue();
    let device = self.window.device();
    let instance = self.window.instance();
    self.resources.sync_load_texture(reference.to_string(), location, Arc::clone(&device), Arc::clone(&instance), &self.command_pool, *graphics_queue);
    self.texture_shader.add_texture(Arc::clone(&device), &self.descriptor_set_pool, reference.to_string(), &self.resources.get_texture(reference).unwrap(), &self.sampler);
  }
  
  fn add_texture(&mut self, reference: String, location: String) {
    self.resources.insert_unloaded_texture(reference, location);
  }
  
  fn preload_font(&mut self, reference: String, font_texture: String, font: &[u8]) {
    let graphics_queue = self.window.get_graphics_queue();
    let device = self.window.device();
    let instance = self.window.instance();
    
    self.resources.sync_load_font(reference.to_string(), font_texture.to_string(), font, Arc::clone(&device), Arc::clone(&instance), &self.command_pool, *graphics_queue);
    
    self.texture_shader.add_texture(Arc::clone(&device), &self.descriptor_set_pool, reference.to_string(), &self.resources.get_font(reference).unwrap().1, &self.sampler);
  }
  
  fn add_font(&mut self, _reference: String, _font_texture: String, _font: &[u8]) {
    
  }
  
  fn create_instance_buffer(&mut self, reference: String) {
    let device = self.window.device();
    let instance = self.window.instance();
    let num_frames = self.fences.len() as u32;
    self.texture_shader.add_instanced_buffer(Arc::clone(&instance), Arc::clone(&device), num_frames, reference);
  }
  
  fn create_model_instance_buffer(&mut self, reference: String) {
    let device = self.window.device();
    let instance = self.window.instance();
    let num_frames = self.fences.len() as u32;
    self.model_shader.add_instanced_buffer(Arc::clone(&instance), Arc::clone(&device), num_frames, reference);
  }
  
  fn load_static_geometry(&mut self, _reference: String, _verticies: Vec<graphics::Vertex2d>, _indicies: Vec<u32>) {
    
  }
  
  fn load_dynamic_geometry(&mut self, _reference: String, _verticies: Vec<graphics::Vertex2d>, _indicies: Vec<u32>) {
    
  }
  
  fn load_shaders(&mut self) {
    
  }
  
  fn init(&mut self) {
    
  }
  
  fn pre_draw(&mut self) -> Vec<(String, Vector3<f32>)> {
    let mut model_details = Vec::new();
    
    {
      let graphics_queue = self.window.get_graphics_queue();
      let device = self.window.device();
      let instance = self.window.instance();
      
      let references: Vec<(String, Option<Vector3<f32>>)> = self.resources.recieve_objects(Arc::clone(&instance), Arc::clone(&device), ImageType::Type2D, ImageViewType::Type2D, &vk::FORMAT_R8G8B8A8_UNORM, SampleCount::OneBit, ImageTiling::Optimal, &self.command_pool, graphics_queue);
      
      for (reference, size) in references {
        if let Some(texture) = self.resources.get_texture(reference.to_string()) {
          if reference.to_string() == "imgui".to_string() {
            if let Some(sampler) = &self.imgui_sampler {
              self.texture_shader.add_texture(Arc::clone(&device), &self.descriptor_set_pool, reference.to_string(), &texture, sampler);
            }
          } else {
            self.texture_shader.add_texture(Arc::clone(&device), &self.descriptor_set_pool, reference.to_string(), &texture, &self.sampler);
          }
        }
        if let Some((Some(model), base_textures)) = self.resources.get_model(reference.to_string()) {
          self.model_shader.add_model(Arc::clone(&instance), Arc::clone(&device), reference.to_string(), model, base_textures, &self.dummy_image, &self.command_pool, &self.descriptor_set_pool, &self.sampler, graphics_queue);
        }
        
        if size.is_some() {
          model_details.push((reference, size.unwrap()));
        }
      }
    }
    
    if !self.recreate_swapchain {
      return model_details;
    }
    
    println!("Reszing window");
    self.recreate_swapchain = false;
    
    self.window.device().wait();
    
    for fence in &self.fences {
      let device = self.window.device();
      fence.wait(Arc::clone(&device));
      fence.destroy(Arc::clone(&device));
    }
    
    self.window.recreate_swapchain(&self.settings);
    self.window_dimensions = self.window.get_current_extent();
    
    for i in 0..self.command_buffers.len() {
      let device = self.window.device();
      self.command_buffers[i].free(Arc::clone(&device), &self.command_pool)
    }
    self.command_buffers.clear();
    
    {
      let device = self.window.device();
      let instance = self.window.instance();
      let image_views = self.window.swapchain_image_views();
      let textures = self.resources.get_all_textures();
      let format = self.window.swapchain_format();
      let graphics_queue = self.window.get_present_queue();
      
      self.fences = CoreMaat::create_fences(Arc::clone(&device), image_views.len() as u32);
      
      self.command_buffers = self.command_pool.create_command_buffers(Arc::clone(&device), image_views.len() as u32);
      
      self.texture_shader.recreate(Arc::clone(&instance), Arc::clone(&device), &format, image_views, &self.window_dimensions, textures.clone(), &self.sampler, &self.command_pool, graphics_queue);
      self.model_shader.recreate(Arc::clone(&instance), Arc::clone(&device), &format, image_views, &self.window_dimensions, &self.command_pool, graphics_queue);
      self.final_shader.recreate(Arc::clone(&device), image_views, &self.window_dimensions, textures, &self.sampler);
      
      // TO REMOVE
      for (reference, texture) in &self.resources.get_all_textures() {
        self.texture_shader.add_texture(Arc::clone(&device), &self.descriptor_set_pool, reference.to_string(), texture, &self.sampler);
      }
      
      self.max_frames = image_views.len();
      self.current_frame = 0;
    }
    
    self.draw(&Vec::new(), 0.0);
    
    self.window.device().wait();
    println!("Finished resize");
    
    model_details
  }
  
  fn draw(&mut self, draw_calls: &Vec<DrawCall>, delta_time: f32) {
    //
    // Build drawcalls
    //
    if self.recreate_swapchain {
      return;
    }
    
    let i = self.current_frame;
    let mut cmd = CommandBufferBuilder::primary_one_time_submit(Arc::clone(&self.command_buffers[i]));
    let mut model_draw_calls = Vec::new();
    let mut image_index = 0;
    {
      let device = self.window.device();
      let instance = self.window.instance();
      let graphics_queue = self.window.get_graphics_queue();
      let window_size = vk::Extent2D { width: self.window_dimensions.width, height: self.window_dimensions.height };
      self.fences[self.current_frame].wait(Arc::clone(&device));
      self.fences[self.current_frame].reset(Arc::clone(&device));
      
      self.model_shader.update_scanline(delta_time);
      
      //
      // Actually Draw stuff
      //
      let (result, local_image_index) = self.window.aquire_next_image(Arc::clone(&device), &self.semaphore_image_available[self.current_frame]);
      
      match result {
        vk::ERROR_OUT_OF_DATE_KHR => {
          self.recreate_swapchain = true;
          return;
        },
        e => {
          check_errors(e);
        }
      }
      
      image_index = local_image_index;
      
      cmd = cmd.begin_command_buffer(Arc::clone(&device));
      cmd = self.texture_shader.fill_buffers(Arc::clone(&instance), Arc::clone(&device), cmd, i);
      cmd = self.texture_shader.begin_renderpass(Arc::clone(&device), cmd, &self.texture_clear_colour, &window_size, i);
      
      cmd = cmd.set_viewport(Arc::clone(&device), 0.0, 0.0, window_size.width as f32, window_size.height as f32);
      cmd = cmd.set_scissor(Arc::clone(&device), 0, 0, window_size.width, window_size.height);
      
      for draw in draw_calls {
        match draw.get_type() {
          DrawType::DrawInstanced(ref references) => {
            let (buffer_ref, texture_ref) = references;
            cmd = self.texture_shader.draw_instanced(Arc::clone(&device), cmd, i, buffer_ref.to_string(), texture_ref.to_string());
          },
          DrawType::AddInstancedSpriteSheet(ref info) => {
            let (buffer_reference, position, scale, rotation, colour, sprite_details) = info.clone(); 
            self.texture_shader.add_instanced_draw
(position, scale, rotation, Some(sprite_details), colour, true, buffer_reference.to_string());
          },
          DrawType::DrawFont(ref info) => {
            let (font, display_text, position, scale, colour, outline_colour, edge_width, _wrapped, wrap_length, centered) = info.clone(); 
            
            let texture_resource = self.resources.get_font(font.clone());
            if let Some((font_details, _texture)) = texture_resource {
              cmd = self.texture_shader.draw_text(Arc::clone(&device), cmd, display_text, font, position, scale, colour, outline_colour, edge_width, wrap_length, centered, font_details, window_size.width as f32, window_size.height as f32);
            }
          },
          DrawType::DrawTextured(ref info) => {
            let (reference, position, scale, rotation, alpha) = info.clone(); 
            
            let texture_resource = self.resources.get_texture(reference.clone());
            if let Some(_texture) = texture_resource {
              cmd = self.texture_shader.draw_texture(Arc::clone(&device), cmd, position, scale, rotation, None, Some(Vector4::new(1.0, 1.0, 1.0, alpha)), true, reference.to_string());
            }
          },
          DrawType::DrawSpriteSheet(ref info) => {
            let (reference, position, scale, rotation, sprite_details, colour) = info.clone(); 
            
            let texture_resource = self.resources.get_texture(reference.clone());
            if let Some(_texture) = texture_resource {
              cmd = self.texture_shader.draw_texture
(Arc::clone(&device), cmd, position, scale, rotation, Some(sprite_details), Some(Vector4::new(colour.x, colour.y, colour.z, colour.w)), true, reference.to_string());
            }
          },
          DrawType::DrawColoured(ref info) => {
            let (position, scale, colour, rotation) = info.clone(); 
            
            cmd = self.texture_shader.draw_texture(Arc::clone(&device), cmd, position, scale, rotation, None, Some(colour), false, "".to_string());
          },
          DrawType::LoadTexture(ref info) => {
            let reference = info.clone();
            self.resources.load_texture_from_reference(reference);
          },
          DrawType::SetTextureScale(ref scale) => {
            self.texture_shader.set_scale(scale.clone());
          },
          DrawType::ScissorRender(ref dim) => {
            cmd = cmd.set_scissor(Arc::clone(&device), dim.x as i32, (window_size.height as f32-dim.y) as i32 , dim.z as u32, dim.w as u32);
          },
          DrawType::ResetScissorRender => {
            cmd = cmd.set_scissor(Arc::clone(&device), 0, 0, window_size.width, window_size.height);
          },
          DrawType::OrthoCamera(ref info) => {
            let (position, size, vel) = info.clone();
            
            if let Some(goal_pos) = position {
              self.texture_shader.lerp_camera(goal_pos, vel);
            } else if let Some(goal_size) = size {
              self.texture_shader.lerp_camera_to_size(goal_size, vel);
            } else {
              self.texture_shader.reset_camera(window_size.width as f32, window_size.height as f32);
            }
          },
          draw => {
            model_draw_calls.push(draw);
          }
        }
      }
      }
      {
        let device = self.window.device();
        let instance = self.window.instance();
        let dpi = self.get_dpi_scale() as f32;
        if let Some(imgui) = &mut self.imgui {
          let frame_size = self.window.imgui_window(imgui);
          
          let mut ui = imgui.frame(frame_size, delta_time);
          ui.show_default_style_editor();
          ui.show_demo_window(&mut true);
          /*ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();
                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos.0,
                    mouse_pos.1
                ));
          });*/
          cmd = self.texture_shader.draw_imgui(Arc::clone(&instance), Arc::clone(&device), cmd, i, self.max_frames, ui, dpi);
        }
      }
      
      let device = self.window.device();
      let instance = self.window.instance();
      let graphics_queue = self.window.get_graphics_queue();
      let window_size = vk::Extent2D { width: self.window_dimensions.width, height: self.window_dimensions.height };
      
      cmd = cmd.end_render_pass(Arc::clone(&device));
      
      // Model Shader
      cmd = self.model_shader.begin_renderpass(Arc::clone(&device), cmd, &self.model_clear_colour, &window_size, i);
      
      cmd = cmd.set_viewport(Arc::clone(&device), 0.0, 0.0, window_size.width as f32, window_size.height as f32);
      cmd = cmd.set_scissor(Arc::clone(&device), 0, 0, window_size.width, window_size.height);
      
      for draw in model_draw_calls {
        match draw {
          DrawType::DrawModel(ref info) => {
            let (reference, position, scale, rotation, hologram) = info;
            cmd = self.model_shader.draw_model(Arc::clone(&device), cmd, *position, *scale, *rotation, reference.to_string(), *hologram, window_size.width as f32, window_size.height as f32, delta_time);
          },
          DrawType::AddInstancedModel(ref info) => {
            let (reference, position, scale, rotation, colour, hologram) = info;
            self.model_shader.add_instanced_model(*position, *scale, *rotation, *colour, reference.to_string(), *hologram);
          },
          DrawType::DrawInstancedModel(ref reference) => {
            cmd = self.model_shader.draw_instanced(Arc::clone(&device), cmd, reference.to_string(), window_size.width as f32, window_size.height as f32, delta_time);
          },
          DrawType::ModelCamera(ref info) => {
            let (new_camera, move_direction, mouse_offset, set_move_speed, set_mouse_sensitivity) = info;
            
            if let Some(camera) = new_camera {
              self.model_shader.set_camera(camera.clone());
            }
            
            if let Some((direction, delta_time)) = move_direction {
              self.model_shader.move_camera(direction.clone(), *delta_time);
            }
            
            if let Some(offset) = mouse_offset {
              self.model_shader.process_mouse_movement(offset.x, offset.y);
            }
            
            if let Some(move_speed) = set_move_speed {
              self.model_shader.set_camera_move_speed(*move_speed);
            }
            
            if let Some(mouse_sensitivity) = set_mouse_sensitivity {
              self.model_shader.set_mouse_sensitivity(*mouse_sensitivity);
            }
          },
          DrawType::LoadModel(ref info) => {
            let reference = info.clone();
            self.resources.load_model_from_reference(reference);
          },
          _ => {}
        }
      }
      
      cmd = cmd.end_render_pass(Arc::clone(&device));
      
      // Final Shader
      cmd = self.final_shader.begin_renderpass(Arc::clone(&device), cmd, &self.final_clear_colour, &window_size, i);
      
      cmd = cmd.set_viewport(Arc::clone(&device), 0.0, 0.0, window_size.width as f32, window_size.height as f32);
      cmd = cmd.set_scissor(Arc::clone(&device), 0, 0, window_size.width, window_size.height);
      
      let texture_image = self.texture_shader.get_texture(self.current_frame);
      let model_image = self.model_shader.get_texture(self.current_frame);
      cmd = self.final_shader.draw_to_screen(Arc::clone(&device), cmd, texture_image, model_image, &self.sampler, window_size.width as f32, window_size.height as f32, self.current_frame);
      
      cmd = cmd.end_render_pass(Arc::clone(&device));
      cmd.end_command_buffer(Arc::clone(&device));
      let swapchain = self.window.get_swapchain();
      match self.command_buffers[self.current_frame].submit(Arc::clone(&device), swapchain, image_index as u32, &self.semaphore_image_available[self.current_frame], &self.semaphore_render_finished[self.current_frame], &self.fences[self.current_frame], &graphics_queue) {
      vk::ERROR_OUT_OF_DATE_KHR => {
        self.recreate_swapchain = true;
      },
      e => { check_errors(e); },
    }
    
    // render to texture
  //  if self.image_from_draw.is_none() {
  //  self.fences[self.current_frame].wait(Arc::clone(&device));
  //    let texture_image = self.model_shader.get_texture(self.current_frame);
  //    let image = ImageAttachment::create_texture_from_command_buffer(Arc::clone(&instance), Arc::clone(&device), window_size.width, window_size.height, texture_image, &ImageTiling::Optimal, &ImageViewType::Type2D, self.window.swapchain_format(), &self.command_pool, self.window.get_graphics_queue());
     // self.resources.unload_texture_from_reference(Arc::clone(&device), "TempModel".to_string());
  //    self.image_from_draw = Some(image.clone());
  //    self.resources.insert_texture("TempModel".to_string(), image);
 //   }
    
    self.current_frame = (self.current_frame+1)%self.max_frames;
  }
  
  fn post_draw(&self) {
    
  }
  
  fn get_physical_dimensions(&self) -> Vector2<f32> {
    Vector2::new(self.window_dimensions.width as f32, self.window_dimensions.height as f32)
  }
  
  fn get_virtual_dimensions(&self) -> Vector2<f32> {
    Vector2::new(self.window_dimensions.width as f32 * self.dpi, self.window_dimensions.height as f32 * self.dpi)
  }
  
  fn get_events(&mut self) -> Vec<winit::Event> {
    let mut events = Vec::new();
    
    let mut recreate = false;
    let mut mouse_pos = self.mouse_position;
    let mut new_dpi = self.dpi;
    let dimensions = self.get_virtual_dimensions();
    
    let all_events = self.window.get_events();
    
    all_events.poll_events(|ev| {
      match &ev {
        winit::Event::WindowEvent{ event, .. } => {
          match event {
            winit::WindowEvent::Resized(_new_size) => {
              recreate = true;
            },
            winit::WindowEvent::CursorMoved{device_id: _, position, modifiers: _} => {
              mouse_pos = Vector2::new(position.x as f32, dimensions.y / new_dpi - position.y as f32);
            },
            winit::WindowEvent::HiDpiFactorChanged(event_dpi) => {
              new_dpi = *event_dpi as f32;
            },
            _ => {}
          }
        },
        _ => {}
      }
      
      events.push(ev);
    });
    
    self.mouse_position = mouse_pos;
    self.dpi = new_dpi;
    if recreate {
      self.recreate_swapchain = true;
    }
    
    for ev in &events {
      if let Some(imgui) = &mut self.imgui {
        imgui_winit_support::handle_event(imgui, ev, new_dpi as f64, new_dpi as f64);
      }
    }
    
    events
  }
  
  fn get_mouse_position(&mut self) -> Vector2<f32> {
    self.mouse_position
  }
  
  fn get_fonts(&self) -> HashMap<String, GenericFont> {
    HashMap::new()
  }
  
  fn get_dpi_scale(&self) -> f32 {
    self.dpi
  }
  
  fn is_ready(&self) -> bool {
    self.resources.pending_objects_loaded()
  }
  
  fn set_cursor_position(&mut self, _x: f32, _y: f32) {
    
  }
  
  fn show_cursor(&mut self) {
    
  }
  
  fn hide_cursor(&mut self) {
    
  }
  
  fn set_clear_colour(&mut self, r: f32, g: f32, b: f32, a: f32) {
    println!("SETTING CLEAR COLOUR");
    println!("Clear Colour: {}, {}, {}, {}", r, g, b, a);
    let model_msaa = self.settings.get_model_msaa();
    println!("model msaa: {}", model_msaa);
    
    self.final_clear_colour = vec!(
      vk::ClearValue {
        color: vk::ClearColorValue { float32: [r, g, b, a] }
      },
    );
  }
  
  fn set_camera(&mut self, _camera: Camera) {
    
  }
  
  fn get_camera(&self) -> Camera {
    Camera::default_vk()
  }
  
  fn num_drawcalls(&self) -> u32 {
    0
  }
}


impl Drop for CoreMaat {
  fn drop(&mut self) {
    self.settings.save();
    
    self.window.device().wait();
    
    println!("Destroying Fences");
    for fence in &self.fences {
      let device = self.window.device();
      fence.wait(Arc::clone(&device));
      fence.destroy(Arc::clone(&device));
    }
    
    let device = self.window.device();
    
    self.resources.destroy(Arc::clone(&device));
    self.dummy_image.destroy(Arc::clone(&device));
    self.sampler.destroy(Arc::clone(&device));
    
    //self.compute_shader.destroy(Arc::clone(&device));
    self.texture_shader.destroy(Arc::clone(&device));
    self.model_shader.destroy(Arc::clone(&device));
    self.final_shader.destroy(Arc::clone(&device));
    
    self.descriptor_set_pool.destroy(Arc::clone(&device));
    
    self.command_pool.destroy(Arc::clone(&device));
    for semaphore in &self.semaphore_image_available {
      semaphore.destroy(Arc::clone(&device));
    }
    for semaphore in &self.semaphore_render_finished {
      semaphore.destroy(Arc::clone(&device));
    }
  }
}
