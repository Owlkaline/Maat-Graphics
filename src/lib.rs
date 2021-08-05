#![allow(dead_code)]

pub extern crate ash;
pub extern crate gilrs;
pub extern crate glam;
pub extern crate image;
pub extern crate winit;

pub use crate::extra::{
  gltf_loader::CollisionInformation, Math, Swizzle2, Swizzle3, Swizzle4, Vector2, Vector3, Vector4,
  VectorMath,
};
pub use crate::shader_handlers::{font::FontChar, Camera};
pub use crate::vkwrapper::VkWindow;

mod extra;
mod shader_handlers;
mod vkwrapper;

use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;

use ash::vk;
use gilrs::{ev::EventType, Axis, Button, Event as GpEvent, GamepadId, Gilrs};
use winit::{
  event::{
    DeviceEvent, ElementState, Event, MouseButton as WMouseButton, VirtualKeyCode, WindowEvent,
  },
  event_loop::{ControlFlow, EventLoop},
  window::Fullscreen,
};

use crate::shader_handlers::{ModelHandler, TextureHandler};
use crate::vkwrapper::{ComputeShader, DescriptorPoolBuilder, DescriptorSet, Image, Vulkan};

const DELTA_STEP: f32 = 0.001;
const ANIMATION_DELTA_STEP: f32 = 0.01;
const MAX_LOOPS_PER_FRAME: u32 = 5;

pub enum MouseInput {
  Left(bool),
  Right(bool),
  Middle(bool),
}

pub enum AxisInput {
  X(Direction, f32),
  Y(Direction, f32),
}

pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}

pub enum ControllerInput {
  ActionButton(Direction), // A b x y, circle cross triangle square
  DPad(Direction),
  Trigger(Direction),
  Trigger2(Direction),
  Stick(Direction),
  Start,
  Select,
}

impl AxisInput {
  pub fn from_axis(axis: Axis, value: f32) -> Option<AxisInput> {
    match axis {
      Axis::LeftStickX => Some(AxisInput::X(Direction::Left, value)),
      Axis::LeftStickY => Some(AxisInput::Y(Direction::Left, value)),
      Axis::RightStickX => Some(AxisInput::X(Direction::Right, value)),
      Axis::RightStickY => Some(AxisInput::Y(Direction::Right, value)),
      _ => None,
    }
  }
}

impl ControllerInput {
  pub fn from_button(button: Button) -> Option<ControllerInput> {
    match button {
      Button::North => Some(ControllerInput::ActionButton(Direction::Up)),
      Button::East => Some(ControllerInput::ActionButton(Direction::Right)),
      Button::South => Some(ControllerInput::ActionButton(Direction::Down)),
      Button::West => Some(ControllerInput::ActionButton(Direction::Left)),
      Button::LeftTrigger => Some(ControllerInput::Trigger(Direction::Left)),
      Button::LeftTrigger2 => Some(ControllerInput::Trigger2(Direction::Left)),
      Button::RightTrigger => Some(ControllerInput::Trigger(Direction::Right)),
      Button::RightTrigger2 => Some(ControllerInput::Trigger2(Direction::Right)),
      Button::LeftThumb => Some(ControllerInput::Stick(Direction::Left)),
      Button::RightThumb => Some(ControllerInput::Stick(Direction::Right)),
      Button::DPadUp => Some(ControllerInput::DPad(Direction::Up)),
      Button::DPadDown => Some(ControllerInput::DPad(Direction::Down)),
      Button::DPadLeft => Some(ControllerInput::DPad(Direction::Left)),
      Button::DPadRight => Some(ControllerInput::DPad(Direction::Right)),
      Button::Start => Some(ControllerInput::Start),
      Button::Select => Some(ControllerInput::Select),
      _ => None,
    }
  }
}

pub enum DrawMode {
  Polygon,
  Wireframe,
  PointsOnly,
}

pub enum MaatSetting {
  DrawMode(DrawMode),
  MouseVisibility(bool),
  CaptureMouse(bool),
  Window(bool, bool), // Window(is fullscreen, broderless)
  LimitFps(f32, bool),
}

pub enum MaatEvent<'a, T: Into<String>, L: Into<String>, S: Into<String>> {
  Draw(
    &'a mut Vec<(Vec<f32>, T, Option<L>)>,
    &'a mut Vec<(Vec<f32>, S)>,
  ),
  FixedUpdate(&'a Vec<VirtualKeyCode>, &'a Vec<u32>, &'a mut Camera, f32),
  Update(
    &'a Vec<VirtualKeyCode>,
    &'a Vec<u32>,
    &'a mut Camera,
    f32,
    &'a mut bool,
  ),
  MouseMoved(f64, f64),
  MouseButton(MouseInput),
  MouseDelta(f64, f64, &'a mut Camera), // delta x delta y, camera
  ScrollDelta(f32, f32, &'a mut Camera), // scroll x, y, camera
  GamepadButton(ControllerInput, bool),
  GamepadAxis(AxisInput),
  Resized(u32, u32),
  UpdateMaatSettings(&'a Vec<VirtualKeyCode>, &'a mut Vec<MaatSetting>),
  // NewModelLoaded -> HashMap<>
  UnhandledWindowEvent(WindowEvent<'a>),
  UnhandledDeviceEvent(DeviceEvent),
}

pub struct MaatGraphics {
  vulkan: Vulkan,
  texture_handler: TextureHandler,
  model_handler: ModelHandler,
  compute_descriptor_pool: vk::DescriptorPool,
  compute_shader: ComputeShader,
  compute_descriptor_sets: DescriptorSet,

  gamepads: Option<Gilrs>,
  active_controller: Option<GamepadId>,
}

impl MaatGraphics {
  pub fn new<T: Into<String>>(
    window: &mut VkWindow,
    screen_resolution: [u32; 2],
    font_location: T,
  ) -> MaatGraphics {
    let screen_resolution = vk::Extent2D {
      width: screen_resolution[0],
      height: screen_resolution[1],
    };
    let mut vulkan = Vulkan::new(window, screen_resolution);

    let compute_descriptor_pool = DescriptorPoolBuilder::new()
      .num_storage(5)
      .build(vulkan.device());
    let compute_descriptor_sets = DescriptorSet::builder()
      .storage_compute()
      .build(vulkan.device(), &compute_descriptor_pool);
    let compute_shader = ComputeShader::new(
      vulkan.device(),
      Cursor::new(&include_bytes!("../shaders/collatz_comp.spv")[..]),
      &compute_descriptor_sets,
    );

    let mut compute_data = vec![64, 32, 8, 12, 96];
    vulkan.run_compute(&compute_shader, &compute_descriptor_sets, &mut compute_data);
    println!("Compute Data: {:?}", compute_data);

    let texture_handler = TextureHandler::new(&mut vulkan, screen_resolution, font_location);
    let model_handler = ModelHandler::new(&mut vulkan, screen_resolution);

    MaatGraphics {
      vulkan,
      texture_handler,
      model_handler,
      compute_descriptor_pool,
      compute_shader,
      compute_descriptor_sets,

      gamepads: None,
      active_controller: None,
    }
  }

  pub fn replace_window(&mut self, window: &mut VkWindow) {
    let extent = self.vulkan.swapchain().screen_resolution();
    let models = self.model_handler.loaded_models();
    let camera = self.camera().clone();

    self.destroy();
    self.vulkan = Vulkan::new(window, extent);
    self.texture_handler = TextureHandler::new(&mut self.vulkan, extent, "./fonts/dejavasans");
    self.model_handler = ModelHandler::new(&mut self.vulkan, extent);
    *self.model_handler.mut_camera() = camera;

    for (model_ref, model) in models {
      self.load_model(model_ref, model);
    }
  }

  pub fn enable_gamepad_input(&mut self) {
    match Gilrs::new() {
      Ok(gilrs_object) => {
        for (id, gamepad) in gilrs_object.gamepads() {
          assert!(gamepad.is_connected());
          println!(
            "Gamepad with id {} and name {} is connected",
            id,
            gamepad.name()
          );
          if self.active_controller.is_none() {
            self.active_controller = Some(id);
          }
        }

        self.gamepads = Some(gilrs_object);
      }
      _ => {
        println!("Failed to create gamepad listener.");
      }
    }
  }

  pub fn load_texture<T: Into<String>>(&mut self, texture_ref: T, texture: T) {
    self
      .texture_handler
      .load_texture(&mut self.vulkan, texture_ref, texture);
  }

  pub fn load_model<T: Into<String>>(&mut self, model_ref: T, model: T) {
    self
      .model_handler
      .load_model(&mut self.vulkan, model_ref, model);
  }

  pub fn instance_render_model<T: Into<String>>(&mut self, model_ref: T) {
    self
      .model_handler
      .create_instance_render_buffer(&mut self.vulkan, model_ref);
  }

  pub fn all_collision_models(&self) -> HashMap<String, CollisionInformation> {
    self.model_handler.all_collision_models()
  }

  //pub fn model_collision_meshes(&self) -> Vec<(String, Vec<[f32; 3]>, Vec<u32>)> {
  //  self.model_handler.model_collision_meshes()
  //}

  pub fn get_font_data(&self) -> (Vec<FontChar>, u32, u32) {
    self.texture_handler.get_font_data()
  }

  pub fn recreate_swapchain(&mut self, width: u32, height: u32) {
    self.vulkan.swapchain().set_screen_resolution(width, height);

    self.vulkan.recreate_swapchain();

    self
      .texture_handler
      .update_uniform_buffer(&self.vulkan.device(), width, height);
    self.model_handler.window_resized(width, height);
  }

  pub fn camera(&self) -> &Camera {
    self.model_handler.camera()
  }

  pub fn mut_camera(&mut self) -> &mut Camera {
    self.model_handler.mut_camera()
  }

  pub fn update_maat_settings(
    &mut self,
    window: &mut VkWindow,
    event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
    limit_fps: &mut bool,
    fps_limit: &mut f32,
    maat_settings: Vec<MaatSetting>,
  ) {
    for setting in maat_settings {
      match setting {
        MaatSetting::DrawMode(mode) => {
          self.model_handler.set_draw_mode(&self.vulkan, mode);
        }
        MaatSetting::MouseVisibility(visible) => {
          window.internal().set_cursor_visible(visible);
        }
        MaatSetting::CaptureMouse(capture) => {
          window.internal().set_cursor_grab(capture).ok(); // We are not too concerned if this isn't Ok()
        }
        MaatSetting::Window(fullscreen, borderless) => {
          if fullscreen {
            let monitor = event_loop.available_monitors().nth(0).unwrap();

            let fullscreen_mode = {
              if borderless {
                let video_mode = monitor.video_modes().nth(0).unwrap();
                // This is how we get the video resolutions
                for (i, video_mode) in monitor.video_modes().enumerate() {
                  println!("Video mode #{}: {}", i, video_mode);
                }
                Fullscreen::Exclusive(video_mode)
              } else {
                Fullscreen::Borderless(Some(monitor))
              }
            };

            window.internal().set_fullscreen(Some(fullscreen_mode));
          } else {
            let new_window = winit::window::Window::new(&event_loop).unwrap();
            window.replace_window(new_window);
            self.replace_window(window);
          }
        }
        MaatSetting::LimitFps(limit, should_limit) => {
          *limit_fps = should_limit;
          *fps_limit = (1.0 / limit).min(1.0);
        }
      }
    }
  }

  pub fn draw<T: Into<String>, L: Into<String>, S: Into<String>>(
    &mut self,
    texture_data: Vec<(Vec<f32>, T, Option<L>)>,
    model_data: Vec<(Vec<f32>, S)>,
  ) {
    if self.model_handler.mut_camera().is_updated() {
      self
        .model_handler
        .update_uniform_buffer(self.vulkan.device());
    }

    if let Some(present_index) = self.vulkan.start_render() {
      self.vulkan.begin_renderpass_model(present_index);
      for (data, model) in model_data {
        self
          .model_handler
          .draw(&mut self.vulkan, data, &model.into());
      }

      //self.model_handler.draw_instanced_models(&mut self.vulkan);

      self.vulkan.end_renderpass();
      self.vulkan.begin_renderpass_texture(present_index);

      let mut text_count = 0;

      for (data, texture, some_text) in texture_data {
        if let Some(text) = some_text {
          self
            .texture_handler
            .add_text_data(&mut text_count, data, &text.into(), &texture.into());
        } else {
          self
            .texture_handler
            .draw(&mut self.vulkan, data, &texture.into());
        }
      }

      self
        .texture_handler
        .draw_instanced_text(&mut self.vulkan, text_count);

      self.vulkan.end_renderpass();
      self.vulkan.end_render(present_index);
    }
  }

  pub fn update_animations(&mut self, delta_time: f32) {
    self
      .model_handler
      .update_animations(&mut self.vulkan, delta_time);
  }

  pub fn destroy(&mut self) {
    unsafe {
      self.vulkan.device().internal().device_wait_idle().unwrap();
    }

    // self.texture_handler.destroy(&mut self.vulkan);
    //    self.model_handler.destroy(&mut self.vulkan);

    //self.compute_descriptor_sets.destroy(self.vulkan.device());
    //self.compute_shader.destroy(self.vulkan.device());

    //unsafe {
    //  self
    //    .vulkan
    //    .device()
    //    .destroy_descriptor_pool(self.compute_descriptor_pool, None);
    //}
  }

  pub fn run<T, L, S, V>(
    mut vulkan: MaatGraphics,
    mut window: VkWindow,
    event_loop: EventLoop<()>,
    mut callback: T,
  ) -> !
  where
    T: 'static + FnMut(MaatEvent<L, S, V>),
    L: Into<String>,
    S: Into<String>,
    V: Into<String>,
  {
    let mut device_keys = Vec::new();
    let mut software_keys = Vec::new();

    let mut _delta_time = 0.0;
    let mut last_time = Instant::now();

    let mut total_delta_time = 0.0;
    let mut total_animation_delta_time = 0.0;

    let mut window_dimensions = [1280.0, 720.0];

    let mut limit_fps = false;
    let mut fps_limit = 1.0 / 144.0;
    let mut total_frame_limit_time = 0.0;

    event_loop.run(move |event, event_loop, control_flow| {
      *control_flow = ControlFlow::Poll;

      _delta_time = last_time.elapsed().subsec_nanos() as f32 / 1000000000.0 as f32;
      last_time = Instant::now();
      total_delta_time += _delta_time as f32;
      total_frame_limit_time += _delta_time as f32;
      total_animation_delta_time += _delta_time as f32;

      let mut should_exit = false;
      callback(MaatEvent::Update(
        &device_keys,
        &software_keys,
        vulkan.mut_camera(),
        _delta_time,
        &mut should_exit,
      ));

      if should_exit {
        *control_flow = ControlFlow::Exit;
      }

      if total_delta_time >= 0.05 {
        total_delta_time = DELTA_STEP;
      }

      if total_delta_time > DELTA_STEP {
        let delta_steps = ((total_delta_time / DELTA_STEP).floor() as usize).min(5);

        for _ in 0..delta_steps {
          callback(MaatEvent::FixedUpdate(
            &device_keys,
            &software_keys,
            vulkan.mut_camera(),
            DELTA_STEP,
          ));
          total_delta_time -= DELTA_STEP;
        }
      }

      if total_animation_delta_time > ANIMATION_DELTA_STEP {
        let delta_steps =
          ((total_animation_delta_time / ANIMATION_DELTA_STEP).floor() as usize).max(5);
        for _ in 0..delta_steps {
          vulkan.update_animations(ANIMATION_DELTA_STEP);
          total_animation_delta_time -= ANIMATION_DELTA_STEP;
        }
      }

      let mut texture_data = Vec::new();
      let mut model_data = Vec::new();
      let mut maat_settings_data = Vec::new();

      let mut should_redraw = !limit_fps;
      if total_frame_limit_time > fps_limit {
        total_frame_limit_time -= fps_limit;
        should_redraw = true;
      }

      if should_redraw {
        callback(MaatEvent::UpdateMaatSettings(
          &mut device_keys,
          &mut maat_settings_data,
        ));

        vulkan.update_maat_settings(
          &mut window,
          event_loop,
          &mut limit_fps,
          &mut fps_limit,
          maat_settings_data,
        );

        window.internal().request_redraw();
      }

      if let Some(controllers) = &mut vulkan.gamepads {
        if let Some(gamepad_id) = &vulkan.active_controller {
          if let Some(event) = controllers.next_event() {
            match event {
              GpEvent { id, event, .. } => {
                if id == *gamepad_id {
                  match event {
                    EventType::ButtonPressed(button, _) => {
                      if let Some(input) = ControllerInput::from_button(button) {
                        callback(MaatEvent::GamepadButton(input, true));
                      }
                    }
                    EventType::ButtonReleased(button, _) => {
                      if let Some(input) = ControllerInput::from_button(button) {
                        callback(MaatEvent::GamepadButton(input, false));
                      }
                    }
                    EventType::AxisChanged(axis, value, _) => {
                      if let Some(axis) = AxisInput::from_axis(axis, value) {
                        callback(MaatEvent::GamepadAxis(axis));
                      }
                    }
                    _ => {}
                  }
                }
              }
            }
          }
        }
      }

      match event {
        Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => {
            *control_flow = ControlFlow::Exit;
          }
          //WindowEvent::KeyboardInput {
          //  input:
          //    KeyboardInput {
          //      virtual_keycode: Some(VirtualKeyCode::Escape),
          //      ..
          //    },
          //  ..
          //} => *control_flow = ControlFlow::Exit,
          WindowEvent::Resized(dimensions) => {
            vulkan.recreate_swapchain(dimensions.width, dimensions.height);
            callback(MaatEvent::Resized(dimensions.width, dimensions.height));
            window_dimensions[0] = dimensions.width as f64;
            window_dimensions[1] = dimensions.height as f64;
            //window.internal().request_redraw();
          }
          WindowEvent::KeyboardInput { input, .. } => {
            let key_code = input.scancode;
            software_keys.push(key_code);
          }

          WindowEvent::CursorMoved { position, .. } => {
            callback(MaatEvent::MouseMoved(
              position.x,
              window_dimensions[1] - position.y,
            ));
          }
          // TODO:
          WindowEvent::MouseInput { state, button, .. } => {
            let state = match state {
              ElementState::Pressed => true,
              ElementState::Released => false,
            };

            if let Some(button) = match button {
              WMouseButton::Left => Some(MouseInput::Left(state)),
              WMouseButton::Right => Some(MouseInput::Right(state)),
              WMouseButton::Middle => Some(MouseInput::Middle(state)),
              WMouseButton::Other(_id) => None,
            } {
              callback(MaatEvent::MouseButton(button));
            }
          }
          window_event => {
            callback(MaatEvent::UnhandledWindowEvent(window_event));
          }
        },
        Event::DeviceEvent { event, .. } => match event {
          DeviceEvent::MouseMotion { delta: (mx, my) } => {
            callback(MaatEvent::MouseDelta(mx, my, vulkan.mut_camera()));
          }
          DeviceEvent::MouseWheel { delta } => match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => {
              #[cfg(target_os = "windows")]
              let y = -y;

              callback(MaatEvent::ScrollDelta(x, y, vulkan.mut_camera()));
            }
            _ => {}
          },
          DeviceEvent::Key(key) => match key.state {
            ElementState::Pressed => {
              if let Some(key_code) = key.virtual_keycode {
                if !device_keys.contains(&key_code) {
                  device_keys.push(key_code);
                }
              }
            }
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
          },
          device_event => {
            callback(MaatEvent::UnhandledDeviceEvent(device_event));
          }
        },
        //Event::MainEventsCleared => {
        Event::RedrawRequested(_id) => {
          callback(MaatEvent::Draw(&mut texture_data, &mut model_data));
          vulkan.draw(texture_data, model_data);
        }
        Event::LoopDestroyed => {
          vulkan.destroy();
        }
        _unhandled_event => {}
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn length() {
    let vec3 = [1.0, 0.0, 0.0];
    let length = Math::vec3_mag(vec3);

    assert_eq!(length, 1.0);
  }

  #[test]
  fn dot_product() {
    let vec3_0 = [1.0, 0.0, 0.0];
    let vec3_1 = [1.0, 0.0, 0.0];

    let vec4_0 = [1.0, 0.0, 0.0, 0.0];
    let vec4_1 = [1.0, 0.0, 0.0, 0.0];

    let dot3 = Math::vec3_dot(vec3_0, vec3_1);
    let dot4 = Math::vec4_dot(vec4_0, vec4_1);

    assert_eq!(dot3, 1.0);
    assert_eq!(dot4, 1.0);
  }

  #[test]
  fn cross() {
    let cross_1 = Math::vec3_cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let cross_2 = Math::vec3_cross([0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);

    assert_eq!(cross_1, [0.0, 0.0, 1.0]);
    assert_eq!(cross_2, [0.0, 0.0, -1.0]);
  }

  #[test]
  fn normalise() {
    let vec3_normalise_1 = Math::vec3_normalise([1.0, 0.0, 0.0]);
    let vec3_normalise_2 = Math::vec3_normalise([2.0, 0.0, 0.0]);

    let vec4_normalise_1 = Math::vec4_normalise([1.0, 0.0, 0.0, 0.0]);
    let vec4_normalise_2 = Math::vec4_normalise([2.0, 0.0, 0.0, 0.0]);

    assert_eq!(vec3_normalise_1, [1.0, 0.0, 0.0]);
    assert_eq!(vec3_normalise_2, [1.0, 0.0, 0.0]);

    assert_eq!(vec4_normalise_1, [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(vec4_normalise_2, [1.0, 0.0, 0.0, 0.0]);
  }

  #[test]
  fn equals() {
    let vec3 = [1.0, 0.0, 0.0];
    let vec4 = [1.0, 0.0, 0.0, 0.0];

    assert_eq!(Math::vec3_equals(vec3, vec3), true);
    assert_eq!(Math::vec4_equals(vec4, vec4), true);
  }

  #[test]
  fn operators() {
    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 5.0, 6.0];

    let c = Math::vec3_add(a, b);
    assert_eq!(c, [5.0, 7.0, 9.0]);

    let d = Math::vec3_minus(b, a);
    assert_eq!(d, [3.0, 3.0, 3.0]);

    let e = Math::vec3_mul(a, b);
    assert_eq!(e, [4.0, 10.0, 18.0]);

    let f = Math::vec3_div(b, a);
    assert_eq!(f, [4.0, 2.5, 2.0]);

    let g = Math::vec3_mul_f32(a, 2.0);
    assert_eq!(g, [2.0, 4.0, 6.0]);

    let h = Math::vec3_div_f32(b, 2.0);
    assert_eq!(h, [2.0, 2.5, 3.0]);

    let a = [1.0, 2.0, 3.0, 4.0];
    let b = [5.0, 6.0, 7.0, 8.0];

    let c = Math::vec4_add(a, b);
    assert_eq!(c, [6.0, 8.0, 10.0, 12.0]);

    let d = Math::vec4_minus(b, a);
    assert_eq!(d, [4.0, 4.0, 4.0, 4.0]);

    let e = Math::vec4_mul(a, b);
    assert_eq!(e, [5.0, 12.0, 21.0, 32.0]);

    let f = Math::vec4_div(b, a);
    assert_eq!(f, [5.0, 3.0, 7.0 / 3.0, 2.0]);

    let g = Math::vec4_mul_f32(a, 2.0);
    assert_eq!(g, [2.0, 4.0, 6.0, 8.0]);

    let h = Math::vec4_div_f32(b, 2.0);
    assert_eq!(h, [2.5, 3.0, 3.5, 4.0]);
  }

  #[test]
  fn mat4_multiply() {
    let m: [f32; 16] = [
      0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    ];
    let n = Math::mat4_mul(m, m);
    /*let expected = [0.0, 1.0, 4.0, 9.0,
    16.0, 25.0, 36.0, 49.0,
    64.0, 81.0, 100.0, 121.0,
    144.0, 169.0, 196.0, 255.0];*/
    let expected = [
      56.0, 62.0, 68.0, 74.0, 152.0, 174.0, 196.0, 218.0, 248.0, 286.0, 324.0, 362.0, 344.0, 398.0,
      452.0, 506.0,
    ];

    assert_eq!(n, expected);
  }

  #[test]
  fn mat4_transpose() {
    let m: [f32; 16] = [
      0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    ];
    let t = Math::mat4_transpose(m);
    let expected = [
      0.0, 4.0, 8.0, 12.0, 1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0,
    ];

    assert_eq!(t, expected);
  }

  #[test]
  fn mat4_inverse() {
    let mut a = Math::mat4_identity();
    a[2] = 1.0;

    let b = Math::mat4_inverse(a);
    let i = Math::mat4_mul(a, b);

    assert_eq!(i, Math::mat4_identity());
  }

  #[test]
  fn mat4_scale() {
    let m = Math::mat4_identity();
    let v = [2.0, 2.0, 2.0];

    let s = Math::mat4_scale(m, v);
    let r = [
      2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    assert_eq!(s, r);
  }

  #[test]
  fn mat4_det() {
    let m: [f32; 16] = [
      0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    ];

    let det = Math::mat4_determinant(m);

    assert_eq!(det, 0.0);
  }

  /*
  #[test]
  fn mat4_rotate_eular() {
    let a = [1.0, 0.0, 0.0, 1.0];

    let r = Math::mat4_rotate_eular_axis(Math::mat4_identity(), (90.0f32).to_radians(), [0.0, 0.0, 1.0]);
    let b = Math::vec4_mul_mat4(a, r);

    assert_eq!(b, [0.0, 1.0, 0.0, 1.0]);
  }*/

  #[test]
  fn quat() {}
}
