extern crate maat_graphics;

use maat_graphics::winit;

use winit::{
  event::VirtualKeyCode,
  event_loop::EventLoop,
};

use maat_graphics::{MaatGraphics, VkWindow, MaatEvent};

const APP_NAME: &str = "MaatGraphics - Minimal 3D Example";

fn main() {
  let create_window_size: [u32; 2] = [1280, 720];
  let mut screen_resolution = [1, 1];
  
  let event_loop = EventLoop::new();
  let mut window = VkWindow::new(APP_NAME, create_window_size[0], create_window_size[1], &event_loop, &mut screen_resolution);
  let mut vulkan = MaatGraphics::new(&mut window, screen_resolution);
  
  vulkan.load_texture("orientation", "./textures/negativeviewportheight.jpg");
  vulkan.load_texture("rust_crab", "./textures/rust.png");
  
  vulkan.load_model("example_model", "./models/sample_model.glb");
  
  let _model_boundries = vulkan.all_model_bounding_boxes();
  
  vulkan.mut_camera().set_movement_speed(10.0);
  
  MaatGraphics::run(vulkan, event_loop, move |event| {
    match event {
      MaatEvent::MouseMoved(mx, my, camera) => {
        // Do stuff when mouse is moved
        let dx = -mx as f32*0.1;
        let dy = -my as f32*0.1;
        
        camera.update_rotate([dy, dx, 0.0]);
      }
      MaatEvent::RealTimeInput(device_keys, camera, delta_time) => {
         // Do stuff with camera/non-game object/non-deterministic 
        if device_keys.contains(&VirtualKeyCode::W) {
          camera.forward(delta_time);
        }
        if device_keys.contains(&VirtualKeyCode::A) {
          camera.left(delta_time);
        }
        if device_keys.contains(&VirtualKeyCode::S) {
          camera.backward(delta_time);
        }
        if device_keys.contains(&VirtualKeyCode::D) {
          camera.right(delta_time);
        }
      },
      MaatEvent::Update(_device_keys, _software_keys, _camera, _delta_time) => {
        // Update game objects and physics
      },
      MaatEvent::Resized(width, height) => {
        screen_resolution[0] = width;
        screen_resolution[1] = height;
      }
      MaatEvent::Draw(texture_data, model_data) => {
        // Place your draw function here and add data to the draw vector.
        let text_x = 720.0;
        let text_y = 700.0;
        let text_size = 32.0;
        let text_outline = 0.5;
        let text_edge_width = 0.1;
        
        // Get draw data
        *texture_data = vec!(
          (vec!(0.0, 0.0, 720.0, 720.0,  // x y scale_x scale_y
                0.0, 0.0, 1.0, 1.0, // r g b a
                1.0, 0.0), // use texture, rotation
                "orientation", // Texture reference name
                None), // None to tell it that it is a texture, when this value is Some() it is treated as Text
          
          // Draw Crab at 150, 150 with size of 573x300 and rotate it by 45 degrees
          (vec!(150.0, 150.0, 573.0, 300.0,
                1.0, 0.0, 1.0, 1.0,
                1.0, 45.0), 
                "rust_crab", None),
          
          // Example drawing text
          (vec!(text_x, text_y, text_size, 0.0, // x, y, size
                1.0,    1.0,    1.0,       1.0, // r g b a (outline colour)
                text_outline, text_edge_width), // text outline, text edge width, this are Signed Distanced feild parameters for text.
                "", Some("The quick brown fox")),
           (vec!(text_x, text_y-text_size, text_size, 0.0, // x, y, size
                 1.0,    1.0,              1.0,       1.0, // r g b a (outline colour)
                 text_outline, text_edge_width), // text outline, text edge width, this are Signed Distanced feild parameters for text.
                 "", Some("jumped over the fence.")),
           
        );
        
        // Place your draw function here and add data to the draw vector.
        *model_data = vec!(
          (
            vec!(0.0, 0.0, 0.0, 0.0, // (x y z nothing) defines where it should place the model, 4th parameter is not used.
                 1.0, 1.0, 1.0),     // (scale x y z) defines what it should scale by.
                 "example_model"     // Reference name for the model loaded in with vulkan.model_load function.
          ),
        );
      },
      _ => {},
    }
  });
}
