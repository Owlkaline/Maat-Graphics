use vk;

use crate::modules::Device;
use crate::modules::DescriptorSet;
use crate::modules::RenderPass;
use crate::ownage::check_errors;

use std::mem;
use std::ptr;
use std::ffi::CString;

pub struct Pipeline {
  pipelines: Vec<vk::Pipeline>,
  cache: vk::PipelineCache,
  layout: vk::PipelineLayout,
}

impl Pipeline {
  pub fn new(device: &Device, vertex_shader: &vk::ShaderModule, fragment_shader: &vk::ShaderModule, render_pass: &RenderPass, swapchain_extent: &vk::Extent2D, swapchain_format: &vk::Format, descriptor_set: &DescriptorSet, vertex_binding: Vec<vk::VertexInputBindingDescription>, vertex_input_attribute_descriptions: Vec<vk::VertexInputAttributeDescription>) -> Pipeline {
    let mut pipelines: Vec<vk::Pipeline> = Vec::with_capacity(1);
    let mut layout: vk::PipelineLayout = unsafe { mem::uninitialized() };
    let mut cache: vk::PipelineCache = unsafe { mem::uninitialized() };
    
    let mut graphics_pipeline_create_infos: Vec<vk::GraphicsPipelineCreateInfo> = Vec::with_capacity(2);
    let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::with_capacity(2);
    let mut vertex_input_binding_descriptions: Vec<vk::VertexInputBindingDescription> = Vec::with_capacity(1);
    
    let topology = vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;
    let polygon_mode = vk::POLYGON_MODE_FILL;
    let enable_depth_clamp = vk::FALSE;
    let cull_mode =  vk::CULL_MODE_BACK_BIT;
    let front_face = vk::FRONT_FACE_CLOCKWISE;
    let depth_test = vk::FALSE;
    let depth_write = vk::FALSE;
    
    let blend_constants: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
    
    shader_stages.push(
      vk::PipelineShaderStageCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        stage: vk::SHADER_STAGE_VERTEX_BIT,
        module: *vertex_shader,
        pName: CString::new("main").unwrap().into_raw(),
        pSpecializationInfo: ptr::null(),
      }
    );
    
    shader_stages.push(
      vk::PipelineShaderStageCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        stage: vk::SHADER_STAGE_FRAGMENT_BIT,
        module: *fragment_shader,
        pName: CString::new("main").unwrap().into_raw(),
        pSpecializationInfo: ptr::null(),
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
    
    let pipeline_vertex_input_state_create_info = {
      vk::PipelineVertexInputStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        vertexBindingDescriptionCount: vertex_binding.len() as u32,
        pVertexBindingDescriptions: vertex_binding.as_ptr(),
        vertexAttributeDescriptionCount: vertex_input_attribute_descriptions.len() as u32,
        pVertexAttributeDescriptions: vertex_input_attribute_descriptions.as_ptr(),
      }
    };
    
    let pipeline_input_assembly_state_create_info = {
      vk::PipelineInputAssemblyStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        topology: topology,
        primitiveRestartEnable: vk::FALSE,
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
    
    let viewport = {
      vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swapchain_extent.width as f32,
        height: swapchain_extent.height as f32,
        minDepth: 0.0,
        maxDepth: 1.0,
      }
    };
    
    let scissor = {
      vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0,},
        extent: vk::Extent2D { width: swapchain_extent.width, height: swapchain_extent.height },
      }
    };
    
    let pipeline_viewport_state_create_info = {
      vk::PipelineViewportStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        viewportCount: 1,
        pViewports: &viewport,
        scissorCount: 1,
        pScissors: &scissor,
      }
    };
    
    let pipeline_rasterization_state_create_info = {
      vk::PipelineRasterizationStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        depthClampEnable: enable_depth_clamp,
        rasterizerDiscardEnable: vk::FALSE,
        polygonMode: polygon_mode,
        cullMode: cull_mode,
        frontFace: front_face,
        depthBiasEnable: vk::FALSE,
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
        rasterizationSamples: vk::SAMPLE_COUNT_1_BIT,
        sampleShadingEnable: vk::FALSE,
        minSampleShading: 1.0,
        pSampleMask: ptr::null(),
        alphaToCoverageEnable: vk::FALSE,
        alphaToOneEnable: vk::FALSE,
      }
    };
    
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
    };
    
    let pipeline_color_blend_attachments = {
      vk::PipelineColorBlendAttachmentState {
        blendEnable: vk::FALSE,
        srcColorBlendFactor: vk::BLEND_FACTOR_ONE,
        dstColorBlendFactor: vk::BLEND_FACTOR_ZERO,
        colorBlendOp: vk::BLEND_OP_ADD,
        srcAlphaBlendFactor: vk::BLEND_FACTOR_ONE,
        dstAlphaBlendFactor: vk::BLEND_FACTOR_ZERO,
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
        blendConstants: blend_constants,
      }
    };
    
    let dynamic_state_create_info = {
      vk::PipelineDynamicStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        dynamicStateCount: 2,
        pDynamicStates: vec!(vk::DYNAMIC_STATE_VIEWPORT, vk::DYNAMIC_STATE_LINE_WIDTH).as_ptr(),
      }
    };
    
    let push_constant_range = {
      vk::PushConstantRange {
        stageFlags: vk::SHADER_STAGE_VERTEX_BIT,
        offset: 0,
        size: 0,
      }
    };
    
    let pipeline_layout_create_info = {
      vk::PipelineLayoutCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        setLayoutCount: 1,
        pSetLayouts: descriptor_set.layouts().as_ptr(),
        pushConstantRangeCount: 0,
        pPushConstantRanges: ptr::null(),//&push_constant_range,
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
        pDynamicState: ptr::null(),//&dynamic_state_create_info,
        layout: layout,
        renderPass: *render_pass.internal_object(),
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
    
    Pipeline {
      pipelines,
      cache,
      layout,
    }
  }
  
  pub fn pipeline(&self, index: usize) -> &vk::Pipeline {
    &self.pipelines[index]
  }
  
  pub fn layout(&self) -> &vk::PipelineLayout {
    &self.layout
  }
  
  pub fn destroy(&self, device: &Device) {
    let vk = device.pointers();
    let device = device.internal_object();
    
    println!("Destroying Pipeline");
    
    unsafe {
      vk.DestroyPipelineLayout(*device, self.layout, ptr::null());
      vk.DestroyPipelineCache(*device, self.cache, ptr::null());
      
      for pipeline in &self.pipelines {
        vk.DestroyPipeline(*device, *pipeline, ptr::null());
      }
    }
  }
}
