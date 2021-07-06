extern crate maat_graphics;

use maat_graphics::{winit, Math};

use winit::{
  event::{Event, KeyboardInput, VirtualKeyCode, MouseButton, ElementState, WindowEvent, DeviceEvent},
  event_loop::{ControlFlow, EventLoop}
};

use std::time::Instant;

use maat_graphics::{MaatGraphics, VkWindow, Camera};

use maat_graphics::ash::vk;

const APP_NAME: &str = "MaatGraphics - Example";
//const WINDOW_SIZE: [u32; 2] = [1280, 720];

const DELTA_STEP: f32 = 0.001;
const ANIMATION_DELTA_STEP: f32 = 0.01;

fn main() {
  let create_window_size: [u32; 2] = [1280, 720];
  let mut screen_resolution = [1, 1];
  
  let event_loop = EventLoop::new();
  let mut window = VkWindow::new(APP_NAME, create_window_size[0], create_window_size[1], &event_loop, &mut screen_resolution);
  
  let mut vulkan = MaatGraphics::new(&mut window, screen_resolution);
  
  //vulkan.load_texture("orientation", "./textures/negativeviewportheight.jpg");
  vulkan.load_texture("rust_crab", "./textures/rust.png");
  
  //vulkan.load_text("test", "The quick brown fox jumps over the lazy dog.", 10.0);
  
  vulkan.load_model("floor", "./models/owned/floor.glb");
  //vulkan.load_model("orientation_test", "./models/OrientationTest.glb");
  //vulkan.load_model("animation_test", "./models/CesiumMan.glb");
  //vulkan.load_model("aniamted_cube", "./models/AnimatedCube.glb");
  //vulkan.load_model("helmet", "./models/DamagedHelmet.glb");
  //vulkan.load_model("box_animated", "./models/BoxAnimated.glb");
  //vulkan.load_model("simple_rigged", "./models/RiggedSimple.glb");
  //vulkan.load_model("simple_skin", "./models/SimpleSkin.glb");
  //vulkan.load_model("interpolation_text", "./models/InterpolationTest.glb");
  //vulkan.load_model("cesium", "./models/CesiumMan.glb");
  //vulkan.load_model("milk_truck", "./models/CesiumMilkTruck.glb");
  //vulkan.load_model("brain_stem", "./models/BrainStem.glb");
  //vulkan.load_model("vc", "./models/vc.glb");
  //vulkan.load_model("vertex_colour_test", "./models/VertexColorTest.glb");
 // vulkan.load_model("Ult_animation_test", "./models/owned/offsetscale.glb");
  vulkan.load_model("space_ship", "./models/owned/scorpionspaceship.glb");
  vulkan.load_model("axis", "./models/owned/Axis.glb");
  
  let ship_size = vulkan.model_bounding_box("space_ship");
  let floor_size = vulkan.model_bounding_box("floor");
  //println!("ship: {:?}, floor: {:?}", ship_size, floor_size);
  
  let mut delta_time = 0.0;
  let mut last_time = Instant::now();
  
  let mut total_delta_time = 0.0;
  let mut total_animation_delta_time = 0.0;
  
  vulkan.mut_camera().set_movement_speed(10.0);
  
  let mut device_keys = Vec::new();
  
  let mut ship_pos = [0.0, 50.0, 0.0];
  let fall_speed = 10.0;
  let ship_speed = 1.0;
  
  event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;
      
      delta_time = last_time.elapsed().subsec_nanos() as f32 / 1000000000.0 as f32;
      last_time = Instant::now();
      total_delta_time += delta_time as f32;
      total_animation_delta_time += delta_time as f32;
      
      let mut model_data: Vec<(Vec<f32>, &str)> = Vec::new();
      let mut texture_data = Vec::new();
      
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
          
          if device_keys.contains(&VirtualKeyCode::R) {
            ship_pos = [0.0, 50.0, 0.0];
          }
          
          if device_keys.contains(&VirtualKeyCode::Up) {
            ship_pos[2] += ship_speed*DELTA_STEP;
          }
          if device_keys.contains(&VirtualKeyCode::Down) {
            ship_pos[2] -= ship_speed*DELTA_STEP;
          }
          
          if device_keys.contains(&VirtualKeyCode::Left) {
            ship_pos[0] -= ship_speed*DELTA_STEP;
          }
          if device_keys.contains(&VirtualKeyCode::Right) {
            ship_pos[0] += ship_speed*DELTA_STEP;
          }
          
          example_collision_update(&mut ship_pos, &ship_size, &floor_size, fall_speed, DELTA_STEP);
          
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
      
      // Get draw data
      texture_data = vec!(
        /*(vec!(0.0, 0.0, 720.0, 720.0,  // x y scale_x scale_y
              0.0, 0.0, 1.0, 1.0, // r g b a
              1.0, 45.0), // use texture, rotation
         format!("orientation"), None),*/
        (vec!(150.0, 150.0, 573.0, 300.0,
              1.0, 0.0, 1.0, 1.0,
              1.0, 0.0), 
         "rust_crab", None),
         
        // Example using pre calculated text that doesn't change (Very fast)
       /* (vec!(0.0, 20.0, 10.0, 0.0, // x, y
              1.0, 1.0, 1.0, 1.0, //  r g b a (outline colour)
              0.5, 0.1), // outline, width
         format!("test"), Some(format!(""))),*/
        
        // Example using text that changes every or most fames (slower)
        (vec!(0.0, 0.0, 64.0, 0.0, // x, y
              1.0, 1.0, 1.0, 1.0, // r g b a (outline colour)
              0.5, 0.1), // outline, width
         "", Some("Hello, welcome gg")),
      );
      
      model_data = vec!(
        //(Vec::new(), "orientation_test"),
        //(Vec::new(), "animation_test"),
        //(Vec::new(), "box_animated")
        //(Vec::new(), "helmet")
        //(Vec::new(), "simple_rigged")
        //(Vec::new(), "simple_skin")
        //(Vec::new(), "interpolation_text"),
        //(Vec::new(), "cesium"),
        //(Vec::new(), "vertex_colour_test"),
        (vec!(ship_pos[0], ship_pos[1], ship_pos[2], 0.0,
              1.0, 1.0, 1.0), "space_ship"),
        (vec!( 0.0, 0.0,  0.0, 0.0,
              1.0, 1.0, 1.0), "floor"),
        (vec!(0.0, 0.0, 0.0, 0.0,
              1.0, 1.0, 1.0), "axis"),
      );
      /*
      for x in 0..10 {
        for z in 0..10 {
          model_data.push((vec!(2.0*x as f32 - 10.0, 0.0, 2.0*z as f32-10.0), "floor"));
        }
      }
      */
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
                screen_resolution[0] = dimensions.width;
                screen_resolution[1] = dimensions.height;
              },
              window_event => {
                handle_window_event(window_event, delta_time);
              },
          },
          Event::DeviceEvent { event, .. } => match event {
            device_event => {
              handle_device_event(device_event, &mut device_keys, vulkan.mut_camera(), delta_time);
            }
          },
          Event::MainEventsCleared => {
            //vulkan.draw_texture(texture_data);
            vulkan.draw_model(model_data);
          },
          Event::LoopDestroyed => {
            vulkan.destroy();
          }
          _ => (),
      }
  });
}

fn example_collision_update(ship_pos: &mut [f32; 3], ship_size: &([f32; 3], [f32; 3]), floor_size: &([f32; 3], [f32; 3]), 
                            fall_speed: f32, delta_time: f32) {
  ship_pos[1] -= fall_speed*DELTA_STEP;
  
  let mut collision_confirmed = false;

  let floor_x = 0.0;
  let floor_z = 0.0;
  let floor_pos = [floor_x, 0.0, floor_z];
  
  let ship_min_x = Math::vec3_add(*ship_pos, ship_size.0);
  let ship_max_x = Math::vec3_add(*ship_pos, ship_size.1);
  
  let floor_min_x = Math::vec3_add(floor_pos, floor_size.0);
  let floor_max_x = Math::vec3_add(floor_pos, floor_size.1);
  
  if intersect_AABB(ship_min_x, ship_max_x, floor_min_x, floor_max_x) {
    ship_pos[1] = 0.0+floor_size.1[1];
    collision_confirmed = true;
  }
}

pub fn is_point_inside_AABB(point: [f32; 3], box_location: [f32; 3], box_size: [f32; 3]) -> bool {
  let min_x = box_location[0] - box_size[0]*0.5;
  let max_x = box_location[0] + box_size[0]*0.5;
  let min_y = box_location[1] - box_size[1]*0.5;
  let max_y = box_location[1] + box_size[1]*0.5;
  let min_z = box_location[2] - box_size[2]*0.5;
  let max_z = box_location[2] + box_size[2]*0.5;
  
  (point[0] >= min_x && point[0] <= max_x) && 
  (point[1] >= min_y && point[1] <= max_y) && 
  (point[2] >= min_z && point[2] <= max_z)
}

pub fn intersect_AABB(box_a_min: [f32; 3], box_a_max: [f32; 3], box_b_min: [f32; 3], box_b_max: [f32; 3]) -> bool {
  let a_min_x = box_a_min[0];
  let a_max_x = box_a_max[0];
  let a_min_y = box_a_min[1];
  let a_max_y = box_a_max[1];
  let a_min_z = box_a_min[2];
  let a_max_z = box_a_max[2];
  
  let b_min_x = box_b_min[0];
  let b_max_x = box_b_max[0];
  let b_min_y = box_b_min[1];
  let b_max_y = box_b_max[1];
  let b_min_z = box_b_min[2];
  let b_max_z = box_b_max[2];
  
  (a_min_x <= b_max_x && a_max_x >= b_min_x) &&
  (a_min_y <= b_max_y && a_max_y >= b_min_y) &&
  (a_min_z <= b_max_z && a_max_z >= b_min_z)
}
/*
pub fn intersect_AABB(box_a_location: [f32; 3], box_a_size: [f32; 3], box_b_location: [f32; 3], box_b_size: [f32; 3]) -> bool {
  let a_min_x = box_a_location[0] - box_a_size[0]*0.5;
  let a_max_x = box_a_location[0] + box_a_size[0]*0.5;
  let a_min_y = box_a_location[1] - box_a_size[1]*0.5;
  let a_max_y = box_a_location[1] + box_a_size[1]*0.5;
  let a_min_z = box_a_location[2] - box_a_size[2]*0.5;
  let a_max_z = box_a_location[2] + box_a_size[2]*0.5;
  
  let b_min_x = box_b_location[0] - box_b_size[0]*0.5;
  let b_max_x = box_b_location[0] + box_b_size[0]*0.5;
  let b_min_y = box_b_location[1] - box_b_size[1]*0.5;
  let b_max_y = box_b_location[1] + box_b_size[1]*0.5;
  let b_min_z = box_b_location[2] - box_b_size[2]*0.5;
  let b_max_z = box_b_location[2] + box_b_size[2]*0.5;
  
  (a_min_x <= b_max_x && a_max_x >= b_min_x) &&
  (a_min_y <= b_max_y && a_max_y >= b_min_y) &&
  (a_min_z <= b_max_z && a_max_z >= b_min_z)
}*/

fn handle_device_event(event: DeviceEvent, device_keys: &mut Vec<VirtualKeyCode>, camera: &mut Camera, delta_time: f32) {
  match event {
    DeviceEvent::MouseMotion { delta: (mx, my) } => {
      let sens = 5.0;

      let dx = -mx as f32;
      let dy = -my as f32;
      
      camera.update_rotate([dy, dx, 0.0]);
    },
    DeviceEvent::MouseWheel { delta } => {
      match delta {
        winit::event::MouseScrollDelta::LineDelta(_x, y) => {
          camera.update_translate([0.0, 0.0, y as f32 * 1000.0 * delta_time]);
        },
        _ => {},
        /*winit::event::MouseScrollDelta::PixelData(phys_pos) => {
          
        }*/
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

fn handle_window_event(event: WindowEvent, delta_time: f32) {
  match event {
    // Software Inputs useful for typing words etc.
    WindowEvent::KeyboardInput {input, ..} => {
      let key_code = input.scancode;
      //println!("KeyInput: {:?}", key_code);
    },
    WindowEvent::MouseInput {state, button, ..} => {
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
