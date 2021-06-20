use ash::vk;

pub struct Scissors {
  scissors: Vec<vk::Rect2D>,
}

impl Scissors {
  pub fn new() -> Scissors {
    
    Scissors {
      scissors: Vec::new(),
    }
  }
  
  pub fn add_scissor(mut self, offset_x: i32, offset_y: i32, extent_width: u32, extent_height: u32) -> Scissors {
    let offset: vk::Offset2D = vk::Offset2D { x: offset_x, y: offset_y };
    let extent: vk::Extent2D = vk::Extent2D { width: extent_width, height: extent_height };
    
    self.scissors.push(vk::Rect2D { offset, extent });
    
    self
  }
  
  pub fn build(&self) -> Vec<vk::Rect2D> {
    (*self.scissors).to_vec()
  }
}
