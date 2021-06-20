use ash::vk;
use ash::version::DeviceV1_0;

use image;

use crate::modules::{VkDevice, Memory};

pub struct Buffer<T: Sized + Copy> {
  buffer: vk::Buffer,
  memory: Memory<T>,
  data: Vec<T>
}

impl<T: Sized + Copy> Buffer<T> {
  pub fn builder() -> BufferBuilder<T> {
    BufferBuilder::new()
  }
  
  pub fn new_generic(device: &VkDevice, data: Vec<T>, memory_properties: vk::MemoryPropertyFlags, 
                     usage: vk::BufferUsageFlags) -> Buffer<T> {
    let buffer = Buffer::create_buffer(device, usage, &data);
    let memory = Memory::new_buffer_memory(device, &buffer, memory_properties, &data);
    
    Buffer {
      buffer,
      memory,
      data,
    }
  }
  
  pub fn retrieve_buffer_data(&self, device: &VkDevice) -> Vec<T> {
    self.memory.data_from_memory(device, self.data.len())
  }
  
  pub fn new_index(device: &VkDevice, data: Vec<T>) -> Buffer<T> {
    let memory_properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let usage = vk::BufferUsageFlags::INDEX_BUFFER;
    
    Buffer::new_generic(device, data, memory_properties, usage)
  }
  
  pub fn new_vertex(device: &VkDevice, data: Vec<T>) -> Buffer<T> {
    let memory_properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let usage = vk::BufferUsageFlags::VERTEX_BUFFER;
    
    Buffer::new_generic(device, data, memory_properties, usage)
  }
  
  pub fn new_image(device: &VkDevice, data: Vec<u8>) -> Buffer<u8> {
    let memory_properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let usage = vk::BufferUsageFlags::TRANSFER_SRC;
    
    Buffer::new_generic(device, data, memory_properties, usage)
  }
  
  pub fn new_uniform_buffer(device: &VkDevice, data: &Vec<T>) -> Buffer<T> {
    let memory_properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let usage = vk::BufferUsageFlags::UNIFORM_BUFFER;
    
    Buffer::new_generic(device, data.to_vec(), memory_properties, usage)
  }
  
  fn create_buffer(device: &VkDevice, usage: vk::BufferUsageFlags, data: &Vec<T>) -> vk::Buffer {
    let buffer_info = vk::BufferCreateInfo::builder() 
                          .size(std::mem::size_of::<T>() as u64 * (data.len() as u64))
                          .usage(usage)
                          .sharing_mode(vk::SharingMode::EXCLUSIVE);
    
    unsafe {
      device.internal().create_buffer(&buffer_info, None).unwrap()
    }
  }
  
  pub fn destroy(&self, device: &VkDevice) {
    unsafe {
      device.internal().destroy_buffer(self.buffer, None);
    }
    self.memory.destroy(device);
  }
  
  pub fn internal(&self) -> &vk::Buffer {
    &self.buffer
  }
  
  pub fn data(&self) -> &Vec<T> {
    &self.data
  }
}

pub struct BufferBuilder<T: Sized + Copy> {
  usage: vk::BufferUsageFlags,
  memory_properties: vk::MemoryPropertyFlags,
  data: Vec<T>,
}

impl<T: Sized + Copy> BufferBuilder<T> {
  pub fn new() -> BufferBuilder<T> {
    BufferBuilder {
      usage: Default::default(),
      memory_properties: Default::default(),
      data: Vec::new(),
    }
  }
  
  pub fn data(mut self, data: Vec<T>) -> BufferBuilder<T> {
    self.data = data;
    self
  }
  
  pub fn usage_index_buffer(mut self) -> BufferBuilder<T> {
    self.usage = vk::BufferUsageFlags::INDEX_BUFFER;
    self
  }
  
  pub fn usage_vertex_buffer(mut self) -> BufferBuilder<T> {
    self.usage = vk::BufferUsageFlags::VERTEX_BUFFER;
    self
  }
  
  pub fn usage_transfer_src(mut self) -> BufferBuilder<T> {
    self.usage = vk::BufferUsageFlags::TRANSFER_SRC;
    self
  }
  
  pub fn usage_transfer_dst(mut self) -> BufferBuilder<T> {
    self.usage = vk::BufferUsageFlags::TRANSFER_DST;
    self
  }
  
  pub fn usage_uniform_buffer(mut self) -> BufferBuilder<T> {
    self.usage = vk::BufferUsageFlags::UNIFORM_BUFFER;
    self
  }
  
  pub fn usage_transfer_src_dst(mut self) -> BufferBuilder<T> {
    self.usage = vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST;
    self
  }
  
  pub fn usage_transfer_storage_src_dst(mut self) -> BufferBuilder<T> {
    self.usage = vk::BufferUsageFlags::TRANSFER_SRC | 
                 vk::BufferUsageFlags::TRANSFER_DST | 
                 vk::BufferUsageFlags::STORAGE_BUFFER;
    self
  }
  
  pub fn memory_properties_host_visible_coherent(mut self) -> BufferBuilder<T> {
    self.memory_properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    self
  }
  
  pub fn memory_properties_device_local(mut self) -> BufferBuilder<T> {
    self.memory_properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    self
  }
  
  pub fn build(&self, device: &VkDevice) -> Buffer<T> {
    Buffer::new_generic(device,
                        self.data.to_vec(),
                        self.memory_properties, 
                        self.usage)
  }
}
