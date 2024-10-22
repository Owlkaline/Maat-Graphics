use ash::vk;

use crate::vkwrapper::{Renderpass, Scissors, Viewport, VkDevice};

pub struct GraphicsPipeline {
  pipeline: vk::Pipeline,
}

impl GraphicsPipeline {
  pub fn new(pipeline: vk::Pipeline) -> GraphicsPipeline {
    GraphicsPipeline { pipeline }
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

pub struct GraphicsPipelineBuilder {
  topology: vk::PrimitiveTopology,
  front_face: vk::FrontFace,
  polygon_mode: vk::PolygonMode,
  samples: vk::SampleCountFlags,
  cull_mode: vk::CullModeFlags,
}

impl GraphicsPipelineBuilder {
  pub fn new() -> GraphicsPipelineBuilder {
    let topology: vk::PrimitiveTopology = Default::default();
    let front_face: vk::FrontFace = Default::default();
    let polygon_mode: vk::PolygonMode = Default::default();
    let samples: vk::SampleCountFlags = Default::default();
    let cull_mode: vk::CullModeFlags = Default::default();

    GraphicsPipelineBuilder {
      topology,
      front_face,
      polygon_mode,
      samples,
      cull_mode,
    }
  }

  pub fn topology_point_list(mut self) -> GraphicsPipelineBuilder {
    self.topology = vk::PrimitiveTopology::POINT_LIST;
    self
  }

  pub fn topology_line_list(mut self) -> GraphicsPipelineBuilder {
    self.topology = vk::PrimitiveTopology::LINE_LIST;
    self
  }

  pub fn topology_line_strip(mut self) -> GraphicsPipelineBuilder {
    self.topology = vk::PrimitiveTopology::LINE_STRIP;
    self
  }

  pub fn topology_triangle_list(mut self) -> GraphicsPipelineBuilder {
    self.topology = vk::PrimitiveTopology::TRIANGLE_LIST;
    self
  }

  pub fn topology_triangle_strip(mut self) -> GraphicsPipelineBuilder {
    self.topology = vk::PrimitiveTopology::TRIANGLE_STRIP;
    self
  }

  pub fn topology_triangle_fan(mut self) -> GraphicsPipelineBuilder {
    self.topology = vk::PrimitiveTopology::TRIANGLE_FAN;
    self
  }

  pub fn front_face_counter_clockwise(mut self) -> GraphicsPipelineBuilder {
    self.front_face = vk::FrontFace::COUNTER_CLOCKWISE;
    self
  }

  pub fn front_face_clockwise(mut self) -> GraphicsPipelineBuilder {
    self.front_face = vk::FrontFace::CLOCKWISE;
    self
  }

  pub fn cull_none(mut self) -> GraphicsPipelineBuilder {
    self.cull_mode = vk::CullModeFlags::NONE;
    self
  }

  pub fn cull_front(mut self) -> GraphicsPipelineBuilder {
    self.cull_mode = vk::CullModeFlags::FRONT;
    self
  }

  pub fn cull_back(mut self) -> GraphicsPipelineBuilder {
    self.cull_mode = vk::CullModeFlags::BACK;
    self
  }

  pub fn cull_front_and_back(mut self) -> GraphicsPipelineBuilder {
    self.cull_mode = vk::CullModeFlags::FRONT_AND_BACK;
    self
  }

  pub fn polygon_mode_fill(mut self) -> GraphicsPipelineBuilder {
    self.polygon_mode = vk::PolygonMode::FILL;
    self
  }

  pub fn polygon_mode_line(mut self) -> GraphicsPipelineBuilder {
    self.polygon_mode = vk::PolygonMode::LINE;
    self
  }

  pub fn polygon_mode_point(mut self) -> GraphicsPipelineBuilder {
    self.polygon_mode = vk::PolygonMode::POINT;
    self
  }

  pub fn samples_1(mut self) -> GraphicsPipelineBuilder {
    self.samples = vk::SampleCountFlags::TYPE_1;
    self
  }

  pub fn samples_2(mut self) -> GraphicsPipelineBuilder {
    self.samples = vk::SampleCountFlags::TYPE_2;
    self
  }

  pub fn samples_4(mut self) -> GraphicsPipelineBuilder {
    self.samples = vk::SampleCountFlags::TYPE_4;
    self
  }

  pub fn samples_8(mut self) -> GraphicsPipelineBuilder {
    self.samples = vk::SampleCountFlags::TYPE_8;
    self
  }

  pub fn samples_16(mut self) -> GraphicsPipelineBuilder {
    self.samples = vk::SampleCountFlags::TYPE_16;
    self
  }

  pub fn samples_32(mut self) -> GraphicsPipelineBuilder {
    self.samples = vk::SampleCountFlags::TYPE_32;
    self
  }

  pub fn samples_64(mut self) -> GraphicsPipelineBuilder {
    self.samples = vk::SampleCountFlags::TYPE_64;
    self
  }

  pub fn build(
    &self,
    device: &VkDevice,
    pipeline_layout: &vk::PipelineLayout,
    shader_stage_create_info: Vec<vk::PipelineShaderStageCreateInfo>,
    vertex_input_state: vk::PipelineVertexInputStateCreateInfo,
    //vertex_input_attributes: Vec<vk::VertexInputAttributeDescription>,
    viewport: &Viewport,
    scissors: &Scissors,
    renderpass: &Renderpass,
  ) -> GraphicsPipeline {
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
      topology: self.topology,
      ..Default::default()
    };

    let scissors = scissors.build();
    let viewport = [viewport.build()];
    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
      .scissors(&scissors)
      .viewports(&viewport);

    let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
      .front_face(self.front_face)
      .polygon_mode(self.polygon_mode)
      .cull_mode(self.cull_mode)
      .line_width(1.0);

    let multisample_state_info =
      vk::PipelineMultisampleStateCreateInfo::builder().rasterization_samples(self.samples);
    let noop_stencil_state_front = vk::StencilOpState::builder()
      .fail_op(vk::StencilOp::KEEP)
      .pass_op(vk::StencilOp::KEEP)
      .depth_fail_op(vk::StencilOp::KEEP)
      .compare_op(vk::CompareOp::ALWAYS);
    let noop_stencil_state_back = vk::StencilOpState::builder()
      .fail_op(vk::StencilOp::KEEP)
      .pass_op(vk::StencilOp::KEEP)
      .depth_fail_op(vk::StencilOp::KEEP)
      .compare_op(vk::CompareOp::ALWAYS);

    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::builder()
      .depth_test_enable(true)
      .depth_write_enable(true)
      .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
      .front(*noop_stencil_state_front)
      .back(*noop_stencil_state_back)
      .max_depth_bounds(1.0);

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
      blend_enable: vk::TRUE,
      src_color_blend_factor: vk::BlendFactor::SRC_ALPHA, //vk::BlendFactor::SRC_COLOR,
      dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA, //vk::BlendFactor::ONE_MINUS_DST_COLOR,
      color_blend_op: vk::BlendOp::ADD,
      src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA, //vk::BlendFactor::ZERO,
      dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
      alpha_blend_op: vk::BlendOp::SUBTRACT,
      color_write_mask: vk::ColorComponentFlags::R
        | vk::ColorComponentFlags::G
        | vk::ColorComponentFlags::B
        | vk::ColorComponentFlags::A,
    }];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
      .logic_op(vk::LogicOp::CLEAR)
      .attachments(&color_blend_attachment_states);

    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info =
      vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

    let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
      .stages(&shader_stage_create_info)
      .vertex_input_state(&vertex_input_state)
      .input_assembly_state(&vertex_input_assembly_state_info)
      .viewport_state(&viewport_state_info)
      .rasterization_state(&rasterization_info)
      .multisample_state(&multisample_state_info)
      .depth_stencil_state(&depth_state_info)
      .color_blend_state(&color_blend_state)
      .dynamic_state(&dynamic_state_info)
      .layout(*pipeline_layout)
      .render_pass(renderpass.internal());

    let graphics_pipelines = unsafe {
      device
        .internal()
        .create_graphics_pipelines(vk::PipelineCache::null(), &[*graphic_pipeline_info], None)
        .expect("Unable to create graphics pipeline")
    };

    GraphicsPipeline::new(graphics_pipelines[0])
  }
}
