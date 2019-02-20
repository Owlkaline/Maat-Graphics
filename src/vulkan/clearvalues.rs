use vk;

use cgmath::Vector4;

pub enum ClearValue {
  ClearColour(Vector4<f32>),
  ClearDepthStencil((f32, u32)),
}

impl ClearValue {
  pub fn to_bits(&self) -> vk::ClearValue {
    match self {
      ClearValue::ClearColour(colour) => {
        vk::ClearValue {
          color: vk::ClearColorValue { float32: [colour.x, colour.y, colour.z, colour.w] }
        }
      },
      ClearValue::ClearDepthStencil((depth, stencil)) => {
        vk::ClearValue {
          depthStencil: vk::ClearDepthStencilValue { depth: *depth, stencil: *stencil }
        }
      },
    }
  }
}

pub struct ClearValues {
  clear_values: Vec<ClearValue>,
}

impl ClearValues {
  pub fn new() -> ClearValues {
    ClearValues {
      clear_values: Vec::new()
    }
  }
  
  pub fn get_len(&self) -> u32 {
    self.clear_values.len() as u32
  }
  
  pub fn add_colour(mut self, colour: Vector4<f32>) -> ClearValues {
            println!("{:?}", colour);
    self.clear_values.push(ClearValue::ClearColour(colour));
    self
  }
  
  pub fn add_depth(mut self, depth: f32, stencil: u32) -> ClearValues {
    self.clear_values.push(ClearValue::ClearDepthStencil((depth, stencil)));
    self
  }
  
  pub fn to_bits(&self) -> Vec<vk::ClearValue> {
    let mut values = Vec::with_capacity(self.clear_values.len());
    
    for value in &self.clear_values {
      values.push(value.to_bits());
    }
    
    values
  }
}
