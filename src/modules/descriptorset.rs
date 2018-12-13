use vk;

use crate::modules::Device;
use crate::modules::DescriptorPool;
use crate::ownage::check_errors;

use std::mem;
use std::ptr;

pub struct DescriptorSet {
  set: vk::DescriptorSet,
  layout: vk::DescriptorSetLayout,
}

impl DescriptorSet {
  pub fn new(device: &Device, set_pool: &DescriptorPool) -> DescriptorSet {
    let layout = DescriptorSet::create_layout(device);
    let set = DescriptorSet::create_set(device, &layout, set_pool);
    
    DescriptorSet {
      set,
      layout,
    }
  }
  
  pub fn set(&self) -> &vk::DescriptorSet {
    &self.set
  }
  
  pub fn layout(&self) -> &vk::DescriptorSetLayout {
    &self.layout
  }
  
  fn create_set(device: &Device, layout: &vk::DescriptorSetLayout, set_pool: &DescriptorPool) -> vk::DescriptorSet {
    let mut descriptor_sets: Vec<vk::DescriptorSet> = Vec::with_capacity(1);
    
    let descriptor_set_allocate_info = {
      vk::DescriptorSetAllocateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
        pNext: ptr::null(),
        descriptorPool: *set_pool.local_pool(),
        descriptorSetCount: 1,
        pSetLayouts: layout,
      }
    };
    
    let vk = device.pointers();
    let device = device.local_device();
    
    unsafe {
      check_errors(vk.AllocateDescriptorSets(*device, &descriptor_set_allocate_info, descriptor_sets.as_mut_ptr()));
      descriptor_sets.set_len(1);
    }
    
    descriptor_sets[0]
  }
  
  fn create_layout(device: &Device) -> vk::DescriptorSetLayout {
    let mut layout: vk::DescriptorSetLayout = unsafe { mem::uninitialized() };
    let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = Vec::with_capacity(1);
    
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
    let device = device.local_device();
    
    unsafe {
      vk.CreateDescriptorSetLayout(*device, &descriptor_set_layout_create_info, ptr::null(), &mut layout);
    }
    
    layout
  }
  
  pub fn destroy(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.local_device();
    
    println!("Destroying DescriptorSet Layout");
    
    unsafe {
      vk.DestroyDescriptorSetLayout(*device, self.layout, ptr::null());
    }
  }
}
