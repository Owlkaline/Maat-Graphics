#![ allow( dead_code )]

pub extern crate ash;
pub extern crate winit;
pub extern crate image;

mod modules;
mod shader_handlers;

pub use crate::modules::{VkWindow};

use ash::vk;
use std::io::Cursor;

use crate::ash::version::DeviceV1_0;

use crate::modules::{Vulkan, Image, DescriptorSet, ComputeShader, DescriptorPoolBuilder};
use crate::shader_handlers::{TextureHandler, ModelHandler};
pub use crate::shader_handlers::Camera;

pub struct MaatGraphics {
  vulkan: Vulkan,
  texture_handler: TextureHandler,
  model_handler: ModelHandler,
  compute_descriptor_pool: vk::DescriptorPool,
  compute_shader: ComputeShader,
  compute_descriptor_sets: DescriptorSet,
}

impl MaatGraphics {
  pub fn new(window: &mut VkWindow, screen_resolution: vk::Extent2D) -> MaatGraphics {
    let mut vulkan = Vulkan::new(window, screen_resolution);
    
    let compute_descriptor_pool = DescriptorPoolBuilder::new()
                                              .num_storage(5)
                                              .build(vulkan.device());
    let compute_descriptor_sets = DescriptorSet::builder().storage_compute().build(vulkan.device(), &compute_descriptor_pool);
    let compute_shader = ComputeShader::new(vulkan.device(), 
                                            Cursor::new(&include_bytes!("../shaders/collatz_comp.spv")[..]),
                                            &compute_descriptor_sets);
    
    let mut compute_data = vec![64, 32, 8, 12, 96];
    vulkan.run_compute(&compute_shader, &compute_descriptor_sets, &mut compute_data);
    println!("Compute Data: {:?}", compute_data);
    
    let texture_handler = TextureHandler::new(&mut vulkan, screen_resolution);
    let model_handler = ModelHandler::new(&mut vulkan, screen_resolution);
    
    MaatGraphics {
      vulkan,
      texture_handler,
      model_handler,
      compute_descriptor_pool,
      compute_shader,
      compute_descriptor_sets,
    }
  }
  
  pub fn load_text(&mut self, text_ref: &str, text: &str, size: f32) {
    self.texture_handler.load_text(&mut self.vulkan, text_ref, text, size);
  }
  
  pub fn load_texture(&mut self, texture_ref: &str, texture: &str) {
    self.texture_handler.load_texture(&mut self.vulkan, texture_ref, texture);
  }
  
  pub fn load_model(&mut self, model_ref: &str, model: &str) {
    self.model_handler.load_model(&mut self.vulkan, model_ref, model);
  }
  
  pub fn recreate_swapchain(&mut self, width: u32, height: u32) {
    self.vulkan.swapchain().set_screen_resolution(
      width,
      height,
    );
    
    self.vulkan.recreate_swapchain();
  }
  
  pub fn mut_camera(&mut self) -> &mut Camera {
    self.model_handler.mut_camera()
  }
  
  pub fn draw_texture(&mut self, draw_data: Vec<(Vec<f32>, &str, Option<&str>)>) {
    if let Some(present_index) = self.vulkan.start_texture_render(self.texture_handler.shader(),
                                                                  self.texture_handler.uniform_descriptor()) {
      for (data, texture, some_text) in draw_data {
        if let Some(text) = some_text {
          self.texture_handler.draw_text(&mut self.vulkan, data, text, texture);
        } else {
          self.texture_handler.draw(&mut self.vulkan, data, texture);
        }
      }
      self.vulkan.end_render(present_index);
    }
  }
  
  pub fn draw_model(&mut self, draw_data: Vec<(Vec<f32>, &str)>) {
    if let Some(present_index) = self.vulkan.start_model_render() {
      for (data, model) in draw_data {
        self.model_handler.draw(&mut self.vulkan, data, model);
      }
      self.vulkan.end_render(present_index);
    }
    
    if self.model_handler.mut_camera().is_updated() {
      self.model_handler.update_uniform_buffer(self.vulkan.device());
    }
  }
  
  pub fn update_animations(&mut self, delta_time: f32) {
    self.model_handler.update_animations(&mut self.vulkan, delta_time);
  }
  
  pub fn destroy(&mut self) {
    unsafe {
      self.vulkan.device().internal().device_wait_idle().unwrap();
    }
    
    self.texture_handler.destroy(&mut self.vulkan);
    
    self.compute_descriptor_sets.destroy(self.vulkan.device());
    self.compute_shader.destroy(self.vulkan.device());
    
    unsafe {
      self.vulkan.device().destroy_descriptor_pool(self.compute_descriptor_pool, None);
    }
  }
}
