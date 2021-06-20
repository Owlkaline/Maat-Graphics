use ash::vk;
use ash::util::read_spv;
use ash::version::DeviceV1_0;

use std::io::Read;
use std::io::Seek;

use std::ffi::CString;

use crate::modules::{VkDevice, ComputePipeline, DescriptorSet};

pub struct ComputeShader {
  compute_shader: vk::ShaderModule,
  pipeline_layout: vk::PipelineLayout,
  compute_pipeline: ComputePipeline,
}

impl ComputeShader {
  pub fn new<W: Read + Seek>(device: &VkDevice, mut compute_shader: W,
                                     descriptor_sets: &DescriptorSet) -> ComputeShader {
    let compute_code = read_spv(&mut compute_shader).expect("Failed to read vertex shader");
    let compute_info = vk::ShaderModuleCreateInfo::builder().code(&compute_code);
    
    let compute_shader = unsafe {
      device.internal().create_shader_module(&compute_info, None).expect("Vertex shader module error")
    };
    
    let layout_info = {
      if descriptor_sets.layouts().len() == 0 {
        vk::PipelineLayoutCreateInfo::default()
      } else {
        vk::PipelineLayoutCreateInfo::builder().set_layouts(descriptor_sets.layouts()).build()
      }
    };
    
    let pipeline_layout = unsafe {
      device.internal().create_pipeline_layout(&layout_info, None).unwrap()
    };
    
    let shader_entry = CString::new("main").unwrap();
    let shader_stage_create_info = 
      vk::PipelineShaderStageCreateInfo {
          module: compute_shader,
          p_name: shader_entry.as_ptr(),
          stage: vk::ShaderStageFlags::COMPUTE,
          //p_specialization_info: Default::default(),
          ..Default::default()
      };
    
    let compute_pipeline = ComputePipeline::new(device, &pipeline_layout, shader_stage_create_info);
    
    ComputeShader {
      compute_shader,
      pipeline_layout,
      compute_pipeline,
    }
  }
  
  pub fn pipeline(&self) -> &ComputePipeline {
    &self.compute_pipeline
  }
  
  pub fn pipeline_layout(&self) -> vk::PipelineLayout {
    self.pipeline_layout
  }
  
  pub fn destroy(&self, device: &VkDevice) {
    self.compute_pipeline.destroy(device);
    
    unsafe {
      device.internal().destroy_pipeline_layout(self.pipeline_layout, None);
      device.internal().destroy_shader_module(self.compute_shader, None);
    }
  }
}
