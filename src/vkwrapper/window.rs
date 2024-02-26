use raw_window_handle;
use winit::{
  dpi::{LogicalSize, PhysicalSize},
  event_loop::EventLoop,
  window::{Window, WindowBuilder},
};

pub struct VkWindow {
  window: Window,
}

impl VkWindow {
  pub fn new(
    app_name: &str,
    window_width: u32,
    window_height: u32,
    event_loop: &EventLoop<()>,
    screen_resolution: &mut [u32; 2],
  ) -> VkWindow {
    let (logical_window_size, physical_window_size) = {
      let dpi = event_loop.primary_monitor().unwrap().scale_factor();
      let logical: LogicalSize<u32> = (window_width, window_height).into();
      let physical: PhysicalSize<u32> = logical.to_physical(dpi);

      (logical, physical)
    };

    *screen_resolution = [physical_window_size.width, physical_window_size.height];

    let window = WindowBuilder::new()
      .with_title(app_name)
      .with_inner_size(logical_window_size)
      .build(&event_loop)
      .expect("Failed to create window");

    VkWindow { window }
  }

  pub fn internal(&self) -> &Window {
    &self.window
  }

  pub fn replace_window(&mut self, window: Window) {
    self.window = window;
  }
}
