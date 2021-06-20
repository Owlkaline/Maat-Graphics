use ash::vk;
use ash::version::DeviceV1_0;
use ash::util::Align;
use std::mem::{align_of};
use std::mem;
use std::slice;
use std::ptr;

use crate::modules::VkDevice;

pub struct Memory<T: Copy> {
  memory: vk::DeviceMemory,
  requirements: vk::MemoryRequirements,
  _type: T,
}

impl<T: Copy> Memory<T> {
  pub fn new_empty() -> Memory<u8> {
    Memory {
      memory: vk::DeviceMemory::null(),
      requirements: vk::MemoryRequirements::default(),
      _type: 0u8,
    }
  }
  
  pub fn data_from_memory(&self, device: &VkDevice, data_len: usize) -> Vec<T> {
    Memory::<T>::map_memory_to_data(device, self.memory, 
                                    self.requirements, data_len)
  }
  
  pub fn new_image_memory(device: &VkDevice, image: &vk::Image, memory_property: vk::MemoryPropertyFlags) -> Memory<u8> {
    
    let requirements = Memory::<u8>::image_memory_requirements(device, *image);
    
    let image_memory_index = Memory::<u8>::find_memorytype_index(
        &requirements,
        &device.device_memory_properties(),
        memory_property,
    ).expect("Unable to find suitable memory index for depth image.");
    
    let memory = Memory::<u8>::allocate_memory(device, requirements, image_memory_index);
    
    unsafe {
      device.internal()
            .bind_image_memory(*image, memory, 0)
            .expect("Unable to bind image memory");
    };
    
    Memory {
      memory,
      requirements,
      _type: 0u8,
    }
  }
  
  pub fn  new_buffer_memory(device: &VkDevice, buffer: &vk::Buffer, memory_property: vk::MemoryPropertyFlags,
                               data: &Vec<T>) -> Memory<T> {
    
    let requirements = Memory::<T>::buffer_memory_requirements(device, *buffer);
    
    let buffer_memory_index = Memory::<T>::find_memorytype_index(
        &requirements,
        &device.device_memory_properties(),
        memory_property,
    ).expect("Unable to find suitable memory index for buffer.");
    
    let memory = Memory::<T>::allocate_memory(device, requirements, buffer_memory_index);
    
    Memory::<T>::map_data_to_memory(device, memory, requirements, data);
    
    unsafe {
      device.internal()
            .bind_buffer_memory(*buffer, memory, 0)
            .expect("Unable to bind depth image memory");
    };
    
    Memory {
      memory,
      requirements,
      _type: data[0],
    }
  }
  
  pub fn internal(&self) -> vk::DeviceMemory {
    self.memory
  }
  
  pub fn destroy(&self, device: &VkDevice) {
    unsafe {
      device.internal().free_memory(self.memory, None);
    }
  }
  
  pub fn image_memory_requirements(device: &VkDevice, image: vk::Image) -> vk::MemoryRequirements {
    unsafe { device.internal().get_image_memory_requirements(image) }
  }
  
  pub fn buffer_memory_requirements(device: &VkDevice, image: vk::Buffer) -> vk::MemoryRequirements {
    unsafe { device.internal().get_buffer_memory_requirements(image) }
  }
  
  pub fn map_data_to_memory(device: &VkDevice, memory: vk::DeviceMemory, 
                    memory_requirements: vk::MemoryRequirements, data: &Vec<T>) {
    let index_ptr = unsafe {
      device.internal()
      .map_memory(
          memory,
          0,
          memory_requirements.size,
          vk::MemoryMapFlags::empty(),
      )
      .unwrap() };
    
    let mut index_slice = unsafe { Align::new(
        index_ptr,
        align_of::<T>() as u64,
        memory_requirements.size,
    ) };
    
    unsafe {
      index_slice.copy_from_slice(data);
      device.internal().unmap_memory(memory);
    }
  }
  
  pub fn map_memory_to_data(device: &VkDevice, memory: vk::DeviceMemory, 
                    memory_requirements: vk::MemoryRequirements, data_len: usize) -> Vec<T> {
    let index_ptr = unsafe {
      device.internal()
      .map_memory(
          memory,
          0,
          memory_requirements.size,
          vk::MemoryMapFlags::empty(),
      )
      .unwrap() };
    
    let mut data_slice = unsafe { slice::from_raw_parts(index_ptr as *const T, data_len) };
    let data = data_slice.to_vec();
    
    unsafe {
      device.internal().unmap_memory(memory);
    }
    
    data
  }
  
  pub fn allocate_memory(device: &VkDevice, memory_requirements: vk::MemoryRequirements, memory_index: u32) -> vk::DeviceMemory {
    let memory_allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(memory_requirements.size)
        .memory_type_index(memory_index);
    
    unsafe {
      device.internal()
            .allocate_memory(&memory_allocate_info, None)
            .unwrap()
    }
  }
  
  pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
  ) -> Option<u32> {
      memory_prop.memory_types[..memory_prop.memory_type_count as _]
          .iter()
          .enumerate()
          .find(|(index, memory_type)| {
              (1 << index) & memory_req.memory_type_bits != 0
                  && memory_type.property_flags & flags == flags
          })
          .map(|(index, _memory_type)| index as _)
  }
}
