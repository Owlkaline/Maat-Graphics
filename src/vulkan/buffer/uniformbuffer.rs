use crate::vulkan::Instance;
use crate::vulkan::Device;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::buffer::BufferUsage;

use std::mem;
use std::sync::Arc;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;

#[derive(Clone)]
enum Ty {
  Float,
  Vector2,
  Vector3,
  Vector4,
  Mat4,
}

impl Ty {
  pub fn _size(&self) -> vk::DeviceSize {
    let mut size = mem::size_of::<f32>();
    match self {
      Ty::Float => {
        size *= 1;
      },
      Ty::Vector2 => {
        size *= 2;
      },
      Ty::Vector3 => {
        size *= 3;
      },
      Ty::Vector4 => {
        size *= 4;
      },
      Ty::Mat4 => {
        size *= 16;
      }
    }
    size as vk::DeviceSize
  }
}

#[derive(Clone)]
pub struct UniformData {
  data: Vec<f32>,
}

impl UniformData {
  pub fn new() -> UniformData {
    UniformData {
      data: Vec::new(),
    }
  }
  
  pub fn with_capacity(capacity: usize) -> UniformData {
    UniformData {
      data: Vec::with_capacity(capacity),
    }
  }
  
  pub fn empty(&mut self) {
    self.data.clear();
  }
  
  pub fn add_float(mut self, value: f32) -> UniformData {
    self.data.push(value);
    self
  }
  
  pub fn add_vector2(mut self, values: Vector2<f32>) -> UniformData {
    self.data.push(values.x);
    self.data.push(values.y);
    self
  }
  
  pub fn add_vector3(mut self, values: Vector3<f32>) -> UniformData {
    self.data.push(values.x);
    self.data.push(values.y);
    self.data.push(values.z);
    self
  }
  
  pub fn add_vector4(mut self, values: Vector4<f32>) -> UniformData {
    self.data.push(values.x);
    self.data.push(values.y);
    self.data.push(values.z);
    self.data.push(values.w);
    self
  }
  
  pub fn add_matrix4(mut self, values: Matrix4<f32>) -> UniformData {
    self.data.push(values.x.x);
    self.data.push(values.x.y);
    self.data.push(values.x.z);
    self.data.push(values.x.w);
    self.data.push(values.y.x);
    self.data.push(values.y.y);
    self.data.push(values.y.z);
    self.data.push(values.y.w);
    self.data.push(values.z.x);
    self.data.push(values.z.y);
    self.data.push(values.z.z);
    self.data.push(values.z.w);
    self.data.push(values.w.x);
    self.data.push(values.w.y);
    self.data.push(values.w.z);
    self.data.push(values.w.w);
    self
  }
  
  pub fn size_non_aligned(&self) -> vk::DeviceSize {
    let size = self.data.len();
    
    (mem::size_of::<f32>() * size) as vk::DeviceSize
  }
  
  pub fn build_non_aligned(&mut self) -> Vec<f32> {
    self.data.clone()
  }
  
  pub fn size(&self, device: Arc<Device>) -> vk::DeviceSize {
    let mut size = self.data.len();
    let alignment = device.get_non_coherent_atom_size();
    if size as u64%alignment != 0 {
      size += 1;
    }
    
    (mem::size_of::<f32>() * size) as vk::DeviceSize
  }
  
  pub fn build(&mut self, device: Arc<Device>) -> Vec<f32> {
    let alignment = device.get_non_coherent_atom_size();
    while self.data.len() as u64%alignment != 0 {
      self.data.push(0.0);
    }
    
    self.data.clone()
  }
  
  pub fn build_uniform_data(&mut self, device: Arc<Device>) -> Vec<f32> {
    let alignment = device.get_min_uniformbuffer_offset_alignment();
    
    while self.data.len() as u64%alignment != 0 {
      self.data.push(0.0);
    }
    
    self.data.clone()
  }
}
/*
pub struct UniformBuffer {
  uniform_ty: Vec<Ty>,
  buffer: Buffer<f32>,
}

impl UniformBuffer {
  pub fn info(&self) -> vk::DescriptorBufferInfo {
    let mut size: vk::DeviceSize = 0;
    for ty in &self.uniform_ty {
      size += ty.size();
    }
    
    vk::DescriptorBufferInfo {
      buffer: *self.buffer.internal_object(),
      offset: 0,
      range: size,
    }
  }
}*/

pub struct UniformBufferBuilder {
  uniform_ty: Vec<Ty>,
  binding: u32,
}

impl UniformBufferBuilder {
  pub fn new() -> UniformBufferBuilder {
    UniformBufferBuilder {
      uniform_ty: Vec::new(),
      binding: 0,
    }
  }
  
  pub fn get_binding(&self) -> u32 {
    self.binding
  }
  
  pub fn set_binding(mut self, binding: u32) -> UniformBufferBuilder {
    self.binding = binding;
    self
  }
  
  pub fn add_float(mut self) -> UniformBufferBuilder {
    self.uniform_ty.push(Ty::Float);
    self
  }
  
  pub fn add_vector2(mut self) -> UniformBufferBuilder {
    self.uniform_ty.push(Ty::Vector2);
    self
  }
  
  pub fn add_vector3(mut self) -> UniformBufferBuilder {
    self.uniform_ty.push(Ty::Vector3);
    self
  }
  
  pub fn add_vector4(mut self) -> UniformBufferBuilder {
    self.uniform_ty.push(Ty::Vector4);
    self
  }
  
  pub fn add_matrix4(mut self) -> UniformBufferBuilder {
    self.uniform_ty.push(Ty::Mat4);
    self
  }
  
  pub fn build(&self, instance: Arc<Instance>, device: Arc<Device>, num_sets: u32) -> Buffer<f32> {
    let usage = BufferUsage::uniform_buffer();
    let uniform_buffer = self.build_buffer(Arc::clone(&instance), Arc::clone(&device), usage, num_sets);
    uniform_buffer
  }
  
  fn build_buffer(&self, instance: Arc<Instance>, device: Arc<Device>, usage: BufferUsage, num_sets: u32) -> Buffer<f32> {
    let mut data: Vec<f32> = Vec::new();
    for ty in &self.uniform_ty {
      match ty {
        Ty::Float => {
          data.push(0.0);
        },
        Ty::Vector2 => {
          data.push(0.0);
          data.push(0.0);
        },
        Ty::Vector3 => {
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
        },
        Ty::Vector4 => {
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
        },
        Ty::Mat4 => {
          data.push(1.0);
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
          data.push(1.0);
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
          data.push(1.0);
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
          data.push(0.0);
          data.push(1.0);
        }
      }
    }
    
    let alignment = device.get_non_coherent_atom_size();
    while data.len() as u64% alignment != 0 {
      data.push(0.0);
    }
    
    let uniform_buffer: Buffer<f32> = Buffer::cpu_buffer_with_data(instance, device, usage, num_sets, data);
    uniform_buffer
  }
}
