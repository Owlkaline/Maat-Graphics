use vk;

use crate::vulkan::buffer::{CommandBuffer, BufferUsage};
use crate::vulkan::pool::CommandPool;
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
  size: u64,
  data: Vec<T>,
}

impl<T: Clone> Buffer<T> {
  fn illegal_size(max_size: &u64, data: &Vec<T>) -> bool {
    *max_size < data.len() as u64*mem::size_of::<T>() as u64
  }
  
  fn align_data(device: Arc<Device>, data: &mut Vec<T>) {
    let mut buffer_size = mem::size_of::<T>() * data.len();
    let min_alignment = device.get_non_coherent_atom_size();
    
    while buffer_size as u64%min_alignment != 0 {
      let temp = data[data.len()-1].clone();
      data.push(temp);
      buffer_size = mem::size_of::<T>() * data.len();
    }
  }
  
  fn align_phantom_data(device: Arc<Device>, data_len: &mut u64) {
    let mut buffer_size = mem::size_of::<T>() as u64 * *data_len;
    let min_alignment = device.get_non_coherent_atom_size();
    
    while buffer_size as u64%min_alignment != 0 {
      *data_len += 1;
      buffer_size = mem::size_of::<T>() as u64 * *data_len;
    }
  }
  
  pub fn cpu_buffer(instance: Arc<Instance>, device: Arc<Device>, usage: BufferUsage, num_sets: u32, data_len: u64) -> Buffer<T> {
    let mut data_len = data_len;
    Buffer::<T>::align_phantom_data(Arc::clone(&device), &mut data_len);
    
    let mut buffers: Vec<vk::Buffer> = Vec::new();
    let mut memorys: Vec<vk::DeviceMemory> = Vec::new();
    
    for _ in 0..num_sets {
      let (buffer, memory) = Buffer::<T>::create_buffer(Arc::clone(&instance), Arc::clone(&device), &usage, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT, data_len);
      buffers.push(buffer);
      memorys.push(memory);
    }
    
    let buffer = Buffer {
      buffer: buffers,
      memory: memorys,
      usage,
      size: mem::size_of::<T>() as u64*data_len,
      data: Vec::new(),
    };
    
    buffer
  }
  
  pub fn cpu_buffer_with_data(instance: Arc<Instance>, device: Arc<Device>, usage: BufferUsage, num_sets: u32, data: Vec<T>) -> Buffer<T> {
    let mut data = data;
    Buffer::align_data(Arc::clone(&device), &mut data);
    let mut buffers: Vec<vk::Buffer> = Vec::new();
    let mut memorys: Vec<vk::DeviceMemory> = Vec::new();
    
    for _ in 0..num_sets {
      let (buffer, memory) = Buffer::<T>::create_buffer(Arc::clone(&instance), Arc::clone(&device), &usage, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT, data.len() as u64);
      buffers.push(buffer);
      memorys.push(memory);
    }
    
    let mut buffer = Buffer {
      buffer: buffers,
      memory: memorys,
      usage,
      size: mem::size_of::<T>() as u64*data.len() as u64,
      data: data,
    };
    
    let data = buffer.internal_data();
    buffer.fill_entire_buffer_all_frames(Arc::clone(&device), data);
    
    buffer
  }
  
  pub fn device_local_buffer(instance: Arc<Instance>, device: Arc<Device>, usage: BufferUsage, num_sets: u32, data_len: u64) -> Buffer<T> {
    let mut data_len = data_len;
    Buffer::<T>::align_phantom_data(Arc::clone(&device), &mut data_len);
    let mut buffers: Vec<vk::Buffer> = Vec::new();
    let mut memorys: Vec<vk::DeviceMemory> = Vec::new();
    
    for _ in 0..num_sets {
      let (buffer, memory) = Buffer::<T>::create_buffer(Arc::clone(&instance), Arc::clone(&device), &usage, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT, data_len);
      buffers.push(buffer);
      memorys.push(memory);
    }
    
    Buffer {
      buffer: buffers,
      memory: memorys,
      usage,
      size: mem::size_of::<T>() as u64*data_len,
      data: Vec::new(),
    }
  }
  
  pub fn device_local_buffer_with_data(instance: Arc<Instance>, device: Arc<Device>, command_pool: &CommandPool, graphics_queue: &vk::Queue, buffer_usage: BufferUsage, data: Vec<T>) -> Buffer<T> {
    let mut data = data;
    Buffer::align_data(Arc::clone(&device), &mut data);
    let mut buffer_usage = buffer_usage;
    buffer_usage.set_as_transfer_dst();
    
    let data_len = data.len() as u64;
    
    let usage_src = BufferUsage::transfer_src_buffer();
    let usage_dst = buffer_usage;
    
    let staging_buffer: Buffer<T> = Buffer::cpu_buffer_with_data(Arc::clone(&instance), Arc::clone(&device), usage_src, 1, data);
    let buffer: Buffer<T> = Buffer::device_local_buffer(Arc::clone(&instance), Arc::clone(&device), usage_dst, 1, data_len);
    
    let command_buffer = CommandBuffer::begin_single_time_command(Arc::clone(&device), command_pool);
    command_buffer.copy_buffer(Arc::clone(&device), &staging_buffer, &buffer, 0);
    command_buffer.end_single_time_command(Arc::clone(&device), command_pool, graphics_queue);
    
    staging_buffer.destroy(Arc::clone(&device));
    
    buffer
  }
  
  pub fn _fill_partial_buffer(&mut self, device: Arc<Device>, current_buffer: usize, offset: u32, data: Vec<T>) {
    let mut data = data;
    Buffer::align_data(Arc::clone(&device), &mut data);
    if Buffer::illegal_size(&self.size, &data) {
      return;
    }
    self.data = data;
    
    let mut host_visible_data = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let buffer_offset = mem::size_of::<T>() * offset as usize;
    let buffer_size = mem::size_of::<T>() * self.data.len();
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      
      check_errors(vk.MapMemory(*device, self.memory[current_buffer], buffer_offset as u64, buffer_size as u64, 0, &mut host_visible_data));
      memcpy(host_visible_data, self.data.as_ptr() as *const _, buffer_size as usize);
      let mapped_memory_range = vk::MappedMemoryRange {
        sType: vk::STRUCTURE_TYPE_MAPPED_MEMORY_RANGE,
        pNext: ptr::null(),
        memory: self.memory[current_buffer],
        offset: buffer_offset as vk::DeviceSize,
        size: buffer_size as vk::DeviceSize,
      };
      vk.FlushMappedMemoryRanges(*device, 1, &mapped_memory_range);
      vk.UnmapMemory(*device, self.memory[current_buffer]);
    }
  }
  
  pub fn fill_entire_buffer_single_frame(&mut self, device: Arc<Device>, current_buffer: usize, data: Vec<T>) {
    let mut data = data;
    Buffer::align_data(Arc::clone(&device), &mut data);
    if Buffer::illegal_size(&self.size, &data) {
      return;
    }
    self.data = data;
    
    let mut host_visible_data = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let buffer_size = mem::size_of::<T>() * self.data.len();
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      
      check_errors(vk.MapMemory(*device, self.memory[current_buffer], 0, buffer_size as u64, 0, &mut host_visible_data));
      memcpy(host_visible_data, self.data.as_ptr() as *const _, buffer_size as usize);
      let mapped_memory_range = vk::MappedMemoryRange {
        sType: vk::STRUCTURE_TYPE_MAPPED_MEMORY_RANGE,
        pNext: ptr::null(),
        memory: self.memory[current_buffer],
        offset: 0 as vk::DeviceSize,
        size: buffer_size as vk::DeviceSize,
      };
      let mut ranges = Vec::new();
      ranges.push(mapped_memory_range);
      vk.FlushMappedMemoryRanges(*device, 1, ranges.as_ptr());
      vk.UnmapMemory(*device, self.memory[current_buffer]);
    }
  }
  
  pub fn fill_entire_buffer_all_frames(&mut self, device: Arc<Device>, data: Vec<T>) {
    let mut data = data;
    Buffer::align_data(Arc::clone(&device), &mut data);
    if Buffer::illegal_size(&self.size, &data) {
      return;
    }
    self.data = data;
    
    let mut host_visible_data = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let buffer_size = mem::size_of::<T>() * self.data.len();
    
    for i in 0..self.memory.len() {
      unsafe {
        let vk = device.pointers();
        let device = device.internal_object();
        
        check_errors(vk.MapMemory(*device, self.memory[i], 0, buffer_size as u64, 0, &mut host_visible_data));
        memcpy(host_visible_data, self.data.as_ptr() as *const _, buffer_size as usize);
          let mapped_memory_range = vk::MappedMemoryRange {
          sType: vk::STRUCTURE_TYPE_MAPPED_MEMORY_RANGE,
          pNext: ptr::null(),
          memory: self.memory[i],
          offset: 0 as vk::DeviceSize,
          size: buffer_size as vk::DeviceSize,
        };
        vk.FlushMappedMemoryRanges(*device, 1, &mapped_memory_range);
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
  
  pub fn max_size(&self) -> u64 {
    self.size
  }
  
  fn create_buffer(instance: Arc<Instance>, device: Arc<Device>, usage: &BufferUsage, properties: vk::MemoryPropertyFlags, data_len: u64) -> (vk::Buffer, vk::DeviceMemory) {
    let mut data_len = data_len;
    Buffer::<T>::align_phantom_data(Arc::clone(&device), &mut data_len);
    let mut buffer: vk::Buffer = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let mut buffer_memory: vk::DeviceMemory = unsafe { mem::MaybeUninit::uninit().assume_init() };
    
    let buffer_create_info = {
      vk::BufferCreateInfo {
        sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        size: (mem::size_of::<T>() * data_len as usize) as vk::DeviceSize,
        usage: usage.to_bits(),
        sharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: ptr::null(),
      }
    };
    
    let mut memory_requirements: vk::MemoryRequirements = unsafe { mem::MaybeUninit::uninit().assume_init() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.internal_object();
      check_errors(vk.CreateBuffer(*device, &buffer_create_info, ptr::null(), &mut buffer));
      vk.GetBufferMemoryRequirements(*device, buffer, &mut memory_requirements);
    }
    
    let memory_type_bits_index = {
      let mut memory_properties: vk::PhysicalDeviceMemoryProperties = unsafe { mem::MaybeUninit::uninit().assume_init() };
      
      unsafe {
        let vk = instance.pointers();
        let phys_device = device.physical_device();
        vk.GetPhysicalDeviceMemoryProperties(*phys_device, &mut memory_properties);
      }
      
      let mut index: i32 = -1;
      for i in 0..memory_properties.memoryTypeCount as usize {
        if memory_requirements.memoryTypeBits & (1 << i) != 0 && memory_properties.memoryTypes[i].propertyFlags & properties == properties && (memory_properties.memoryTypes[i].propertyFlags & (0x00000080 | 0x00000040)) == 0 {
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
