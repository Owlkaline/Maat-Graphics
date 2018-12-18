use vk;

use crate::modules::Device;
use crate::modules::buffer::Buffer;
use crate::modules::pool::DescriptorPool;
use crate::ownage::check_errors;

use std::mem;
use std::ptr;

pub struct DescriptorSet {
  sets: Vec<vk::DescriptorSet>,
  layouts: Vec<vk::DescriptorSetLayout>,
}

impl DescriptorSet {
  pub fn new(device: &Device, set_pool: &DescriptorPool, num_sets: u32) -> DescriptorSet {
    let layouts = DescriptorSet::create_layouts(device, num_sets);
    let sets = DescriptorSet::create_sets(device, &layouts, set_pool, num_sets);
    
    DescriptorSet {
      sets,
      layouts,
    }
  }
  
  pub fn sets(&self) -> &Vec<vk::DescriptorSet> {
    &self.sets
  }
  
  pub fn layouts(&self) -> &Vec<vk::DescriptorSetLayout> {
    &self.layouts
  }
  
  pub fn update_sets<T: Clone>(&self, device: &Device, uniform_buffer: &Buffer<T>) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    let descriptor_buffer_info = {
      vk::DescriptorBufferInfo {
        buffer: *uniform_buffer.internal_object(),
        offset: 0,
        range: uniform_buffer.size(),
      }
    };
    
    for i in 0..self.sets.len() {
      let write_descriptor_set = {
        vk::WriteDescriptorSet {
          sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
          pNext: ptr::null(),
          dstSet: self.sets[i],
          dstBinding: 0,
          dstArrayElement: 0,
          descriptorCount: 1,
          descriptorType: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
          pImageInfo: ptr::null(),
          pBufferInfo: &descriptor_buffer_info,
          pTexelBufferView: ptr::null(),
        }
      };
      
      unsafe {
        vk.UpdateDescriptorSets(*device, 1, &write_descriptor_set, 0, ptr::null());
      }
    }
  }
  
  fn create_sets(device: &Device, layouts: &Vec<vk::DescriptorSetLayout>, set_pool: &DescriptorPool, num_sets: u32) -> Vec<vk::DescriptorSet> {
    let mut descriptor_sets: Vec<vk::DescriptorSet> = Vec::with_capacity(num_sets as usize);
    
    for i in 0..num_sets as usize {
      let mut descriptor_set: vk::DescriptorSet = unsafe { mem::uninitialized() };
      let descriptor_set_allocate_info = {
        vk::DescriptorSetAllocateInfo {
          sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
          pNext: ptr::null(),
          descriptorPool: *set_pool.local_pool(),
          descriptorSetCount: 1,//layouts.len() as u32,
          pSetLayouts: &layouts[i],
        }
      };
      
      let vk = device.pointers();
      let device = device.internal_object();
      
      unsafe {
        check_errors(vk.AllocateDescriptorSets(*device, &descriptor_set_allocate_info, &mut descriptor_set));
      }
      
      descriptor_sets.push(descriptor_set);
    }
    
    descriptor_sets
  }
  
  fn create_layouts(device: &Device, num_sets: u32) -> Vec<vk::DescriptorSetLayout> {
    let mut layouts: Vec<vk::DescriptorSetLayout> = Vec::with_capacity(num_sets as usize);
    let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = Vec::with_capacity(num_sets as usize);
    
    bindings.push(
      vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptorType: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        descriptorCount: 1,
        stageFlags: vk::SHADER_STAGE_VERTEX_BIT,
        pImmutableSamplers: ptr::null(),
      }
    );
    
    /*
    descriptor_bindings.push(
      vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptorType: vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
        descriptorCount: 1,
        stageFlags: vk::SHADER_STAGE_FRAGMENT_BIT,
        pImmutableSamplers: ptr::null(),
      }
    );
    */
    
    let descriptor_set_layout_create_info = {
      vk::DescriptorSetLayoutCreateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        bindingCount: bindings.len() as u32,
        pBindings: bindings.as_ptr(),
      }
    };
    
    let vk = device.pointers();
    let device = device.internal_object();
    
    for i in 0..num_sets as usize {
      let mut layout = unsafe { mem::uninitialized() };
      unsafe {
        vk.CreateDescriptorSetLayout(*device, &descriptor_set_layout_create_info, ptr::null(), &mut layout);
      }
      
      layouts.push(layout);
    }
    
    layouts
  }
  
  pub fn destroy(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    println!("Destroying DescriptorSet Layout");
    
    for layout in &self.layouts {
      unsafe {
        vk.DestroyDescriptorSetLayout(*device, *layout, ptr::null());
      }
    }
  }
}
