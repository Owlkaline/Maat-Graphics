use ash::vk;

pub struct Viewport {
  x: f32,
  y: f32,
  width: f32,
  height: f32,
  min_depth: f32,
  max_depth: f32,
}

impl Viewport {
  pub fn new(x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32) -> Viewport {
    
    Viewport {
      x,
      y,
      width,
      height,
      min_depth,
      max_depth,
    }
  }
  
  pub fn build(&self) -> vk::Viewport {
    vk::Viewport {
      x: self.x,
      y: self.y,
      width: self.width,
      height: self.height,
      min_depth: self.min_depth,
      max_depth: self.max_depth,
    }
  }
}
