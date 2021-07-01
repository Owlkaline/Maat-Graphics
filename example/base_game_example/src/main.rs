extern crate maat_graphics;

use maat_graphics::{winit};

use winit::{
  event::{Event, KeyboardInput, VirtualKeyCode, MouseButton, ElementState, WindowEvent},
  event_loop::{ControlFlow, EventLoop}
};

use std::time::Instant;

use maat_graphics::{MaatGraphics, VkWindow, Camera};

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
  
  vulkan.load_text("test", "Hello, Please kill me", 10.0);
  
  vulkan.load_model("floor", "./models/owned/floor.glb");
  vulkan.load_model("orientation_test", "./models/OrientationTest.glb");
  vulkan.load_model("animation_test", "./models/CesiumMan.glb");
  
  let mut delta_time = 0.0;
  let mut last_time = Instant::now();
  
  let mut total_delta_time = 0.0;
  
  let mut last_mouse_pos = [0.0, 0.0];
  let mut dxy = [0.0, 0.0];
  
  vulkan.mut_camera().set_movement_speed(1000.0);
  
  event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;
      
      delta_time = last_time.elapsed().subsec_nanos() as f32 / 1000000000.0 as f32;
      last_time = Instant::now();
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
                //println!("resized");
                vulkan.recreate_swapchain(dimensions.width, dimensions.height);
                screen_resolution.width = dimensions.width;
                screen_resolution.height = dimensions.height;
              },
              event => {
                handle_event(event, screen_resolution, &mut last_mouse_pos, &mut dxy, vulkan.mut_camera(), delta_time);
              },
          },
          Event::MainEventsCleared => {
           /* vulkan.draw_texture(vec!(
              (vec!(0.0, 0.0, 720.0, 720.0,  // x y scale_x scale_y
                    0.0, 0.0, 1.0, 1.0, // r g b a
                    1.0, 45.0), // use texture, rotation
               "orientation", None),
              (vec!(150.0, 150.0, 573.0, 300.0,
                    1.0, 0.0, 1.0, 1.0,
                    1.0, 0.0), 
               "rust_crab", None),
               
              // Example using pre calculated text that doesn't change (Very fast)
              (vec!(0.0, 20.0, 0.0, 0.0, // x, y
                    1.0, 1.0, 1.0, 1.0, //  r g b a (outline colour)
                    0.5, 0.1), // outline, width
               "test", Some("")),
              
              // Example using text that changes every or most fames (slower)
              (vec!(0.0, 0.0, 0.0, 0.0, // x, y
                    1.0, 1.0, 1.0, 1.0, // r g b a (outline colour)
                    0.5, 0.1), // outline, width
               "", Some("Nah, dont kill me")),
            ));*/
            vulkan.draw_model(
              vec!(
                (Vec::new(), "floor"),
                (Vec::new(), "orientation_test"),
                (Vec::new(), "animation_test"),
              )
            );
          },
          Event::LoopDestroyed => {
            vulkan.destroy();
          }
          _ => (),
      }
  });
}

fn handle_event(event: WindowEvent, screen_resolution: vk::Extent2D,
                last_mouse_pos: &mut [f32; 2], dxy: &mut [f32; 2], camera: &mut Camera, delta_time: f32) {
  
  match event {
    WindowEvent::KeyboardInput {device_id: _, input: key, is_synthetic: _} => {
      if let Some(key_code) = key.virtual_keycode {
        println!("KeyInput: {:?}", key_code);
        match key_code {
          VirtualKeyCode::W => {
            camera.forward(delta_time);
          },
          VirtualKeyCode::A => {
            camera.left(delta_time);
          },
          VirtualKeyCode::S => {
            camera.backward(delta_time);
          },
          VirtualKeyCode::D => {
            camera.right(delta_time);
          },
          _ => {
            
          }
        }
      }
    },
    WindowEvent::MouseWheel { device_id: _, delta, phase: _, ..} => {
      match delta {
        winit::event::MouseScrollDelta::LineDelta(_x, y) => {
          camera.update_translate([0.0, 0.0, y as f32 * 0.5]);
        },
        _ => {},
        /*winit::event::MouseScrollDelta::PixelData(phys_pos) => {
          
        }*/
      }
    },
    WindowEvent::CursorMoved { device_id: _, position: pos, ..} => {
      let x: f64 = pos.x;
      let y: f64 = screen_resolution.height as f64 - pos.y;
      
      let dx = last_mouse_pos[0]-x as f32;
      let dy = y as f32-last_mouse_pos[1];
      
      dxy[0] = if dx > 0.0 { 1.0 } else if dx < 0.0 { -1.0 } else { 0.0 };
      dxy[1] = if dy > 0.0 { 1.0 } else if dy < 0.0 { -1.0 } else { 0.0 };
      
      camera.update_rotate([dxy[1], dxy[0], 0.0]);
      
      *last_mouse_pos = [x as f32, y as f32];
    },
    WindowEvent::MouseInput {device_id: _, state, button, ..} => {
      //camera.update_rotate([dxy[1], -dxy[0], 0.0]);
      
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
          //camera.update_translate([0.0, 0.0, dxy[1]*0.005]);
        },
        MouseButton::Middle => {
          //camera.update_translate([-dxy[0]*0.01, -dxy[1] * 0.01, 0.0]);
        },
        MouseButton::Other(_id) => {
          
        },
      }
    },
    _ => {},
  }
}
