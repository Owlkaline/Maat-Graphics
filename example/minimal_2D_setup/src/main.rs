extern crate maat_graphics;

use maat_graphics::winit;
use maat_graphics::{MaatEvent, MaatGraphics, VkWindow};
use winit::event_loop::EventLoop;

const APP_NAME: &str = "MaatGraphics - Minimal 2D Example";

fn main() {
  let create_window_size: [u32; 2] = [1280, 720];
  let mut screen_resolution = [1, 1];

  let event_loop = EventLoop::new();
  let mut window = VkWindow::new(
    APP_NAME,
    create_window_size[0],
    create_window_size[1],
    &event_loop,
    &mut screen_resolution,
  );

  let mut vulkan = MaatGraphics::new(&mut window, screen_resolution, "./fonts/DOSVGA");

  vulkan.load_texture("orientation", "./textures/negativeviewportheight.jpg");
  vulkan.load_texture("rust_crab", "./textures/rust.png");

  MaatGraphics::run(vulkan, window, event_loop, move |event| {
    match event {
      MaatEvent::MouseMoved(_mx, _my) => {
        // Do stuff when mouse is moved
      }
      MaatEvent::Update(_device_keys, _software_keys, _camera, _delta_time, _) => {
        // Update game objects and physics
      }
      MaatEvent::Resized(width, height) => {
        screen_resolution[0] = width;
        screen_resolution[1] = height;
      }
      MaatEvent::Draw(texture_data, model_data) => {
        *model_data = Vec::new() as Vec<(Vec<f32>, String)>;

        // Place your draw function here and add data to the draw vector.
        let text_x = 720.0;
        let text_y = 700.0;
        let text_size = 32.0;
        let text_outline = 0.5;
        let text_edge_width = 0.1;

        // Get draw data
        *texture_data = vec![
          (
            vec![
              0.0, 0.0, 720.0, 720.0, // x y scale_x scale_y
              0.0, 0.0, 1.0, 1.0, // r g b a
              1.0, 0.0,
            ], // use texture, rotation
            "orientation", // Texture reference name
            None,
          ), // None to tell it that it is a texture, when this value is Some() it is treated as Text
          // Draw Crab at 150, 150 with size of 573x300 and rotate it by 45 degrees
          (
            vec![150.0, 150.0, 573.0, 300.0, 1.0, 0.0, 1.0, 1.0, 1.0, 45.0],
            "rust_crab",
            None,
          ),
          // Example drawing text
          (
            vec![
              text_x,
              text_y,
              text_size,
              0.0, // x, y, size
              1.0,
              1.0,
              1.0,
              1.0, // r g b a (outline colour)
              text_outline,
              text_edge_width,
            ], // text outline, text edge width, this are Signed Distanced feild parameters for text.
            "",
            Some("The quick brown fox"),
          ),
          (
            vec![
              text_x,
              text_y - text_size,
              text_size,
              0.0, // x, y, size
              1.0,
              1.0,
              1.0,
              1.0, // r g b a (outline colour)
              text_outline,
              text_edge_width,
            ], // text outline, text edge width, this are Signed Distanced feild parameters for text.
            "",
            Some("jumped over the fence."),
          ),
        ];
      }
      _ => {}
    }
  });
}
