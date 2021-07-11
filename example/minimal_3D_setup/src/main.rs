extern crate maat_graphics;

use maat_graphics::winit::{
  event::{Event, KeyboardInput, VirtualKeyCode, MouseButton, ElementState, 
          MouseScrollDelta, WindowEvent, DeviceEvent},
  event_loop::{ControlFlow, EventLoop}
};

use std::time::Instant;

use maat_graphics::{MaatGraphics, VkWindow, Camera};

const APP_NAME: &str = "MaatGraphics - Minimal 3D Example";

const DELTA_STEP: f32 = 0.001;
const ANIMATION_DELTA_STEP: f32 = 0.01;

fn main() {
  let create_window_size: [u32; 2] = [1280, 720];
  let mut screen_resolution = [1, 1];
  
  let event_loop = EventLoop::new();
  let mut window = VkWindow::new(APP_NAME, create_window_size[0], create_window_size[1], &event_loop, &mut screen_resolution);
  
  let mut vulkan = MaatGraphics::new(&mut window, screen_resolution);
  
  vulkan.load_model("example_model", "./models/sample_model.glb"); // example importing model
  
  // Retrieve the actual size of the model once loaded.
  // Useful for collisions
  let _example_model_size = vulkan.model_bounding_box("example_model");
  
  let mut _delta_time = 0.0;
  let mut last_time = Instant::now();
  
  let mut total_delta_time = 0.0;
  let mut total_animation_delta_time = 0.0;
  
  vulkan.mut_camera().set_movement_speed(10.0);
  
  let mut device_keys = Vec::new();
  
  event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;
      
      _delta_time = last_time.elapsed().subsec_nanos() as f32 / 1000000000.0 as f32;
      last_time = Instant::now();
      total_delta_time += _delta_time as f32;
      total_animation_delta_time += _delta_time as f32;
      
      let model_data: Vec<(Vec<f32>, &str)>;
      
      if total_delta_time > DELTA_STEP {
        let delta_steps = (total_delta_time / DELTA_STEP).floor() as usize;
        
        let camera = vulkan.mut_camera();
        for _ in 0..delta_steps {
          if device_keys.contains(&VirtualKeyCode::W) {
            camera.forward(DELTA_STEP);
          }
          if device_keys.contains(&VirtualKeyCode::A) {
            camera.left(DELTA_STEP);
          }
          if device_keys.contains(&VirtualKeyCode::S) {
            camera.backward(DELTA_STEP);
          }
          if device_keys.contains(&VirtualKeyCode::D) {
            camera.right(DELTA_STEP);
          }
          
          //F(DELTA_STEP); // update
          total_delta_time -= DELTA_STEP;
        }
      }
      
      if total_animation_delta_time > ANIMATION_DELTA_STEP {
        let delta_steps = (total_animation_delta_time / ANIMATION_DELTA_STEP).floor() as usize;
        for _ in 0..delta_steps {
          vulkan.update_animations(ANIMATION_DELTA_STEP);
          total_animation_delta_time -= ANIMATION_DELTA_STEP;
        }
      }
      
      // Place your draw function here and add data to the draw vector.
      model_data = vec!(
        (
          vec!(0.0, 0.0, 0.0, 0.0, // (x y z nothing) defines where it should place the model, 4th parameter is not used.
               1.0, 1.0, 1.0),     // (scale x y z) defines what it should scale by.
               "example_model"     // Reference name for the model loaded in with vulkan.model_load function.
        ),
      );
      
      match event {
          Event::WindowEvent { event, .. } => match event {
              WindowEvent::CloseRequested => {
                  *control_flow = ControlFlow::Exit; // Happens when the X is clicked by user
              },
              
              // Close the window when escape is pressed
              WindowEvent::KeyboardInput {
                  input:
                  KeyboardInput {
                      virtual_keycode: Some(VirtualKeyCode::Escape),
                      ..
                  },
                  ..
              } => {
                *control_flow = ControlFlow::Exit // Tell the window it should close
              },
              WindowEvent::Resized(dimensions) => {
                // Update the render area
                vulkan.recreate_swapchain(dimensions.width, dimensions.height);
                screen_resolution[0] = dimensions.width;
                screen_resolution[1] = dimensions.height;
              },
              window_event => {
                handle_window_event(window_event, _delta_time);
              },
          },
          Event::DeviceEvent { event, .. } => match event {
            device_event => {
              handle_device_event(device_event, &mut device_keys, vulkan.mut_camera(), _delta_time);
            }
          },
          Event::MainEventsCleared => {
            // Currently only 2D or only 3D works at a time, in future will fix so both can happen at the same time.
            vulkan.draw_model(model_data);
          },
          Event::LoopDestroyed => {
            vulkan.destroy();
          }
          _ => (),
      }
  });
}

fn handle_device_event(event: DeviceEvent, device_keys: &mut Vec<VirtualKeyCode>, camera: &mut Camera, delta_time: f32) {
  match event {
    DeviceEvent::MouseMotion { delta: (mx, my) } => {
      let dx = -mx as f32*0.1;
      let dy = -my as f32*0.1;
      
      camera.update_rotate([dy, dx, 0.0]);
    },
    DeviceEvent::MouseWheel { delta } => {
      match delta {
        MouseScrollDelta::LineDelta(_x, y) => {
          camera.update_translate([0.0, 0.0, y as f32 * 1000.0 * delta_time]);
        },
        _ => {},
      }
    },
    // Hardware inputs, useful for things like game movements
    DeviceEvent::Key(key) => {
      match key.state {
        ElementState::Pressed => {
          if let Some(key_code) = key.virtual_keycode {
            device_keys.push(key_code);
          }
        },
        ElementState::Released => {
          if let Some(key_code) = key.virtual_keycode {
            let mut i = 0;
            while i < device_keys.len() {
              if device_keys[i] == key_code {
                device_keys.remove(i);
              }
              
              i += 1;
            }
          }
        }
      }
    },
    _ => {},
  }
}

fn handle_window_event(event: WindowEvent, _delta_time: f32) {
  match event {
    // Software Inputs useful for typing words etc.
    WindowEvent::KeyboardInput {input, ..} => {
      let _key_code = input.scancode;
    },
    WindowEvent::MouseInput {state, button, ..} => {
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
