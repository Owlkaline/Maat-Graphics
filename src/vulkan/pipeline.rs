use vk;

use crate::vulkan::Device;

use std::ptr;
use std::sync::Arc;

pub struct PipelineInfo {
  pub vertex_shader: vk::ShaderModule,
  pub fragment_shader: vk::ShaderModule,
  pub vertex_binding: Vec<vk::VertexInputBindingDescription>,
  pub vertex_input_attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
}

pub struct Pipeline {
  info: PipelineInfo,
  pipelines: Vec<vk::Pipeline>,
  cache: vk::PipelineCache,
  layout: vk::PipelineLayout,
}

impl Pipeline {
  pub fn new_with_fields(pipeline_info: PipelineInfo, pipelines: Vec<vk::Pipeline>, cache: vk::PipelineCache, layout: vk::PipelineLayout) -> Pipeline {
    Pipeline {
      info: pipeline_info,
      pipelines,
      cache,
      layout,
    }
  }
  
  pub fn pipeline(&self, index: usize) -> &vk::Pipeline {
    &self.pipelines[index]
  }
  
  pub fn pipelines(&self) -> &Vec<vk::Pipeline> {
    &self.pipelines
  }
  
  pub fn cache(&self) -> &vk::PipelineCache {
    &self.cache
  }
  
  pub fn layout(&self) -> &vk::PipelineLayout {
    &self.layout
  }
  
  pub fn shaders(&self) -> (&vk::ShaderModule, &vk::ShaderModule) {
    (&self.info.vertex_shader, &self.info.fragment_shader)
  }
  
  pub fn destroy(&self, device: Arc<Device>) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    unsafe {
      vk.DestroyPipelineLayout(*device, self.layout, ptr::null());
      vk.DestroyPipelineCache(*device, self.cache, ptr::null());
      
      for pipeline in &self.pipelines {
        vk.DestroyPipeline(*device, *pipeline, ptr::null());
      }
    }
  }
}
