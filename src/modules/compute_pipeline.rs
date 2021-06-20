use ash::vk;

use ash::version::DeviceV1_0;

use crate::modules::{VkDevice, Viewport, Scissors, Renderpass};

pub struct ComputePipeline {
  pipeline: vk::Pipeline,
}

impl ComputePipeline {
  pub fn new(device: &VkDevice, pipeline_layout: &vk::PipelineLayout, 
             shader_stage_create_info: vk::PipelineShaderStageCreateInfo) -> ComputePipeline {
    
    let compute_pipeline_info = vk::ComputePipelineCreateInfo::builder()
                                    .stage(shader_stage_create_info)
                                    .layout(*pipeline_layout);
    
    let compute_pipelines = unsafe {
      device.internal()
      .create_compute_pipelines(
        vk::PipelineCache::null(),
        &[compute_pipeline_info.build()],
        None,
      )
      .expect("Unable to create graphics pipeline")
    };
    
    ComputePipeline {
      pipeline: compute_pipelines[0],
    }
  }
  
  pub fn internal(&self) -> &vk::Pipeline {
    &self.pipeline
  }
  
  pub fn destroy(&self, device: &VkDevice) {
    unsafe {
      device.internal().destroy_pipeline(self.pipeline, None);
    }
  }
}
