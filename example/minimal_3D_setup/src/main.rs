extern crate maat_graphics;

use maat_graphics::winit;

use winit::{event_loop::EventLoop, keyboard::KeyCode};

use maat_graphics::{MaatEvent, MaatGraphics, VkWindow};

const APP_NAME: &str = "MaatGraphics - Minimal 3D Example";

fn main() {
  let create_window_size: [u32; 2] = [1280, 720];
  let mut screen_resolution = [1, 1];

  let event_loop = EventLoop::new().unwrap();
  let mut window = VkWindow::new(
    APP_NAME,
    create_window_size[0],
    create_window_size[1],
    &event_loop,
    &mut screen_resolution,
  );
  let mut vulkan = MaatGraphics::new(
    &mut window,
    &event_loop,
    screen_resolution,
    "./fonts/DOSVGA",
  );

  vulkan.load_model("example_model", include_bytes!("../models/RedHexagon.glb"));

  //let _model_boundries = vulkan.all_model_bounding_boxes();

  vulkan.mut_camera().set_movement_speed(10.0);

  MaatGraphics::run(vulkan, window, event_loop, move |event| {
    match event {
      MaatEvent::MouseMoved(mx, my) => {
        // Do stuff when mouse is moved
        let dx = -mx as f32 * 0.1;
        let dy = -my as f32 * 0.1;

        //camera.update_rotate([dy, dx, 0.0]);
      }
      MaatEvent::Update(device_keys, keys, camera, delta_time, something) => {
        // Do stuff with camera/non-game object/non-deterministic
        //if device_keys.contains(&(KeyCode::KeyW) {
        //  camera.forward(delta_time);
        //}
        //if device_keys.contains(&KeyCode::KeyA) {
        //  camera.left(delta_time);
        //}
        //if device_keys.contains(&KeyCode::KeyS) {
        //  camera.backward(delta_time);
        //}
        //if device_keys.contains(&KeyCode::KeyD) {
        //  camera.right(delta_time);
        //}
      }
      MaatEvent::Update(_device_keys, _software_keys, _camera, _delta_time, something) => {
        // Update game objects and physics
      }
      MaatEvent::Resized(width, height) => {
        screen_resolution[0] = width;
        screen_resolution[1] = height;
      }
      MaatEvent::Draw(texture_data, model_data) => {
        *texture_data = Vec::new(); // as Vec<(Vec<f32>, String, Option<String>)>;

        // Place your draw function here and add data to the draw vector.
        *model_data = vec![(
          vec![
            0.0, 0.0, 0.0,
            0.0, // (x y z nothing) defines where it should place the model, 4th parameter is not used.
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
          ], // (scale x y z) defines what it should scale by.
          "example_model", // Reference name for the model loaded in with vulkan.model_load function.
        )];
      }
      _ => {}
    }
  })
}
