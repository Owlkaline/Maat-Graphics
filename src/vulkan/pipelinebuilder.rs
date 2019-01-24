use vk;

use crate::vulkan::Device;
use crate::vulkan::Pipeline;
use crate::vulkan::PipelineInfo;
use crate::vulkan::RenderPass;
use crate::vulkan::buffer::{UniformData};
use crate::vulkan::ownage::check_errors;
use crate::vulkan::vkenums::{BlendFactor, Topology, PolygonMode, CullMode, FrontFace, Sample, VkBool, ShaderStageFlagBits};

use std::mem;
use std::ptr;
use std::sync::Arc;
use std::ffi::CString;

pub struct PipelineBuilder {
  vertex_shader: Option<vk::ShaderModule>,
  fragment_shader: Option<vk::ShaderModule>,
  render_pass: Option<RenderPass>,
  topology: Topology,
  polygon_mode: PolygonMode,
  cull_mode: CullMode,
  front_face: FrontFace,
  depth_test: u32,
  depth_write: u32,
  depth_clamp: u32,
  depth_bias: u32,
  rasterizer_discard: u32,
  blend_constants: [f32; 4],
  primitive_restart: u32,
  rasterization_samples: Sample,
  sample_shader: u32,
  alpha_to_coverage: u32,
  alpha_to_one: u32,
  has_push_constant: bool,
  push_constant_size: u32,
  push_constant_shader_stage: ShaderStageFlagBits,
  specialisation_constants: Vec<(u32, UniformData, u32, ShaderStageFlagBits)>, //Vec<(id, data, offset, shader stage)>
  descriptor_set_layouts: Option<Vec<vk::DescriptorSetLayout>>,
  vertex_binding: Option<Vec<vk::VertexInputBindingDescription>>,
  vertex_attributes: Option<Vec<vk::VertexInputAttributeDescription>>,
}

impl PipelineBuilder {
  pub fn new() -> PipelineBuilder {
    PipelineBuilder {
      vertex_shader: None,
      fragment_shader: None,
      render_pass: None,
      topology: Topology::TriangleList,
      polygon_mode: PolygonMode::Fill,
      cull_mode: CullMode::Back,
      front_face: FrontFace::Clockwise,
      depth_test: vk::FALSE,
      depth_write: vk::FALSE,
      depth_clamp: vk::FALSE,
      depth_bias: vk::FALSE,
      rasterizer_discard: vk::FALSE,
      blend_constants: [0.0, 0.0, 0.0, 0.0],
      primitive_restart: vk::FALSE,
      rasterization_samples: Sample::Count1Bit,
      sample_shader: vk::FALSE,
      alpha_to_coverage: vk::FALSE,
      alpha_to_one: vk::FALSE,
      has_push_constant: false,
      push_constant_size: 0,
      push_constant_shader_stage: ShaderStageFlagBits::Vertex,
      specialisation_constants: Vec::new(),
      descriptor_set_layouts: None,
      vertex_binding: None,
      vertex_attributes: None,
    }
  }
  
  pub fn push_constants(mut self, shader_stage: ShaderStageFlagBits, size: u32) -> PipelineBuilder {
    self.has_push_constant = true;
    self.push_constant_size = size;
    self.push_constant_shader_stage = shader_stage;
    
    self
  }
  
  pub fn vertex_shader(mut self, shader: vk::ShaderModule) -> PipelineBuilder {
    self.vertex_shader = Some(shader);
    self
  }
  
  pub fn fragment_shader(mut self, shader: vk::ShaderModule) -> PipelineBuilder {
    self.fragment_shader = Some(shader);
    self
  }
  
  pub fn add_vertex_specialisation_constant(mut self, id: u32, data: UniformData, offset: u32) -> PipelineBuilder {
    self.specialisation_constants.push((id, data, offset, ShaderStageFlagBits::Vertex));
    self
  }
  
  pub fn add_fragment_specialisation_constant(mut self, id: u32, data: UniformData, offset: u32) -> PipelineBuilder {
    self.specialisation_constants.push((id, data, offset, ShaderStageFlagBits::Fragment));
    self
  }
  
  pub fn vertex_binding(mut self, binding: Vec<vk::VertexInputBindingDescription>) -> PipelineBuilder {
    self.vertex_binding = Some(binding);
    self
  }
  
  pub fn vertex_attributes(mut self, attributes: Vec<vk::VertexInputAttributeDescription>) -> PipelineBuilder {
    self.vertex_attributes = Some(attributes);
    self
  }
  
  pub fn render_pass(mut self, render_pass: RenderPass) -> PipelineBuilder {
    self.render_pass = Some(render_pass);
    self
  }
  
  pub fn descriptor_set_layout(mut self, layouts: Vec<vk::DescriptorSetLayout>) -> PipelineBuilder {
    self.descriptor_set_layouts = Some(layouts);
    self
  }
  
  pub fn topology_point_list(mut self) -> PipelineBuilder {
    self.topology = Topology::PointList;
    self
  }
  
  pub fn topology_line_list(mut self) -> PipelineBuilder {
    self.topology = Topology::LineList;
    self
  }
  
  pub fn topology_line_strip(mut self) -> PipelineBuilder {
    self.topology = Topology::LineStrip;
    self
  }
  
  pub fn topology_triangle_list(mut self) -> PipelineBuilder {
    self.topology = Topology::TriangleList;
    self
  }
  
  pub fn topology_triangle_strip(mut self) -> PipelineBuilder {
    self.topology = Topology::TriangleStrip;
    self
  }
  
  pub fn topology_triangle_fan(mut self) -> PipelineBuilder {
    self.topology = Topology::TriangleFan;
    self
  }
  
  pub fn topology_line_list_with_adjacency(mut self) -> PipelineBuilder {
    self.topology = Topology::LineListWithAdjacency;
    self
  }
  
  pub fn topology_line_strip_with_adjacency(mut self) -> PipelineBuilder {
    self.topology = Topology::LineStripWithAdjacency;
    self
  }
  
  pub fn topology_triangle_list_with_adjacency(mut self) -> PipelineBuilder {
    self.topology = Topology::TriangleListWithAdjacency;
    self
  }
  
  pub fn topology_triangle_strip_with_adjacency(mut self) -> PipelineBuilder {
    self.topology = Topology::TriangleStripWithAjacency;
    self
  }
  
  pub fn topology_patch_list(mut self) -> PipelineBuilder {
    self.topology = Topology::PatchList;
    self
  }
  
  pub fn polygon_mode_fill(mut self) -> PipelineBuilder {
    self.polygon_mode = PolygonMode::Fill;
    self
  }
  
  pub fn polygon_mode_line(mut self) -> PipelineBuilder {
    self.polygon_mode = PolygonMode::Line;
    self
  }
  
  pub fn polygon_mode_point(mut self) -> PipelineBuilder {
    self.polygon_mode = PolygonMode::Point;
    self
  }
  
  pub fn cull_mode_none(mut self) -> PipelineBuilder {
    self.cull_mode = CullMode::None;
    self
  }
  
  pub fn cull_mode_font(mut self) -> PipelineBuilder {
    self.cull_mode = CullMode::Front;
    self
  }
  
  pub fn cull_mode_back(mut self) -> PipelineBuilder {
    self.cull_mode = CullMode::Back;
    self
  }
  
  pub fn cull_mode_front_and_back(mut self) -> PipelineBuilder {
    self.cull_mode = CullMode::FrontAndBack;
    self
  }
  
  pub fn front_face_clockwise(mut self) -> PipelineBuilder {
    self.front_face = FrontFace::Clockwise;
    self
  }
  
  pub fn front_face_counter_clockwise(mut self) -> PipelineBuilder {
    self.front_face = FrontFace::CounterClockwise;
    self
  }
  
  pub fn enable_depth_test(mut self) -> PipelineBuilder {
    self.depth_test = vk::TRUE;
    self
  }
  
  pub fn enable_depth_write(mut self) -> PipelineBuilder {
    self.depth_write = vk::TRUE;
    self
  }
  
  pub fn enable_depth_clamp(mut self) -> PipelineBuilder {
    self.depth_clamp = vk::TRUE;
    self
  }
  
  pub fn enable_depth_bias(mut self) -> PipelineBuilder {
    self.depth_bias = vk::TRUE;
    self
  }
  
  pub fn discard_rasterizer(mut self) -> PipelineBuilder {
    self.rasterizer_discard = vk::TRUE;
    self
  }
  
  pub fn primitive_restart(mut self) -> PipelineBuilder {
    self.primitive_restart = vk::TRUE;
    self
  }
  
  pub fn rasterization_samples_1_bit(mut self) -> PipelineBuilder {
    self.rasterization_samples = Sample::Count1Bit;
    self
  }
  
  pub fn rasterization_samples_2_bit(mut self) -> PipelineBuilder {
    self.rasterization_samples = Sample::Count2Bit;
    self
  }
  
  pub fn rasterization_samples_4_bit(mut self) -> PipelineBuilder {
    self.rasterization_samples = Sample::Count4Bit;
    self
  }
  
  pub fn rasterization_samples_8_bit(mut self) -> PipelineBuilder {
    self.rasterization_samples = Sample::Count8Bit;
    self
  }
  
  pub fn rasterization_samples_16_bit(mut self) -> PipelineBuilder {
    self.rasterization_samples = Sample::Count16Bit;
    self
  }
  
  pub fn sample_shader(mut self) -> PipelineBuilder {
    self.sample_shader = vk::TRUE;
    self
  }
  
  pub fn alpha_to_coverage(mut self) -> PipelineBuilder {
    self.alpha_to_coverage = vk::TRUE;
    self
  }
  
  pub fn alpha_to_one(mut self) -> PipelineBuilder {
    self.alpha_to_one = vk::TRUE;
    self
  }
  
  pub fn build(mut self, device: Arc<Device>) -> Pipeline {
    if !self.vertex_shader.is_some() {
      panic!("PipelineBuilder Error: vertex shader missing!");
    }
    
    if !self.fragment_shader.is_some() {
      panic!("PipelineBuilder Error: fragment shader missing!");
    }
    
    if !self.render_pass.is_some() {
      panic!("PipelineBuilder Error: render_pass missing!");
    }
    
    if !self.descriptor_set_layouts.is_some() {
      panic!("PipelineBuilder Error: descriptor_set_layout missing!");
    }
    
    if !self.vertex_binding.is_some() {
      panic!("PipelineBuilder Error: vertex bindings missing!");
    }
    
    if !self.vertex_attributes.is_some() {
      panic!("PipelineBuilder Error: vertex attributes missing!");
    }
    
    let mut pipelines: Vec<vk::Pipeline> = Vec::with_capacity(1);
    let mut layout: vk::PipelineLayout = unsafe { mem::uninitialized() };
    let mut cache: vk::PipelineCache = unsafe { mem::uninitialized() };
    
    let mut graphics_pipeline_create_infos: Vec<vk::GraphicsPipelineCreateInfo> = Vec::with_capacity(2);
    let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::with_capacity(2);
    
    let vertex_specialisation_constants: vk::SpecializationInfo;
    let fragment_specialisation_constants: vk::SpecializationInfo;
    
    let mut vertex_specialisation_map_entry: Vec<vk::SpecializationMapEntry> = Vec::new();
    let mut fragment_specialisation_map_entry: Vec<vk::SpecializationMapEntry> = Vec::new();
    let mut vertex_specialisation_data: UniformData = UniformData::new();
    let mut fragment_specialisation_data: UniformData = UniformData::new();
    
    for (id, data, offset, shader_stage) in &mut self.specialisation_constants {
      match shader_stage {
        ShaderStageFlagBits::Vertex => {
          vertex_specialisation_map_entry.push(
            vk::SpecializationMapEntry {
              constantID: *id,
              offset: *offset,
              size: data.size_non_aligned() as usize,
            }
          );
          let raw_data = data.build_non_aligned();
          for float in raw_data.iter() {
            vertex_specialisation_data = vertex_specialisation_data.add_float(*float);
          }
        },
        ShaderStageFlagBits::Fragment => {
          fragment_specialisation_map_entry.push(
            vk::SpecializationMapEntry {
              constantID: *id,
              offset: *offset,
              size: data.size_non_aligned() as usize,
            }
          );
          let raw_data = data.build_non_aligned();
          for float in raw_data.iter() {
            fragment_specialisation_data = fragment_specialisation_data.add_float(*float);
          }
        },
        _ => {}
      }
    }
    
    vertex_specialisation_constants = vk::SpecializationInfo {
                                        mapEntryCount: vertex_specialisation_map_entry.len() as u32,
                                        pMapEntries: vertex_specialisation_map_entry.as_ptr(),
                                        dataSize: vertex_specialisation_data.size_non_aligned() as usize,
                                        pData: vertex_specialisation_data.build_non_aligned().as_ptr() as *const _,
                                      };
                                      
    fragment_specialisation_constants = vk::SpecializationInfo {
                                        mapEntryCount: fragment_specialisation_map_entry.len() as u32,
                                        pMapEntries: fragment_specialisation_map_entry.as_ptr(),
                                        dataSize: fragment_specialisation_data.size_non_aligned() as usize,
                                        pData: fragment_specialisation_data.build_non_aligned().as_ptr() as *const _,
                                      };
    
    shader_stages.push(
      vk::PipelineShaderStageCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        stage: vk::SHADER_STAGE_VERTEX_BIT,
        module: self.vertex_shader.unwrap(),
        pName: CString::new("main").unwrap().into_raw(),
        pSpecializationInfo: if vertex_specialisation_map_entry.len() == 0 { ptr::null() } else { &vertex_specialisation_constants },
      }
    );
    
    shader_stages.push(
      vk::PipelineShaderStageCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        stage: vk::SHADER_STAGE_FRAGMENT_BIT,
        module: self.fragment_shader.unwrap(),
        pName: CString::new("main").unwrap().into_raw(),
        pSpecializationInfo: if fragment_specialisation_map_entry.len() == 0 { ptr::null() } else { &fragment_specialisation_constants },
      }
    );
    
    /*
    float: VK_FORMAT_R32_SFLOAT
    vec2: VK_FORMAT_R32G32_SFLOAT
    vec3: VK_FORMAT_R32G32B32_SFLOAT
    vec4: VK_FORMAT_R32G32B32A32_SFLOAT
    ivec2: VK_FORMAT_R32G32_SINT
    uvec4: VK_FORMAT_R32G32B32A32_UINT
    double: VK_FORMAT_R64_SFLOAT
    */

    let vertex_binding = self.vertex_binding.unwrap();
    let vertex_attributes = self.vertex_attributes.unwrap();
    let pipeline_vertex_input_state_create_info = {
      vk::PipelineVertexInputStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        vertexBindingDescriptionCount: vertex_binding.len() as u32,
        pVertexBindingDescriptions: vertex_binding.as_ptr(),
        vertexAttributeDescriptionCount: vertex_attributes.len() as u32,
        pVertexAttributeDescriptions: vertex_attributes.as_ptr(),
      }
    };
    
    let pipeline_input_assembly_state_create_info = {
      vk::PipelineInputAssemblyStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        topology: self.topology.to_bits(),
        primitiveRestartEnable: self.primitive_restart,
      }
    };
    
    let pipeline_tessellation_state_create_info = {
      vk::PipelineTessellationStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        patchControlPoints: 0,
      }
    };
    
    let pipeline_viewport_state_create_info = {
      vk::PipelineViewportStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        viewportCount: 1,
        pViewports: ptr::null(),//&viewport,
        scissorCount: 1,
        pScissors: ptr::null(),//&scissor,
      }
    };
    
    let pipeline_rasterization_state_create_info = {
      vk::PipelineRasterizationStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        depthClampEnable: self.depth_clamp,
        rasterizerDiscardEnable: self.rasterizer_discard,
        polygonMode: self.polygon_mode.to_bits(),
        cullMode: self.cull_mode.to_bits(),
        frontFace: self.front_face.to_bits(),
        depthBiasEnable: self.depth_bias,
        depthBiasConstantFactor: 0.0,
        depthBiasClamp: 0.0,
        depthBiasSlopeFactor: 0.0,
        lineWidth: 1.0,
      }
    };
    
    let pipeline_multisample_state_create_info = {
      vk::PipelineMultisampleStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        rasterizationSamples: self.rasterization_samples.to_bits(),
        sampleShadingEnable: self.sample_shader,
        minSampleShading: 1.0,
        pSampleMask: ptr::null(),
        alphaToCoverageEnable: self.alpha_to_coverage,
        alphaToOneEnable: self.alpha_to_one,
      }
    };
    /*
    let front_stencil_op_state = {
      vk::StencilOpState {
        failOp: vk::STENCIL_OP_KEEP,
        passOp: vk::STENCIL_OP_KEEP,
        depthFailOp: vk::STENCIL_OP_KEEP,
        compareOp: vk::COMPARE_OP_NEVER,
        compareMask: 0,
        writeMask: 0,
        reference: 0,
      }
    };
    
    let back_stencil_op_state = {
      vk::StencilOpState {
        failOp: vk::STENCIL_OP_KEEP,
        passOp: vk::STENCIL_OP_KEEP,
        depthFailOp: vk::STENCIL_OP_KEEP,
        compareOp: vk::COMPARE_OP_NEVER,
        compareMask: 0,
        writeMask: 0,
        reference: 0,
      }
    };
    
    let pipeline_depth_stencil_state_create_info = {
      vk::PipelineDepthStencilStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        depthTestEnable: depth_test,
        depthWriteEnable: depth_write,
        depthCompareOp: vk::COMPARE_OP_LESS_OR_EQUAL,
        depthBoundsTestEnable: vk::FALSE,
        stencilTestEnable: vk::FALSE,
        front: front_stencil_op_state,
        back: back_stencil_op_state,
        minDepthBounds: 0.0,
        maxDepthBounds: 1.0,
      }
    };*/
    
    let pipeline_color_blend_attachments = {
      vk::PipelineColorBlendAttachmentState {
        blendEnable: VkBool::True.to_bits(),
        srcColorBlendFactor: BlendFactor::SrcAlpha.to_bits(),
        dstColorBlendFactor: BlendFactor::OneMinusSrcAlpha.to_bits(),
        colorBlendOp: vk::BLEND_OP_ADD,
        srcAlphaBlendFactor: BlendFactor::One.to_bits(),
        dstAlphaBlendFactor: BlendFactor::Zero.to_bits(),
        alphaBlendOp: vk::BLEND_OP_ADD,
        colorWriteMask: vk::COLOR_COMPONENT_R_BIT | vk::COLOR_COMPONENT_G_BIT | vk::COLOR_COMPONENT_B_BIT | vk::COLOR_COMPONENT_A_BIT,
      }
    };
    
    let pipeline_colour_blend_state_create_info = {
      vk::PipelineColorBlendStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        logicOpEnable: vk::FALSE,
        logicOp: vk::LOGIC_OP_COPY,
        attachmentCount: 1,
        pAttachments: &pipeline_color_blend_attachments,
        blendConstants: self.blend_constants,
      }
    };
    
    let mut dynamic_states = Vec::with_capacity(3);
    dynamic_states.push(vk::DYNAMIC_STATE_VIEWPORT);
    dynamic_states.push(vk::DYNAMIC_STATE_SCISSOR);
    dynamic_states.push(vk::DYNAMIC_STATE_LINE_WIDTH);
    
    let dynamic_state_create_info = {
      vk::PipelineDynamicStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        dynamicStateCount: dynamic_states.len() as u32,
        pDynamicStates: dynamic_states.as_ptr(),
      }
    };
    
    let push_constant_range = {
      vk::PushConstantRange {
        stageFlags: self.push_constant_shader_stage.to_bits(),
        offset: 0,
        size: self.push_constant_size,
      }
    };
    
    let layouts = self.descriptor_set_layouts.unwrap();
    let pipeline_layout_create_info = {
      vk::PipelineLayoutCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        setLayoutCount: layouts.len() as u32,
        pSetLayouts: layouts.as_ptr(),
        pushConstantRangeCount: 1,
        pPushConstantRanges: &push_constant_range,
      }
    };
    
    let vk = device.pointers();
    let device = device.internal_object();
    
    unsafe {
      vk.CreatePipelineLayout(*device, &pipeline_layout_create_info, ptr::null(), &mut layout);
    }
    
    graphics_pipeline_create_infos.push(
      vk::GraphicsPipelineCreateInfo {
        sType: vk::STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        stageCount: shader_stages.len() as u32,
        pStages: shader_stages.as_ptr(),
        pVertexInputState: &pipeline_vertex_input_state_create_info,
        pInputAssemblyState: &pipeline_input_assembly_state_create_info,
        pTessellationState: &pipeline_tessellation_state_create_info,
        pViewportState: &pipeline_viewport_state_create_info,
        pRasterizationState: &pipeline_rasterization_state_create_info,
        pMultisampleState: &pipeline_multisample_state_create_info,
        pDepthStencilState: ptr::null(),//&pipeline_depth_stencil_state_create_info,
        pColorBlendState: &pipeline_colour_blend_state_create_info,
        pDynamicState: &dynamic_state_create_info,
        layout: layout,
        renderPass: *self.render_pass.unwrap().internal_object(),
        subpass: 0,
        basePipelineHandle: 0,
        basePipelineIndex: -1,
      }
    );
    
    let pipeline_cache_create_info = {
      vk::PipelineCacheCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        initialDataSize: 0,
        pInitialData: ptr::null(),
      }
    };
    
    unsafe {
      check_errors(vk.CreatePipelineCache(*device, &pipeline_cache_create_info, ptr::null(), &mut cache));
      check_errors(vk.CreateGraphicsPipelines(*device, cache, graphics_pipeline_create_infos.len() as u32, graphics_pipeline_create_infos.as_ptr(), ptr::null(), pipelines.as_mut_ptr()));
      pipelines.set_len(graphics_pipeline_create_infos.len());
    }
    
    let pipeline_info = PipelineInfo {
      vertex_shader: self.vertex_shader.unwrap(),
      fragment_shader: self.fragment_shader.unwrap(),
      vertex_binding: vertex_binding,
      vertex_input_attribute_descriptions: vertex_attributes,
    };
    
    Pipeline::new_with_fields(pipeline_info, pipelines, cache, layout)
  }
}
