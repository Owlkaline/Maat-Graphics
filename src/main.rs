#[macro_use]
extern crate lazy_static;
extern crate winit;
extern crate shared_library;
extern crate vk_sys as vk;
extern crate image;
extern crate cgmath;
extern crate libc;

use modules::Vulkan;

mod loader;
mod modules;
mod ownage;

const ENGINE_VERSION: u32 = (0 as u32) << 22 | (5 as u32) << 12 | (0 as u32);

fn main() {
  let app_name = "TestApplication".to_string();
  let mut vulkan = Vulkan::new(app_name, (0 as u32) << 22 | (0 as u32) << 12 | (0 as u32), 1280.0, 720.0, true);
  vulkan.setup();
  
  let mut loop_num = 0;
  
  let mut resized = false;
  let mut done = false;
  loop {
    vulkan.build();
    vulkan.draw();
    
    vulkan.get_events().poll_events(|ev| {
      match ev {
        winit::Event::WindowEvent{ event, .. } => {
          match event {
            winit::WindowEvent::Resized(_new_size) => {
              resized = true;
            },
            winit::WindowEvent::CloseRequested => {
              done = true;
            },
            winit::WindowEvent::HiDpiFactorChanged(new_dpi) => {
              println!("Dpi Changed: {}", new_dpi);
            //  dpi = new_dpi as f32;
            },
            _ => { }
          }
        },
        _ => {},
      }
    });
    
    if done {
      break;
    }
    
    println!("loop: {}", loop_num);
    loop_num += 1;
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_test() {
    assert_eq!(4, 2+2);
  }
}
