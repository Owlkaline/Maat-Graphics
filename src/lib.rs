pub extern crate ash;
pub extern crate winit;
pub extern crate image;

pub mod modules;

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

use crate::ash::version::DeviceV1_0;

use crate::modules::{Vulkan, VkWindow, Buffer, Shader, GraphicsPipelineBuilder, Image, ImageBuilder, Sampler,
                     DescriptorSet, DescriptorWriter, ComputeShader, DescriptorPoolBuilder};
use crate::modules::vulkan::find_memorytype_index;

#[derive(Clone, Debug, Copy)]
pub struct ComboVertex {
  pos: [f32; 4],
  colour: [f32; 4],
  uv: [f32; 2],
}

#[derive(Clone, Debug, Copy)]
struct UniformBuffer {
  colour: [f32; 4],
}

pub struct MaatGraphics {
  vulkan: Vulkan,
  uniform_buffer: Buffer<UniformBuffer>,
  descriptor_pool: vk::DescriptorPool,
  sampler: Sampler,
  combo_shader: Shader<ComboVertex>,
  combo_index_buffer: Buffer<u32>,
  combo_vertex_buffer: Buffer<ComboVertex>,
  textures: Vec<Image>,
  texture_descriptors: Vec<DescriptorSet>,
  compute_shader: ComputeShader,
  compute_descriptor_sets: DescriptorSet,
}

impl MaatGraphics {
  pub fn new(window: &mut VkWindow, screen_resolution: vk::Extent2D) -> MaatGraphics {
    let mut vulkan = Vulkan::new(window, screen_resolution);
    
    let uniform_data = vec![
      UniformBuffer {
        colour: [0.0, 0.0, 0.0, 0.0],
      }
    ];
    
    let uniform_buffer = Buffer::<UniformBuffer>::new_uniform_buffer(vulkan.device(), &uniform_data);
    
    let descriptor_pool = DescriptorPoolBuilder::new()
                                              .num_combined_image_samplers(5)
                                              .num_storage(5)
                                              .num_uniform_buffers(5)
                                              .build(vulkan.device());
    
    let sampler = Sampler::builder()
                           .min_filter_linear()
                           .mag_filter_linear()
                           .address_mode_mirrored_repeat()
                           .mipmap_mode_linear()
                           .border_colour_float_opaque_white()
                           .compare_op_never()
                           .build(vulkan.device());
    
    let descriptor_sets = DescriptorSet::builder()
                                      .uniform_buffer_fragment()
                                      .combined_image_sampler_fragment()
                                      .build(vulkan.device(), &descriptor_pool);
    
    let (combo_shader, combo_index_buffer, combo_vertex_buffer) = create_combo_shader(&vulkan, &descriptor_sets);
    
    let compute_descriptor_sets = DescriptorSet::builder().storage_compute().build(vulkan.device(), &descriptor_pool);
    let compute_shader = ComputeShader::new(vulkan.device(), 
                                            Cursor::new(&include_bytes!("../shaders/collatz_comp.spv")[..]),
                                            &compute_descriptor_sets);
    
    let mut compute_data = vec![64, 32, 8, 12, 96];
    vulkan.run_compute(&compute_shader, &compute_descriptor_sets, &mut compute_data);
    println!("Compute Data: {:?}", compute_data);
    
    MaatGraphics {
      vulkan,
      uniform_buffer,
      descriptor_pool,
      sampler,
      combo_shader,
      combo_index_buffer,
      combo_vertex_buffer,
      textures: Vec::new(),
      texture_descriptors: Vec::new(),
      compute_shader,
      compute_descriptor_sets,
    }
  }
  
  pub fn load_texture(&mut self, texture: &str) {
    let dst_image = create_texture(&mut self.vulkan, texture);
    
    let descriptor_sets = DescriptorSet::builder()
                                      .uniform_buffer_fragment()
                                      .combined_image_sampler_fragment()
                                      .build(self.vulkan.device(), &self.descriptor_pool);
    let descriptor_set_writer = DescriptorWriter::builder().update_uniform_buffer(&self.uniform_buffer, &descriptor_sets)
                                                           .update_image(&dst_image, &self.sampler, &descriptor_sets);
    
    descriptor_set_writer.build(self.vulkan.device());
    
    self.textures.push(dst_image);
    self.texture_descriptors.push(descriptor_sets);
  }
  
  pub fn recreate_swapchain(&mut self, width: u32, height: u32) {
    self.vulkan.swapchain().set_screen_resolution(
      width,
      height,
    );
    
    self.vulkan.recreate_swapchain();
  }
  
  pub fn draw(&mut self, draw_data: Vec<(Vec<f32>, usize)>) {
    if let Some(present_index) = self.vulkan.start_render() {
      for (data, texture) in draw_data {
        self.vulkan.draw(&self.texture_descriptors[texture],
                         &self.combo_shader,
                         &self.combo_vertex_buffer,
                         &self.combo_index_buffer,
                         data);
      }
      self.vulkan.end_render(present_index);
    }
  }
  
  pub fn destroy(&mut self) {
    unsafe {
      self.vulkan.device().internal().device_wait_idle().unwrap();
      
      for i in 0..self.texture_descriptors.len() {
        self.texture_descriptors[i].destroy(self.vulkan.device());
      }
      
      self.compute_descriptor_sets.destroy(self.vulkan.device());
      self.compute_shader.destroy(self.vulkan.device());
      self.combo_shader.destroy(self.vulkan.device());
      self.combo_index_buffer.destroy(self.vulkan.device());
      self.combo_vertex_buffer.destroy(self.vulkan.device());
    }
  }
}

fn create_combo_shader(vulkan: &Vulkan, descriptor_sets: &DescriptorSet) -> (Shader<ComboVertex>, Buffer<u32>, Buffer<ComboVertex>) {
  let combo_index_buffer_data = vec![0, 1, 2, 3, 4, 5];//vec![3, 2, 0, 2, 0, 1];
  let combo_vertices = vec![
    ComboVertex {
        pos: [1.0, 1.0, 0.0, 1.0],
        colour: [0.0, 1.0, 0.0, 1.0],
        uv: [0.0, 0.0],
    },
    ComboVertex {
        pos: [-1.0, 1.0, 0.0, 1.0],
        colour: [0.0, 0.0, 1.0, 1.0],
        uv: [1.0, 0.0],
    },
    ComboVertex {
        pos: [-1.0, -1.0, 0.0, 1.0],
        colour: [1.0, 0.0, 0.0, 1.0],
        uv: [1.0, 1.0],
    },
    
    ComboVertex {
        pos: [-1.0, -1.0, 0.0, 1.0],
        colour: [0.0, 1.0, 0.0, 1.0],
        uv: [1.0, 1.0],
    },
    ComboVertex {
        pos: [1.0, -1.0, 0.0, 1.0],
        colour: [0.0, 0.0, 1.0, 1.0],
        uv: [0.0, 1.0],
    },
    ComboVertex {
        pos: [1.0, 1.0, 0.0, 1.0],
        colour: [1.0, 0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
    }
  ];
  
  let combo_vertex = ComboVertex {
    pos: [0.0, 0.0, 0.0, 0.0],
    colour: [0.0, 0.0, 0.0, 0.0],
    uv: [0.0, 0.0],
  };
  
  let combo_index_buffer = Buffer::<u32>::new_index(&vulkan.device(), combo_index_buffer_data);
  let combo_vertex_buffer = Buffer::<ComboVertex>::new_vertex(vulkan.device(), combo_vertices);
  
  let mut graphics_pipeline_builder = GraphicsPipelineBuilder::new().topology_triangle_list()
                                                                    .front_face_counter_clockwise()
                                                                    .polygon_mode_fill()
                                                                    .samples_1();
  
  let combo_shader = Shader::new(vulkan.device(),
                                    Cursor::new(&include_bytes!("../shaders/combo_vert.spv")[..]),
                                    Cursor::new(&include_bytes!("../shaders/combo_frag.spv")[..]),
                                    combo_vertex, 
                                    vec!(offset_of!(ComboVertex, pos) as u32, 
                                         offset_of!(ComboVertex, colour) as u32,
                                         offset_of!(ComboVertex, uv) as u32), 
                                    graphics_pipeline_builder,
                                    vulkan.renderpass(),
                                    vulkan.viewports(), 
                                    vulkan.scissors(),
                                    descriptor_sets.layouts());
  
  (combo_shader, combo_index_buffer, combo_vertex_buffer)
}

fn create_texture(vulkan: &mut Vulkan, texture: &str) -> Image {
  let image = image::open(&texture.clone()).expect(&("Failed to load texture: ".to_string() + &texture)).fliph().to_rgba();
  
  let dimensions = image.dimensions();
  let image_data = image.into_raw();
  
  let mut src_buffer = Buffer::<u8>::new_image(vulkan.device(), image_data);
  let mut dst_image = ImageBuilder::new(vk::Format::R8G8B8A8_UNORM, 1, 1)
                                   .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                                   .set_dimensions(dimensions.0, dimensions.1)
                                   .build_device_local(vulkan.device());
  
  vulkan.copy_buffer_to_device_local_image(&src_buffer, &dst_image);
  
  dst_image
}














