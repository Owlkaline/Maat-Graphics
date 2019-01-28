use vk;

use crate::vulkan::vkenums::{VkBool, Filter, MipmapMode, AddressMode, BorderColour, CompareOp};
use crate::vulkan::Device;
use crate::vulkan::check_errors;

use std::mem;
use std::ptr;
use std::sync::Arc;

#[derive(Clone)]
pub struct Sampler {
  sampler: vk::Sampler,
}

impl Sampler {
  pub fn internal_object(&self) -> vk::Sampler {
    self.sampler
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      vk.DestroySampler(*device, self.sampler, ptr::null())
    }
  }
}

pub struct SamplerBuilder {
  mag_filter: Filter,
  min_filter: Filter,
  mipmap_mode: MipmapMode,
  address_mode_u: AddressMode,
  address_mode_v: AddressMode,
  address_mode_w: AddressMode,
  mip_lod_bias: f32,
  anisotropy: VkBool,
  max_anisotropy: f32,
  compare: VkBool,
  compare_op: CompareOp,
  min_lod: f32,
  max_lod: f32,
  border_colour: BorderColour,
  unnormalized_coordinates: VkBool,
}

impl SamplerBuilder {
  pub fn new() -> SamplerBuilder {
    SamplerBuilder {
      mag_filter: Filter::Nearest,
      min_filter: Filter::Nearest,
      mipmap_mode: MipmapMode::Linear,
      address_mode_u: AddressMode::ClampToEdge,
      address_mode_v: AddressMode::ClampToEdge,
      address_mode_w: AddressMode::ClampToEdge,
      mip_lod_bias: 0.0,
      anisotropy: VkBool::True,
      max_anisotropy: 16.0,
      compare: VkBool::False,
      compare_op: CompareOp::Always,
      min_lod: 0.0,
      max_lod: 0.0,
      border_colour: BorderColour::IntOpaqueBlack,
      unnormalized_coordinates: VkBool::False,
    }
  }
  
  pub fn min_filter(mut self, filter: Filter) -> SamplerBuilder {
    self.min_filter = filter;
    self
  }
  
  pub fn mag_filter(mut self, filter: Filter) -> SamplerBuilder {
    self.mag_filter = filter;
    self
  }
  
  pub fn mipmap_mode(mut self, mode: MipmapMode) -> SamplerBuilder {
    self.mipmap_mode = mode;
    self
  }
  
  pub fn address_mode_u(mut self, mode: AddressMode) -> SamplerBuilder {
    self.address_mode_u = mode.clone();
    self
  }
  
  pub fn address_mode_v(mut self, mode: AddressMode) -> SamplerBuilder {
    self.address_mode_v = mode.clone();
    self
  }
  
  pub fn address_mode_w(mut self, mode: AddressMode) -> SamplerBuilder {
    self.address_mode_w = mode.clone();
    self
  }
  
  pub fn address_mode(mut self, mode: AddressMode) -> SamplerBuilder {
    self.address_mode_u = mode.clone();
    self.address_mode_v = mode.clone();
    self.address_mode_w = mode;
    self
  }
  
  pub fn anisotropy(mut self, enabled: VkBool) -> SamplerBuilder {
    self.anisotropy = enabled;
    self
  }
  
  pub fn max_anisotropy(mut self, max: f32) -> SamplerBuilder {
    self.max_anisotropy = max;
    self
  }
  
  pub fn build(&self, device: Arc<Device>) -> Sampler {
    let vk = device.pointers();
    let device = device.internal_object();
    
    let mut sampler: vk::Sampler = unsafe { mem::uninitialized() };
    
    let sampler_create_info = {
      vk::SamplerCreateInfo {
        sType: vk::STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        magFilter: self.mag_filter.to_bits(),
        minFilter: self.min_filter.to_bits(),
        mipmapMode: self.mipmap_mode.to_bits(),
        addressModeU: self.address_mode_u.to_bits(),
        addressModeV: self.address_mode_v.to_bits(),
        addressModeW: self.address_mode_w.to_bits(),
        mipLodBias: self.mip_lod_bias,
        anisotropyEnable: self.anisotropy.to_bits(),
        maxAnisotropy: self.max_anisotropy,
        compareEnable: self.compare.to_bits(),
        compareOp: self.compare_op.to_bits(),
        minLod: self.min_lod,
        maxLod: self.max_lod,
        borderColor: self.border_colour.to_bits(),
        unnormalizedCoordinates: self.unnormalized_coordinates.to_bits(),
      }
    };
    
    unsafe {
      check_errors(vk.CreateSampler(*device, &sampler_create_info, ptr::null(), &mut sampler));
    }
    
    Sampler::new_with_sampler(sampler)
  }
}

impl Sampler {
  pub fn new_with_sampler(sampler: vk::Sampler) -> Sampler {
    Sampler {
      sampler
    }
  }
  
  pub fn new_texture_sampler(device: Arc<Device>) -> Sampler {
    let vk = device.pointers();
    let device = device.internal_object();
    
    let mut sampler: vk::Sampler = unsafe { mem::uninitialized() };
    
    let mag_filter = vk::FILTER_NEAREST;
    let min_filter = vk::FILTER_NEAREST;
    let mipmap_mode = vk::SAMPLER_MIPMAP_MODE_LINEAR;
    let address_mode = vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    
    let sampler_create_info = {
      vk::SamplerCreateInfo {
        sType: vk::STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        magFilter: mag_filter,
        minFilter: min_filter,
        mipmapMode: mipmap_mode,
        addressModeU: address_mode,
        addressModeV: address_mode,
        addressModeW: address_mode,
        mipLodBias: 0.0,
        anisotropyEnable: vk::TRUE,
        maxAnisotropy: 16.0,
        compareEnable: vk::FALSE,
        compareOp: vk::COMPARE_OP_ALWAYS,
        minLod: 0.0,
        maxLod: 0.0,
        borderColor: vk::BORDER_COLOR_INT_OPAQUE_BLACK,
        unnormalizedCoordinates: vk::FALSE,
      }
    };
    
    unsafe {
      check_errors(vk.CreateSampler(*device, &sampler_create_info, ptr::null(), &mut sampler));
    }
    
    Sampler::new_with_sampler(sampler)
  }
}
