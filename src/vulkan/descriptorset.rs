use vk;

use crate::vulkan::Device;
use crate::vulkan::ImageAttachment;
use crate::vulkan::Sampler;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::buffer::UniformData;
use crate::vulkan::pool::DescriptorPool;
use crate::vulkan::check_errors;
use crate::vulkan::vkenums::{ShaderStage, DescriptorType, ImageLayout};

use std::mem;
use std::ptr;
use std::sync::Arc;

pub struct DescriptorSet {
  sets: Vec<vk::DescriptorSet>,
  layouts: Vec<vk::DescriptorSetLayout>,
}

struct DescriptorSetLayoutInfo {
  binding: u32,
  descriptor_type: DescriptorType,
  shader_stage: ShaderStage,
}

pub struct DescriptorSetBuilder {
  descriptor_set_layout_info: Vec<DescriptorSetLayoutInfo>
}

pub struct UpdateDescriptorSets<'a> {
  uniform_buffers: Vec<(u32, &'a Buffer<f32>)>,
  dynamic_uniform_buffers: Vec<(u32, &'a Buffer<f32>)>,
  images: Vec<(u32, &'a ImageAttachment, ImageLayout, Option<&'a Sampler>, DescriptorType)>,
}

impl<'a> UpdateDescriptorSets<'a> {
  pub fn new() -> UpdateDescriptorSets<'a> {
    UpdateDescriptorSets {
      uniform_buffers: Vec::new(),
      dynamic_uniform_buffers: Vec::new(),
      images: Vec::new(),
    }
  }
  
  pub fn add_built_uniformbuffer(mut self, binding: u32, uniform_buffer: &'a mut Buffer<f32>) -> UpdateDescriptorSets<'a> {
    self.uniform_buffers.push((binding, uniform_buffer));
    self
  }
  
  
  pub fn add_uniformbuffer(mut self, device: Arc<Device>, binding: u32, uniform_buffer: &'a mut Buffer<f32>, data: UniformData) -> UpdateDescriptorSets<'a> {
    let mut data = data;
    uniform_buffer.fill_entire_buffer_all_frames(Arc::clone(&device), data.build(Arc::clone(&device)));
    self.uniform_buffers.push((binding, uniform_buffer));
    self
  }
  
  pub fn add_built_dynamicuniformbuffer(mut self, binding: u32, uniform_buffer: &'a mut Buffer<f32>) -> UpdateDescriptorSets<'a> {
    self.dynamic_uniform_buffers.push((binding, uniform_buffer));
    self
  }
  
  
  pub fn add_dyanmicuniformbuffer(mut self, device: Arc<Device>, binding: u32, uniform_buffer: &'a mut Buffer<f32>, data: UniformData) -> UpdateDescriptorSets<'a> {
    let mut data = data;
    uniform_buffer.fill_entire_buffer_all_frames(Arc::clone(&device), data.build(Arc::clone(&device)));
    self.dynamic_uniform_buffers.push((binding, uniform_buffer));
    self
  }
  
  pub fn add_sampled_image(mut self, binding: u32, image: &'a ImageAttachment, image_layout: ImageLayout, sampler: &'a Sampler) -> UpdateDescriptorSets<'a> {
    self.images.push((binding, image, image_layout, Some(sampler), DescriptorType::CombinedImageSampler));
    self
  }
  
  pub fn add_storage_image(mut self, binding: u32, image: &'a ImageAttachment, image_layout: ImageLayout) -> UpdateDescriptorSets<'a> {
    self.images.push((binding, image, image_layout, None, DescriptorType::StorageImage));
    self
  }
  
  pub fn finish_update(self, device: Arc<Device>, descriptor_set: &DescriptorSet) {
    let sets = descriptor_set.all_sets();

    for j in 0..sets.len() {
     for i in 0..self.uniform_buffers.len() {
       let (binding, uniform_buffer) = &self.uniform_buffers[i];
       let descriptor_buffer_info = vk::DescriptorBufferInfo {
            buffer: *uniform_buffer.internal_object(j),
            offset: 0,
            range: vk::WHOLE_SIZE,
         };
        
        let write_descriptor_set = 
          vk::WriteDescriptorSet {
            sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ptr::null(),
            dstSet: sets[j],
            dstBinding: *binding,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: DescriptorType::UniformBuffer.to_bits(),
            pImageInfo: ptr::null(),
            pBufferInfo: &descriptor_buffer_info,
            pTexelBufferView: ptr::null(),
          };
        
        let vk = device.pointers();
        let device = device.internal_object();
        unsafe {
          vk.UpdateDescriptorSets(*device, 1, &write_descriptor_set, 0, ptr::null());
        }
      }
      
      for i in 0..self.dynamic_uniform_buffers.len() {
        let (binding, uniform_buffer) = &self.dynamic_uniform_buffers[i];
        let descriptor_buffer_info =  vk::DescriptorBufferInfo {
            buffer: *uniform_buffer.internal_object(j),
            offset: 0,
            range: vk::WHOLE_SIZE,
          };
        
        let write_descriptor_set =
          vk::WriteDescriptorSet {
            sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ptr::null(),
            dstSet: sets[j],
            dstBinding: *binding,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: DescriptorType::UniformBufferDynamic.to_bits(),
            pImageInfo: ptr::null(),
            pBufferInfo: &descriptor_buffer_info,
            pTexelBufferView: ptr::null(),
          };
        
        
        let vk = device.pointers();
        let device = device.internal_object();
        unsafe {
          vk.UpdateDescriptorSets(*device, 1, &write_descriptor_set, 0, ptr::null());
        }
      }
      
      for i in 0..self.images.len() {
        let (binding, ref image, ref layout, ref sampler, ref descriptor_type) = self.images[i];
        
        let descriptor_image_info;
        
        if sampler.is_some() {
          descriptor_image_info = 
            vk::DescriptorImageInfo {
              sampler: sampler.unwrap().internal_object(),
              imageView: image.get_image_view(),
              imageLayout: layout.to_bits(),
            };
          
        } else {
          descriptor_image_info =
            vk::DescriptorImageInfo {
              sampler: 0,
              imageView: image.get_image_view(),
              imageLayout: layout.to_bits(),
            };
        }
        
        let write_descriptor_set = 
          vk::WriteDescriptorSet {
            sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ptr::null(),
            dstSet: sets[j],
            dstBinding: binding,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: descriptor_type.to_bits(),
            pImageInfo: &descriptor_image_info,
            pBufferInfo: ptr::null(),
            pTexelBufferView: ptr::null(),
        };
        
        let vk = device.pointers();
        let device = device.internal_object();
        unsafe {
          vk.UpdateDescriptorSets(*device, 1, &write_descriptor_set, 0, ptr::null());
        }
      }
    }
  }
}

impl DescriptorSetBuilder {
  pub fn new() -> DescriptorSetBuilder {
    DescriptorSetBuilder {
      descriptor_set_layout_info: Vec::new(),
    }
  }
  
  pub fn fragment_input_attachment(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::InputAttachment,
        shader_stage: ShaderStage::Fragment,
      }
    );
    
    self
  }
  
  pub fn vertex_uniform_buffer(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::UniformBuffer,
        shader_stage: ShaderStage::Vertex,
      }
    );
    self
  }
  
  pub fn fragment_uniform_buffer(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::UniformBuffer,
        shader_stage: ShaderStage::Fragment,
      }
    );
    self
  }
  
  pub fn vertex_dynamic_uniform_buffer(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::UniformBufferDynamic,
        shader_stage: ShaderStage::Vertex,
      }
    );
    self
  }
  
  pub fn fragment_dyanmic_uniform_buffer(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::UniformBufferDynamic,
        shader_stage: ShaderStage::Fragment,
      }
    );
    self
  }
  
  pub fn vertex_combined_image_sampler(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::CombinedImageSampler,
        shader_stage: ShaderStage::Vertex,
      }
    );
    self
  }
  
  pub fn fragment_combined_image_sampler(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::CombinedImageSampler,
        shader_stage: ShaderStage::Fragment,
      }
    );
    self
  }
  
  pub fn compute_storage_image(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::StorageImage,
        shader_stage: ShaderStage::Compute,
      }
    );
    self
  }
  
  pub fn compute_combined_image_sampler(mut self, binding_location: u32) -> DescriptorSetBuilder {
    self.descriptor_set_layout_info.push(
      DescriptorSetLayoutInfo {
        binding: binding_location,
        descriptor_type: DescriptorType::CombinedImageSampler,
        shader_stage: ShaderStage::Compute,
      }
    );
    self
  }
  
  pub fn build(&self, device: Arc<Device>, set_pool: &DescriptorPool, num_sets: u32) -> DescriptorSet {
    let mut layouts: Vec<vk::DescriptorSetLayout> = Vec::with_capacity(num_sets as usize);
    let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = Vec::with_capacity(num_sets as usize);
    let mut descriptor_sets: Vec<vk::DescriptorSet> = Vec::with_capacity(num_sets as usize);
    
    let mut binding_counts: Vec<u32> = Vec::new();
    
    for i in 0..self.descriptor_set_layout_info.len() {
      let binding = self.descriptor_set_layout_info[i].binding as usize;
      if binding+1 > binding_counts.len() {
        binding_counts.push(1);
      } else {
        binding_counts[binding] += 1;
      }
    }
    
    for i in 0..self.descriptor_set_layout_info.len() {
      bindings.push(
        vk::DescriptorSetLayoutBinding {
          binding: self.descriptor_set_layout_info[i].binding,
          descriptorType: self.descriptor_set_layout_info[i].descriptor_type.to_bits(),
          descriptorCount: binding_counts[i],
          stageFlags: self.descriptor_set_layout_info[i].shader_stage.to_bits(),
          pImmutableSamplers: ptr::null(),
        }
      );
    }
    
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
    
    for _ in 0..num_sets as usize {
      let mut layout = unsafe { mem::uninitialized() };
      unsafe {
        vk.CreateDescriptorSetLayout(*device, &descriptor_set_layout_create_info, ptr::null(), &mut layout);
      }
      
      layouts.push(layout);
    }
    
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
      
      unsafe {
        check_errors(vk.AllocateDescriptorSets(*device, &descriptor_set_allocate_info, &mut descriptor_set));
      }
      
      descriptor_sets.push(descriptor_set);
    }
    
    DescriptorSet::new_with_internals(descriptor_sets, layouts)
  }
}

impl DescriptorSet {
  pub fn new_with_internals(sets: Vec<vk::DescriptorSet>, layouts: Vec<vk::DescriptorSetLayout>) -> DescriptorSet {
    DescriptorSet {
      sets,
      layouts,
    }
  }
  
  pub fn set(&self, current_buffer: usize) -> &vk::DescriptorSet {
    &self.sets[current_buffer]
  }
  
  pub fn all_sets(&self) -> &Vec<vk::DescriptorSet> {
    &self.sets
  }
  
  pub fn layouts(&self) -> &Vec<vk::DescriptorSetLayout> {
    &self.layouts
  }
  
  pub fn layouts_clone(&self) -> Vec<vk::DescriptorSetLayout> {
    (*self.layouts).to_vec()
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
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
