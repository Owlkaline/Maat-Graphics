extern crate maat_graphics;

use maat_graphics::{winit};

use winit::{
  event::{Event, KeyboardInput, VirtualKeyCode, MouseButton, ElementState, WindowEvent},
  event_loop::{ControlFlow, EventLoop}
};

use std::time::Instant;

use maat_graphics::{MaatGraphics, VkWindow};

use maat_graphics::ash::vk;

const APP_NAME: &str = "MaatGraphics - Example";
//const WINDOW_SIZE: [u32; 2] = [1280, 720];

const DELTA_STEP: f32 = 0.001;

fn main() {
  let create_window_size: [u32; 2] = [1280, 720];
  let mut screen_resolution = vk::Extent2D { width: 1, height: 1};
  
  let event_loop = EventLoop::new();
  let mut window = VkWindow::new(APP_NAME, create_window_size[0], create_window_size[1], &event_loop, &mut screen_resolution);
  
  let mut vulkan = MaatGraphics::new(&mut window, screen_resolution);
  
  vulkan.load_texture("orientation", "./textures/negativeviewportheight.jpg");
  vulkan.load_texture("rust_crab", "./textures/rust.png");
  
  let mut _delta_time = 0.0;
  let mut last_time = Instant::now();
  
  let mut total_delta_time = 0.0;
  
  event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;
      
      _delta_time = last_time.elapsed().subsec_nanos() as f32 / 1000000000.0 as f32;
      last_time = Instant::now();
      total_delta_time += _delta_time as f32;
      
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
                //println!("resized");
                vulkan.recreate_swapchain(dimensions.width, dimensions.height);
                screen_resolution.width = dimensions.width;
                screen_resolution.height = dimensions.height;
              },
              event => {
                handle_event(event, screen_resolution);
              },
          },
          Event::MainEventsCleared => {
            vulkan.draw(vec!(
              (vec!(0.0, 0.0, 720.0, 720.0,  // x y scale_x scale_y
                    0.0, 0.0, 1.0, 1.0, // r g b a
                    1.0, 45.0), // use texture, rotation
               "orientation"),
              (vec!(150.0, 150.0, 573.0, 300.0,
                    1.0, 0.0, 1.0, 1.0,
                    1.0, 0.0), 
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

fn handle_event(event: WindowEvent, screen_resolution: vk::Extent2D) {
  match event {
    WindowEvent::KeyboardInput {device_id: _, input: key, is_synthetic: _} => {
      if let Some(key_code) = key.virtual_keycode {
        println!("KeyInput: {:?}", key_code);
      }
    },
    WindowEvent::CursorMoved { device_id: _, position: pos, ..} => {
      let _x: f64 = pos.x;
      let _y: f64 = screen_resolution.height as f64 - pos.y;
      
    },
    WindowEvent::MouseInput {device_id: _, state, button, ..} => {
      match state {
        ElementState::Pressed => {
          
        },
        ElementState::Released => {
          
        },
      }
      
      match button {
        MouseButton::Left => {
          
        },
        MouseButton::Right => {
          
        },
        MouseButton::Middle => {
          
        },
        MouseButton::Other(_id) => {
          
        },
      }
    },
    _ => {},
  }
}
