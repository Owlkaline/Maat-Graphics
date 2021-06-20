use ash::vk;
use ash::version::DeviceV1_0;

use crate::modules::VkDevice;

pub struct Sampler {
  sampler: vk::Sampler,
}

impl Sampler {
  pub fn builder() -> SamplerBuilder {
    SamplerBuilder::new()
  }
  
  pub fn new(device: &VkDevice, create_info: vk::SamplerCreateInfo) -> Sampler {
    let sampler = unsafe {
      device.internal().create_sampler(&create_info, None).unwrap()
    };
    
    Sampler {
      sampler,
    }
  }
  
  pub fn internal(&self) -> vk::Sampler {
    self.sampler
  }
}

pub struct SamplerBuilder {
  mag_filter: vk::Filter,
  min_filter: vk::Filter,
  mipmap_mode: vk::SamplerMipmapMode,
  address_mode: vk::SamplerAddressMode,
  max_anisotropy: f32,
  border_colour: vk::BorderColor,
  compare_op: vk::CompareOp,
}

impl SamplerBuilder {
  pub fn new() -> SamplerBuilder {
    
    let mag_filter: vk::Filter = Default::default();
    let min_filter: vk::Filter = Default::default();
    let mipmap_mode: vk::SamplerMipmapMode = Default::default();
    let address_mode: vk::SamplerAddressMode = Default::default();
    let max_anisotropy: f32 = Default::default();
    let border_colour: vk::BorderColor = Default::default();
    let compare_op: vk::CompareOp = Default::default();
    
    SamplerBuilder {
      mag_filter,
      min_filter,
      mipmap_mode,
      address_mode,
      max_anisotropy,
      border_colour,
      compare_op,
    }
  }
  
  pub fn address_mode_mirrored_repeat(mut self) -> SamplerBuilder {
    self.address_mode = vk::SamplerAddressMode::MIRRORED_REPEAT;
    self
  }
  
  pub fn address_mode_repeat(mut self) -> SamplerBuilder {
    self.address_mode = vk::SamplerAddressMode::REPEAT;
    self
  }
  
  pub fn address_mode_clamp_to_edge(mut self) -> SamplerBuilder {
    self.address_mode = vk::SamplerAddressMode::CLAMP_TO_EDGE;
    self
  }
  
  pub fn address_mode_clamp_to_border(mut self) -> SamplerBuilder {
    self.address_mode = vk::SamplerAddressMode::CLAMP_TO_BORDER;
    self
  }
  
  pub fn min_filter_nearest(mut self) -> SamplerBuilder {
    self.min_filter = vk::Filter::NEAREST;
    self
  }
  
  pub fn min_filter_linear(mut self) -> SamplerBuilder {
    self.min_filter = vk::Filter::LINEAR;
    self
  }
  
  pub fn mag_filter_nearest(mut self) -> SamplerBuilder {
    self.mag_filter = vk::Filter::NEAREST;
    self
  }
  
  pub fn mag_filter_linear(mut self) -> SamplerBuilder {
    self.mag_filter = vk::Filter::LINEAR;
    self
  }
  
  pub fn mipmap_mode_nearest(mut self) -> SamplerBuilder {
    self.mipmap_mode = vk::SamplerMipmapMode::NEAREST;
    self
  }
  
  pub fn mipmap_mode_linear(mut self) -> SamplerBuilder {
    self.mipmap_mode = vk::SamplerMipmapMode::LINEAR;
    self
  }
  
  pub fn border_colour_float_transparent_black(mut self) -> SamplerBuilder {
    self.border_colour = vk::BorderColor::FLOAT_TRANSPARENT_BLACK;
    self
  }
  
  pub fn border_colour_int_transparent_black(mut self) -> SamplerBuilder {
    self.border_colour = vk::BorderColor::INT_TRANSPARENT_BLACK;
    self
  }
  
  pub fn border_colour_float_opaque_black(mut self) -> SamplerBuilder {
    self.border_colour = vk::BorderColor::FLOAT_OPAQUE_BLACK;
    self
  }
  
  pub fn border_colour_int_opaque_black(mut self) -> SamplerBuilder {
    self.border_colour = vk::BorderColor::INT_OPAQUE_BLACK;
    self
  }
  
  pub fn border_colour_float_opaque_white(mut self) -> SamplerBuilder {
    self.border_colour = vk::BorderColor::FLOAT_OPAQUE_WHITE;
    self
  }
  
  pub fn border_colour_int_opaque_white(mut self) -> SamplerBuilder {
    self.border_colour = vk::BorderColor::INT_OPAQUE_WHITE;
    self
  }
  
  pub fn compare_op_never(mut self) -> SamplerBuilder {
    self.compare_op = vk::CompareOp::NEVER;
    self
  }
  
  pub fn compare_op_less(mut self) -> SamplerBuilder {
    self.compare_op = vk::CompareOp::LESS;
    self
  }
  
  pub fn compare_op_equal(mut self) -> SamplerBuilder {
    self.compare_op = vk::CompareOp::EQUAL;
    self
  }
  
  pub fn compare_op_less_or_equal(mut self) -> SamplerBuilder {
    self.compare_op = vk::CompareOp::LESS_OR_EQUAL;
    self
  }
  
  pub fn compare_op_greater(mut self) -> SamplerBuilder {
    self.compare_op = vk::CompareOp::GREATER;
    self
  }
  
  pub fn compare_op_greater_or_equal(mut self) -> SamplerBuilder {
    self.compare_op = vk::CompareOp::GREATER_OR_EQUAL;
    self
  }
  
  pub fn compare_op_always(mut self) -> SamplerBuilder {
    self.compare_op = vk::CompareOp::ALWAYS;
    self
  }
  
  pub fn build(&self, device: &VkDevice) -> Sampler {
    let sampler_info = vk::SamplerCreateInfo {
      mag_filter: self.mag_filter,
      min_filter: self.min_filter,
      mipmap_mode: self.mipmap_mode,
      address_mode_u: self.address_mode,
      address_mode_v: self.address_mode,
      address_mode_w: self.address_mode,
      max_anisotropy: self.max_anisotropy,
      border_color: self.border_colour,
      compare_op: self.compare_op,
      ..Default::default()
    };
    
    Sampler::new(device, sampler_info)
  }
}
