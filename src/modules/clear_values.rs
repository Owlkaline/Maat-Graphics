use ash::vk;

enum ClearType {
  Colour(f32, f32, f32, f32),
  Depth(f32, u32),
}

impl ClearType {
  pub fn into(&self) -> vk::ClearValue {
    match self {
      ClearType::Colour(r, g, b, a) => {
        vk::ClearValue {
          color: vk::ClearColorValue {
            float32: [*r, *g, *b, *a],
          },
        }
      },
      ClearType::Depth(depth, stencil) => {
        vk::ClearValue {
          depth_stencil: vk::ClearDepthStencilValue {
            depth: *depth,
            stencil: *stencil,
          },
        }
      },
    }
  }
}

pub struct ClearValues {
  clear_colours: Vec<ClearType>,
}

impl ClearValues {
  pub fn new() -> ClearValues {
    ClearValues {
      clear_colours: Vec::new(),
    }
  }
  
  pub fn add_colour(mut self, r: f32, g: f32, b: f32, a: f32) -> ClearValues {
    self.clear_colours.push(ClearType::Colour(r, g, b, a));
    self
  }
  
  pub fn add_depth(mut self, depth: f32, stencil: u32) -> ClearValues {
    self.clear_colours.push(ClearType::Depth(depth, stencil));
    self
  }
  
  pub fn build(&self) -> Vec<vk::ClearValue> {
   let mut clear_values = Vec::new();
   for clear in &self.clear_colours {
     clear_values.push(clear.into());
   }
   
   clear_values
  }
}
