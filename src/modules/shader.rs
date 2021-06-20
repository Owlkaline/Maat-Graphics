use ash::vk;
use ash::util::read_spv;
use ash::version::DeviceV1_0;

use std::io::Cursor;
use std::io::Seek;
use std::io::Read;
use std::ffi::CString;

use crate::modules::{VkDevice, GraphicsPipeline, GraphicsPipelineBuilder, Viewport, Scissors, Renderpass};

use std::mem;

pub struct Shader<T: Sized + Copy> {
  vertex_struct: T,
  vertex_shader: vk::ShaderModule,
  fragment_shader: vk::ShaderModule,
  pipeline_layout: vk::PipelineLayout,
  graphics_pipeline: GraphicsPipeline,
}

impl<T: Sized + Copy> Shader<T> {
  pub fn new<W: Read + Seek>(device: &VkDevice, mut vertex_shader: W, mut fragment_shader: W, 
             vertex_struct: T, offsets: Vec<u32>, graphics_pipeline_builder: GraphicsPipelineBuilder,
             renderpass: &Renderpass, viewport: &Viewport, scissors: &Scissors, 
             descriptor_set_layouts: &Vec<vk::DescriptorSetLayout>) -> Shader<T> {
    let vertex_code = read_spv(&mut vertex_shader).expect("Failed to read vertex shader");
    let fragment_code = read_spv(&mut fragment_shader).expect("Failed to read fragment shader");
    
    let vertex_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);
    let fragment_info = vk::ShaderModuleCreateInfo::builder().code(&fragment_code);
    
    let vertex_shader = unsafe {
      device.internal().create_shader_module(&vertex_info, None).expect("Vertex shader module error")
    };
    let fragment_shader = unsafe {
      device.internal().create_shader_module(&fragment_info, None).expect("Fragment shader module error")
    };
    
    let layout_info = {
      if descriptor_set_layouts.len() == 0 {
        vk::PipelineLayoutCreateInfo::default()
      } else {
        vk::PipelineLayoutCreateInfo::builder().set_layouts(descriptor_set_layouts).build()
      }
    };
    
    let pipeline_layout = unsafe {
      device.internal().create_pipeline_layout(&layout_info, None).unwrap()
    };
    
    let shader_entry = CString::new("main").unwrap();
    let shader_stage_create_info = [
      vk::PipelineShaderStageCreateInfo {
          module: vertex_shader,
          p_name: shader_entry.as_ptr(),
          stage: vk::ShaderStageFlags::VERTEX,
          ..Default::default()
      },
      vk::PipelineShaderStageCreateInfo {
          s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
          module: fragment_shader,
          p_name: shader_entry.as_ptr(),
          stage: vk::ShaderStageFlags::FRAGMENT,
          ..Default::default()
      },
    ];
    
    let vertex_input_binding = 
    [
      vk::VertexInputBindingDescription {
        binding: 0,
        stride: mem::size_of::<T>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
      }
    ];
    
    let mut vertex_input_attributes = Vec::new();
    for i in 0..offsets.len() {
      vertex_input_attributes.push(
        vk::VertexInputAttributeDescription {
          location: i as u32,
          binding: 0,
          format: vk::Format::R32G32B32A32_SFLOAT,
          offset: offsets[i] as u32,
        }
      );
    }
    
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
      vertex_attribute_description_count: vertex_input_attributes.len() as u32,
      p_vertex_attribute_descriptions: vertex_input_attributes.as_ptr(),
      vertex_binding_description_count: vertex_input_binding.len() as u32,
      p_vertex_binding_descriptions: vertex_input_binding.as_ptr(),
      ..Default::default()
    };
    
    let graphics_pipeline = graphics_pipeline_builder.build(device, 
                                                            &pipeline_layout, 
                                                            shader_stage_create_info.to_vec(),
                                                            vertex_input_state,
                                                            vertex_input_attributes,
                                                            viewport, scissors, renderpass);
    
    Shader {
      vertex_struct,
      vertex_shader,
      fragment_shader,
      pipeline_layout,
      graphics_pipeline,
    }
  }
  
  pub fn graphics_pipeline(&self) -> &GraphicsPipeline {
    &self.graphics_pipeline
  }
  
  pub fn pipeline_layout(&self) -> vk::PipelineLayout {
    self.pipeline_layout
  }
  
  pub fn destroy(&self, device: &VkDevice) {
    self.graphics_pipeline.destroy(device);
    
    unsafe {
      device.internal().destroy_pipeline_layout(self.pipeline_layout, None);
      device.internal().destroy_shader_module(self.vertex_shader, None);
      device.internal().destroy_shader_module(self.fragment_shader, None);
    }
  }
}












