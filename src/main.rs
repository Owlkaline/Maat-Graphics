extern crate ash;
extern crate winit;
extern crate image;

use ash::util::*;
use ash::vk;
use std::default::Default;
use std::ffi::CString;
use std::io::Cursor;
use std::mem;
use std::mem::align_of;

use winit::{
  dpi::{LogicalSize, PhysicalSize},
  event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder
};

use std::time::Instant;

use crate::ash::version::DeviceV1_0;

mod modules;

use crate::modules::{Vulkan, VkWindow, Buffer, Shader, GraphicsPipelineBuilder, Image, ImageBuilder, Sampler,
                     DescriptorSet, DescriptorWriter, ComputeShader, DescriptorPoolBuilder};
use crate::modules::vulkan::find_memorytype_index;

const APP_NAME: &str = "Ash - Example";
const WINDOW_SIZE: [u32; 2] = [1280, 720];

#[derive(Clone, Debug, Copy)]
struct ComboVertex {
  pos: [f32; 4],
  colour: [f32; 4],
  uv: [f32; 2],
}

#[derive(Clone, Debug, Copy)]
struct UniformBuffer {
  colour: [f32; 4],
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

fn main() {
  let mut screen_resolution = vk::Extent2D { width: 1, height: 1};
  
  let mut event_loop = EventLoop::new();
  let mut window = VkWindow::new(APP_NAME, WINDOW_SIZE[0], WINDOW_SIZE[1], &event_loop, &mut screen_resolution);
  
  let mut vulkan = Vulkan::new(&mut window, screen_resolution);
  
  let uniform_data = vec![
    UniformBuffer {
      colour: [0.0, 0.0, 0.0, 0.0],
    }
  ];
  
  let uniform_buffer = Buffer::<UniformBuffer>::new_uniform_buffer(vulkan.device(), &uniform_data);
  
  let dst_image = create_texture(&mut vulkan, "./textures/negativeviewportheight.jpg");
  let dst_image2 = create_texture(&mut vulkan, "./textures/rust.png");
  
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
  let descriptor_set_writer = DescriptorWriter::builder().update_uniform_buffer(&uniform_buffer, &descriptor_sets)
                                                         .update_image(&dst_image, &sampler, &descriptor_sets);
  
  let descriptor_sets2 = DescriptorSet::builder()
                                       .uniform_buffer_fragment()
                                       .combined_image_sampler_fragment()
                                       .build(vulkan.device(), &descriptor_pool);
  let descriptor_set_writer2 = DescriptorWriter::builder().update_uniform_buffer(&uniform_buffer, &descriptor_sets2)
                                                         .update_image(&dst_image2, &sampler, &descriptor_sets2);
  
  let (combo_shader, combo_index_buffer, combo_vertex_buffer) = create_combo_shader(&vulkan, &descriptor_sets);
  
  descriptor_set_writer.build(vulkan.device());
  descriptor_set_writer2.build(vulkan.device());
  
  let compute_descriptor_sets = DescriptorSet::builder().storage_compute().build(vulkan.device(), &descriptor_pool);
  let compute_shader = ComputeShader::new(vulkan.device(), 
                                          Cursor::new(&include_bytes!("../shaders/collatz_comp.spv")[..]),
                                          &compute_descriptor_sets);
  
  let mut compute_data = vec![64, 32, 8, 12, 96];
  
  vulkan.run_compute(&compute_shader, &compute_descriptor_sets, &mut compute_data);
  println!("Compute Data: {:?}", compute_data);
  
  event_loop.run(move |event, _, control_flow| {
      //*control_flow = ControlFlow::Wait;
      match event {
          Event::WindowEvent { event, .. } => match event {
              WindowEvent::CloseRequested => {
                  *control_flow = ControlFlow::Exit;
              },
              WindowEvent::KeyboardInput {
                  input:
                  KeyboardInput {
                      virtual_keycode: Some(VirtualKeyCode::Escape),
                      ..
                  },
                  ..
              } => {
                *control_flow = ControlFlow::Exit
              },
              WindowEvent::Resized(dimensions) => {
                println!("resized");
                vulkan.swapchain().set_screen_resolution(
                  dimensions.width,
                  dimensions.height
                );
                
                vulkan.recreate_swapchain();
                //*control_flow = ControlFlow::Wait;
              },
              _ => (),
          },
          Event::MainEventsCleared => {
            if let Some(present_index) = vulkan.start_render() {
              vulkan.draw(&descriptor_sets,
                          &combo_shader,
                          &combo_vertex_buffer,
                          &combo_index_buffer,
                          vec!(0.0, 0.0, 1.0, 0.0,  // x y usetexture empty
                               0.0, 0.0, 1.0, 1.0)); // Colour
              vulkan.draw(&descriptor_sets2,
                          &combo_shader,
                          &combo_vertex_buffer,
                          &combo_index_buffer,
                          vec!(0.2, 0.2, 1.0, 0.0,
                               1.0, 0.0, 0.0, 1.0));
              vulkan.end_render(present_index);
            }
          },
          Event::LoopDestroyed => {
            unsafe {
              vulkan.device().internal().device_wait_idle().unwrap();
              descriptor_sets.destroy(vulkan.device());
              compute_descriptor_sets.destroy(vulkan.device());
              compute_shader.destroy(vulkan.device());
              combo_shader.destroy(vulkan.device());
              combo_index_buffer.destroy(vulkan.device());
              combo_vertex_buffer.destroy(vulkan.device());
            }
          }
          _ => (),
      }
  });
}
