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
pub use crate::shader_handlers::{Camera, font::FontChar, Math};

pub struct MaatGraphics {
  vulkan: Vulkan,
  texture_handler: TextureHandler,
  model_handler: ModelHandler,
  compute_descriptor_pool: vk::DescriptorPool,
  compute_shader: ComputeShader,
  compute_descriptor_sets: DescriptorSet,
}

impl MaatGraphics {
  pub fn new(window: &mut VkWindow, screen_resolution: [u32; 2]) -> MaatGraphics {
    let screen_resolution = vk::Extent2D { width: screen_resolution[0], height: screen_resolution[1] };
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
  /*
  pub fn load_text(&mut self, text_ref: &str, text: &str, size: f32) {
    self.texture_handler.load_text(&mut self.vulkan, text_ref, text, size);
  }*/
  
  pub fn load_texture<T: Into<String>>(&mut self, texture_ref: T, texture: T) {
    self.texture_handler.load_texture(&mut self.vulkan, texture_ref, texture);
  }
  
  pub fn load_model<T: Into<String>>(&mut self, model_ref: T, model: T) {
    self.model_handler.load_model(&mut self.vulkan, model_ref, model);
  }
  
  pub fn model_bounding_box<T: Into<String>>(&self, model_ref: T) -> ([f32; 3], [f32; 3]) {
    self.model_handler.model_bounding_box(model_ref)
  }
  
  pub fn all_model_bounding_boxes(&self) -> Vec<(String, ([f32; 3], [f32; 3]))> {
    self.model_handler.all_model_bounding_boxes()
  }
  
  pub fn get_font_data(&self) -> (Vec<FontChar>, u32, u32) {
    self.texture_handler.get_font_data()
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
  
  pub fn draw<T: Into<String>, L: Into<String>, S: Into<String>>(&mut self,
              texture_data: Vec<(Vec<f32>, T, Option<L>)>,
              model_data: Vec<(Vec<f32>, S)>
             ) {
    
    if self.model_handler.mut_camera().is_updated() {
      self.model_handler.update_uniform_buffer(self.vulkan.device());
    }
    
    if let Some(present_index) = self.vulkan.start_render() {
      self.vulkan.begin_renderpass_model(present_index);
      for (data, model) in model_data {
        self.model_handler.draw(&mut self.vulkan, data, &model.into());
      }
      self.vulkan.end_renderpass();
      self.vulkan.begin_renderpass_texture(present_index);
      for (data, texture, some_text) in texture_data {
        if let Some(text) = some_text {
          self.texture_handler.draw_text(&mut self.vulkan, data, &text.into(), &texture.into());
        } else {
          self.texture_handler.draw(&mut self.vulkan, data, &texture.into());
        }
      }
      self.vulkan.end_renderpass();
      self.vulkan.end_render(present_index);
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
