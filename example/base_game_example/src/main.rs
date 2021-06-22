extern crate maat_graphics;

use maat_graphics::{winit, ash, image};

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

use maat_graphics::{MaatGraphics, VkWindow};

use maat_graphics::ash::util::*;
use maat_graphics::ash::vk;
use maat_graphics::ash::version::DeviceV1_0;

const APP_NAME: &str = "MaatGraphics - Example";
const WINDOW_SIZE: [u32; 2] = [1280, 720];

const DELTA_STEP: f32 = 0.001;

fn main() {
  let mut screen_resolution = vk::Extent2D { width: 1, height: 1};
  
  let mut event_loop = EventLoop::new();
  let mut window = VkWindow::new(APP_NAME, WINDOW_SIZE[0], WINDOW_SIZE[1], &event_loop, &mut screen_resolution);
  
  let mut vulkan = MaatGraphics::new(&mut window, screen_resolution);
  
  vulkan.load_texture("orientation", "./textures/negativeviewportheight.jpg");
  vulkan.load_texture("rust_crab", "./textures/rust.png");
  
  let mut delta_time = 0.0;
  let mut last_time = time::Instant::now();
  
  let mut total_delta_time = 0.0;
  
  event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;
      
      delta_time = last_time.elapsed().subsec_nanos() as f32 / 1000000000.0 as f32;
      last_time = time::Instant::now();
      total_delta_time += delta_time as f32;
      
      if total_delta_time > DELTA_STEP {
        let delta_steps = (total_delta_time / DELTA_STEP).floor() as usize;
        
        for _ in 0..delta_steps {
          //F(DELTA_STEP); // update
          total_delta_time -= DELTA_STEP;
        }
      }
      
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
                vulkan.recreate_swapchain(dimensions.width, dimensions.height);
              },
              _ => (),
          },
          Event::MainEventsCleared => {
            vulkan.draw(vec!(
              (vec!(0.0, 0.0, 720.0, 720.0,  // x y scale_x scale_y
                    0.0, 0.0, 1.0, 1.0, // r g b a
                    1.0), // use texture
               "orientation"),
              (vec!(150.0, 150.0, 573.0, 300.0,
                    1.0, 0.0, 1.0, 1.0,
                    1.0), 
               "rust_crab"),
            ));
          },
          Event::LoopDestroyed => {
            vulkan.destroy();
          }
          _ => (),
      }
  });
}
