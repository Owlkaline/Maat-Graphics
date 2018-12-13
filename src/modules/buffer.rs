use vk;

use crate::modules::BufferUsage;
use crate::modules::Device;

use std::mem;
use std::ptr;
use std::ffi::c_void;

pub struct Buffer {
  buffer: vk::Buffer,
  memory: vk::DeviceMemory,
  usage: BufferUsage,
}
/*

impl Buffer {
  pub fn with_usage(instance: &Instance, device: &Device, usage: BufferUsage, size: usize, data: *const c_void) -> Buffer {
    
    
    let (staging_vertex_buffer, staging_vertex_buffer_memory) = Buffer::create_buffer(instance, device, buffer_size, vk::BUFFER_USAGE_TRANSFER_SRC_BIT, vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT);
    
    let mut host_visible_data = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      check_errors(vk.MapMemory(*device, staging_vertex_buffer_memory, 0, size, 0, &mut host_visible_data));
      memcpy(host_visible_data, data, size);
      vk.UnmapMemory(*device, staging_vertex_buffer_memory);
    }
    
    let (vertex_buffer, vertex_buffer_memory) = Buffer::create_buffer(instance, device, buffer_size, vk::BUFFER_USAGE_VERTEX_BUFFER_BIT | vk::BUFFER_USAGE_TRANSFER_DST_BIT, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    
    let command_buffer = Vulkan::begin_single_time_command(device, command_pool);
    
    let buffer_copy = {
      vk::BufferCopy {
        srcOffset: 0,
        dstOffset: 0,
        size: buffer_size,
      }
    };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      vk.CmdCopyBuffer(command_buffer, staging_vertex_buffer, vertex_buffer, 1, &buffer_copy);
    }
    
    Vulkan::end_single_time_command(device, command_buffer, command_pool, graphics_queue);
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
      vk.FreeMemory(*device, staging_vertex_buffer_memory, ptr::null());
      vk.DestroyBuffer(*device, staging_vertex_buffer, ptr::null());
    }
    
    Buffer {
      buffer,
      memory,
      usage,
    }
  }
  
  fn create_buffer(instance: &Instance, device: &Device, buffer_size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory) {
    
    let mut buffer: vk::Buffer = unsafe { mem::uninitialized() };
    let mut buffer_memory: vk::DeviceMemory = unsafe { mem::uninitialized() };
    
    let mut buffer_create_info = {
      vk::BufferCreateInfo {
        sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        size: buffer_size,
        usage: usage,
        sharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: ptr::null(),
      }
    };
    
    let mut memory_requirements: vk::MemoryRequirements = unsafe { mem::uninitialized() };
    
    unsafe {
      let vk = device.pointers();
      let device = device.local_device();
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
      let device = device.local_device();
      check_errors(vk.AllocateMemory(*device, &memory_allocate_info, ptr::null(), &mut buffer_memory));
      vk.BindBufferMemory(*device, buffer, buffer_memory, 0);
    }
    
    (buffer, buffer_memory)
  }
  
  pub fn destroy(&self) {
    
  }
}*/
