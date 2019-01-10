use vk;

use crate::vulkan::buffer::BufferUsage;
use crate::vulkan::Instance;
use crate::vulkan::Device;
use crate::vulkan::ownage::check_errors;

use libc::memcpy;

use std::mem;
use std::ptr;
use std::sync::Arc;

#[derive(Clone)]
pub struct Buffer<T: Clone> {
  buffer: Vec<vk::Buffer>,
  memory: Vec<vk::DeviceMemory>,
  usage: BufferUsage,
  data: Vec<T>,
}

impl<T: Clone> Buffer<T> {
  pub fn empty(instance: Arc<Instance>, device: Arc<Device>, usage: BufferUsage, num_sets: u32) -> Buffer<T> {
    let mut buffers: Vec<vk::Buffer> = Vec::new();
    let mut memorys: Vec<vk::DeviceMemory> = Vec::new();
    
    for i in 0..num_sets {
      let (buffer, memory) = Buffer::create_buffer(Arc::clone(&instance), Arc::clone(&device), &usage, vk::MEMORY_PROPERTY_HOST_COHERENT_BIT, &Vec::new() as &Vec<T>);
      buffers.push(buffer);
      memorys.push(memory);
    }
    
    Buffer {
      buffer: buffers,
      memory: memorys,
      usage,
      data: Vec::with_capacity(0),
    }
  }
  
  pub fn cpu_buffer(instance: Arc<Instance>, device: Arc<Device>, usage: BufferUsage, num_sets: u32, data: Vec<T>) -> Buffer<T> {
    let mut buffers: Vec<vk::Buffer> = Vec::new();
    let mut memorys: Vec<vk::DeviceMemory> = Vec::new();
    
    for i in 0..num_sets {
      let (buffer, memory) = Buffer::create_buffer(Arc::clone(&instance), Arc::clone(&device), &usage, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT, &data);
      buffers.push(buffer);
      memorys.push(memory);
    }
    
    
    let mut buffer = Buffer {
      buffer: buffers,
      memory: memorys,
      usage,
      data: data,
    };
    
    let data = buffer.internal_data();
    buffer.fill_buffer(Arc::clone(&device), data);
    
    buffer
  }
  
  pub fn device_local_buffer(instance: Arc<Instance>, device: Arc<Device>, usage: BufferUsage, num_sets: u32, data: Vec<T>) -> Buffer<T> {
    let mut buffers: Vec<vk::Buffer> = Vec::new();
    let mut memorys: Vec<vk::DeviceMemory> = Vec::new();
    
    for i in 0..num_sets {
      let (buffer, memory) = Buffer::create_buffer(Arc::clone(&instance), Arc::clone(&device), &usage, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT, &data);
      buffers.push(buffer);
      memorys.push(memory);
    }
    
    Buffer {
      buffer: buffers,
      memory: memorys,
      usage,
      data,
    }
  }
  
  pub fn fill_buffer(&mut self, device: Arc<Device>, data: Vec<T>) {
    self.data = data;
    
    let mut host_visible_data = unsafe { mem::uninitialized() };
    let buffer_size = mem::size_of::<T>() * self.data.len();
    
    for i in 0..self.memory.len() {
      unsafe {
        let vk = device.pointers();
        let device = device.internal_object();
        
        check_errors(vk.MapMemory(*device, self.memory[i], 0, buffer_size as u64, 0, &mut host_visible_data));
        memcpy(host_visible_data, self.data.as_ptr() as *const _, buffer_size as usize);
        vk.UnmapMemory(*device, self.memory[i]);
      }
    }
  }
  
  pub fn internal_object(&self, current_buffer: usize) -> &vk::Buffer {
    &self.buffer[current_buffer]
  }
  
  pub fn internal_memory(&self, current_buffer: usize) -> &vk::DeviceMemory {
    &self.memory[current_buffer]
  }
  
  pub fn internal_data(&self) -> Vec<T> {
    self.data.to_vec()
  }
  
  pub fn size(&self) -> vk::DeviceSize {
    let mut size = (mem::size_of::<T>() * self.data.len());
    print!("");
    size as vk::DeviceSize
  }
  
  fn create_buffer(instance: Arc<Instance>, device: Arc<Device>, usage: &BufferUsage, properties: vk::MemoryPropertyFlags, data: &Vec<T>) -> (vk::Buffer, vk::DeviceMemory) {
    
    let mut buffer: vk::Buffer = unsafe { mem::uninitialized() };
    let mut buffer_memory: vk::DeviceMemory = unsafe { mem::uninitialized() };
    
    let buffer_create_info = {
      vk::BufferCreateInfo {
        sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        size: (mem::size_of::<T>() * data.len()) as vk::DeviceSize,
        usage: usage.to_bits(),
        sharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: ptr::null(),
      }
    };
    
    let mut memory_requirements: vk::MemoryRequirements = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      check_errors(vk.CreateBuffer(*device, &buffer_create_info, ptr::null(), &mut buffer));
      vk.GetBufferMemoryRequirements(*device, buffer, &mut memory_requirements);
    }
    
    let memory_type_bits_index = {
      let mut memory_properties: vk::PhysicalDeviceMemoryProperties = unsafe { mem::uninitialized() };
      
      unsafe {
        let vk = instance.pointers();
        let phys_device = device.physical_device();
        vk.GetPhysicalDeviceMemoryProperties(*phys_device, &mut memory_properties);
      }
      
      let mut index: i32 = -1;
      for i in 0..memory_properties.memoryTypeCount as usize {
        if memory_requirements.memoryTypeBits & (1 << i) != 0 && memory_properties.memoryTypes[i].propertyFlags & properties == properties {
          index = i as i32;
        }
      }
      
      if index == -1 {
        panic!("Failed to find suitable memory type");
      }
      
      index
    };
    
    let memory_allocate_info = {
      vk::MemoryAllocateInfo {
        sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
        pNext: ptr::null(),
        allocationSize: memory_requirements.size,
        memoryTypeIndex: memory_type_bits_index as u32,
      }
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      check_errors(vk.AllocateMemory(*device, &memory_allocate_info, ptr::null(), &mut buffer_memory));
      vk.BindBufferMemory(*device, buffer, buffer_memory, 0);
    }
    
    (buffer, buffer_memory)
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    println!("Destroying buffer");
    for i in 0..self.memory.len() {
      unsafe {
        let vk = device.pointers();
        let device = device.internal_object();
        vk.FreeMemory(*device, self.memory[i], ptr::null());
        vk.DestroyBuffer(*device, self.buffer[i], ptr::null());
      }
    }
  }
}