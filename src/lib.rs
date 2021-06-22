pub extern crate ash;
pub extern crate winit;
pub extern crate image;

mod modules;
mod shader_handlers;

pub use crate::modules::{VkWindow};

use ash::util::*;
use ash::vk;
use std::default::Default;
use std::ffi::CString;
use std::io::Cursor;
use std::mem;
use std::mem::align_of;

use std::time;

use winit::{
  dpi::{LogicalSize, PhysicalSize},
  event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder
};

use std::time::Instant;

use std::collections::HashMap;

use crate::ash::version::DeviceV1_0;

use crate::modules::{Vulkan, Buffer, Shader, GraphicsPipelineBuilder, Image, ImageBuilder, Sampler,
                     DescriptorSet, DescriptorWriter, ComputeShader, DescriptorPoolBuilder};
use crate::modules::vulkan::find_memorytype_index;
use crate::shader_handlers::TextureHandler;

pub struct MaatGraphics {
  vulkan: Vulkan,
  texture_handler: TextureHandler,
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
    
    let texture_handler = TextureHandler::new(&mut vulkan);
    
    MaatGraphics {
      vulkan,
      texture_handler,
      compute_descriptor_pool,
      compute_shader,
      compute_descriptor_sets,
    }
  }
  
  pub fn load_texture(&mut self, texture_ref: &str, texture: &str) {
    self.texture_handler.load_texture(&mut self.vulkan, texture_ref, texture);
  }
  
  pub fn recreate_swapchain(&mut self, width: u32, height: u32) {
    self.vulkan.swapchain().set_screen_resolution(
      width,
      height,
    );
    
    self.vulkan.recreate_swapchain();
  }
  
  pub fn draw(&mut self, draw_data: Vec<(Vec<f32>, &str)>) {
    if let Some(present_index) = self.vulkan.start_render() {
      for (data, texture) in draw_data {
        self.texture_handler.draw(&mut self.vulkan, data, texture);
      }
      self.vulkan.end_render(present_index);
    }
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
