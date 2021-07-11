extern crate maat_graphics;

use maat_graphics::winit::{
  event::{Event, KeyboardInput, VirtualKeyCode, MouseButton, ElementState, WindowEvent, DeviceEvent},
  event_loop::{ControlFlow, EventLoop}
};

use std::time::Instant;

use maat_graphics::{MaatGraphics, VkWindow};

const APP_NAME: &str = "MaatGraphics - Minimal 2D Example";

const DELTA_STEP: f32 = 0.001;

fn main() {
  let create_window_size: [u32; 2] = [1280, 720];
  let mut screen_resolution = [1, 1];
  
  let event_loop = EventLoop::new();
  let mut window = VkWindow::new(APP_NAME, create_window_size[0], create_window_size[1], &event_loop, &mut screen_resolution);
  
  let mut vulkan = MaatGraphics::new(&mut window, screen_resolution);
  
  vulkan.load_texture("orientation", "./textures/negativeviewportheight.jpg");
  vulkan.load_texture("rust_crab", "./textures/rust.png");

  let mut _delta_time = 0.0;
  let mut last_time = Instant::now();
  let mut total_delta_time = 0.0;
  
  let mut device_keys = Vec::new();
  
  event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;
      
      _delta_time = last_time.elapsed().subsec_nanos() as f32 / 1000000000.0 as f32;
      last_time = Instant::now();
      total_delta_time += _delta_time as f32;
      
      let texture_data;
      
      if total_delta_time > DELTA_STEP {
        let delta_steps = (total_delta_time / DELTA_STEP).floor() as usize;
        
        for _ in 0..delta_steps {
          //F(DELTA_STEP); // update
          total_delta_time -= DELTA_STEP;
        }
      }
      
      // Place your draw function here and add data to the draw vector.
      let text_x = 720.0;
      let text_y = 700.0;
      let text_size = 32.0;
      let text_outline = 0.5;
      let text_edge_width = 0.1;
      
      // Get draw data
      texture_data = vec!(
        (vec!(0.0, 0.0, 720.0, 720.0,  // x y scale_x scale_y
              0.0, 0.0, 1.0, 1.0, // r g b a
              1.0, 0.0), // use texture, rotation
              "orientation", // Texture reference name
              None), // None to tell it that it is a texture, when this value is Some() it is treated as Text
        (vec!(150.0, 150.0, 573.0, 300.0,
              1.0, 0.0, 1.0, 1.0,
              1.0, 45.0), 
         "rust_crab", None),
        
        // Example using text that changes every or most fames (slower)
        (vec!(text_x, text_y, text_size, 0.0, // x, y, size
              1.0,    1.0,    1.0,       1.0, // r g b a (outline colour)
              text_outline, text_edge_width), // text outline, text edge width, this are Signed Distanced feild parameters for text.
         "", Some("The quick brown fox")),
         (vec!(text_x, text_y-text_size, text_size, 0.0, // x, y, size
               1.0,    1.0,              1.0,       1.0, // r g b a (outline colour)
               text_outline, text_edge_width), // text outline, text edge width, this are Signed Distanced feild parameters for text.
         "", Some("jumped over the fence.")),
         
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
              handle_device_event(device_event, &mut device_keys, _delta_time);
            }
          },
          Event::MainEventsCleared => {
            // Currently only 2D or only 3D works at a time, in future will fix so both can happen at the same time.
            vulkan.draw_texture(texture_data);
          },
          Event::LoopDestroyed => {
            vulkan.destroy();
          }
          _ => (),
      }
  });
}

fn handle_device_event(event: DeviceEvent, device_keys: &mut Vec<VirtualKeyCode>, _delta_time: f32) {
  match event {
    DeviceEvent::MouseMotion { delta: (_mx, _my) } => {
      
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
      
      // Mouse buttons
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
