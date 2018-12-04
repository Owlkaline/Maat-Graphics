/*
VkResult vkAcquireNextImageKHR(
    VkDevice                                    device,
    VkSwapchainKHR                              swapchain,
    uint64_t                                    timeout,
    VkSemaphore                                 semaphore,
    VkFence                                     fence,
    uint32_t*                                   pImageIndex);
*/

use vk;
use winit;

use modules::VkWindow;
use ownage::check_errors;

use std::ptr;
use std::mem;
use std::ffi::c_void;
use std::ffi::CString;

pub struct Vulkan {
  window: VkWindow,
  fence: vk::Fence,
  semaphore_image_available: vk::Semaphore,
  semaphore_render_finished: vk::Semaphore,
  command_pool: vk::CommandPool,
  command_buffers: Vec<vk::CommandBuffer>,
  render_pass: vk::RenderPass,
  framebuffers: Vec<vk::Framebuffer>,
  vertex_shader: vk::ShaderModule,
  fragment_shader: vk::ShaderModule,
  descriptor_set_layout: vk::DescriptorSetLayout,
  descriptor_set_pool: vk::DescriptorPool,
  descriptor_sets: Vec<vk::DescriptorSet>,
  pipelines: Vec<vk::Pipeline>,
  pipeline_cache: vk::PipelineCache,
  pipeline_layout: vk::PipelineLayout,
}

impl Vulkan {
  pub fn new(app_name: String, app_version: u32, width: f32, height: f32, should_debug: bool) -> Vulkan {
    let window = VkWindow::new(app_name, app_version, width, height, should_debug);
    
    let fence: vk::Fence;
    let semaphore_image_available: vk::Semaphore;
    let semaphore_render_finished: vk::Semaphore;
    let command_pool: vk::CommandPool;
    let command_buffers: Vec<vk::CommandBuffer>;
    let render_pass: vk::RenderPass;
    let framebuffers: Vec<vk::Framebuffer>;
    let vertex_shader: vk::ShaderModule;
    let fragment_shader: vk::ShaderModule;
    let descriptor_set_layout: vk::DescriptorSetLayout;
    let descriptor_set_pool: vk::DescriptorPool;
    let descriptor_sets: Vec<vk::DescriptorSet>;
    let pipelines: Vec<vk::Pipeline>;
    let pipeline_cache: vk::PipelineCache;
    let pipeline_layout: vk::PipelineLayout;
    
    {
      let vk = window.device_pointers();
      let device = window.device();
      let format = window.swapchain_format();
      let graphics_family = window.get_graphics_family();
      let current_extent = window.get_current_extent();
      let image_views = window.swapchain_image_views();
      
      fence = Vulkan::create_fence(vk, device);
      let (semaphore1, semaphore2) = Vulkan::create_semaphores(vk, device);
      semaphore_image_available = semaphore1;
      semaphore_render_finished = semaphore2;
      command_pool = Vulkan::create_command_pool(vk, device, graphics_family);
      command_buffers = Vulkan::create_command_buffers(vk, device, &command_pool, 1);
      render_pass = Vulkan::create_render_pass(vk, device, &format);
      framebuffers = Vulkan::create_frame_buffers(vk, device, &render_pass, &current_extent, image_views);
      
      let (vshader, fshader) = Vulkan::create_shaders(vk, device);
      vertex_shader = vshader;
      fragment_shader = fshader;
      
      descriptor_set_layout = Vulkan::create_descriptor_set_layout(vk, device);
      descriptor_set_pool = Vulkan::create_descriptor_pool(vk, device);
      descriptor_sets = Vulkan::create_descriptor_sets(vk, device, &descriptor_set_layout, &descriptor_set_pool);
      
      let (pipeline, cache, layout) = Vulkan::create_pipelines(vk, device, &vertex_shader, &fragment_shader, &render_pass, &current_extent, &format, &descriptor_set_layout);
      pipelines = pipeline;
      pipeline_cache = cache;
      pipeline_layout = layout;
    }
    
    Vulkan {
      window: window,
      fence: fence,
      semaphore_image_available: semaphore_image_available,
      semaphore_render_finished: semaphore_render_finished,
      command_pool: command_pool,
      command_buffers: command_buffers,
      render_pass: render_pass,
      framebuffers: framebuffers,
      vertex_shader: vertex_shader,
      fragment_shader: fragment_shader,
      descriptor_set_layout: descriptor_set_layout,
      descriptor_set_pool: descriptor_set_pool,
      descriptor_sets: descriptor_sets,
      pipelines: pipelines,
      pipeline_cache: pipeline_cache,
      pipeline_layout: pipeline_layout,
    }
  }
  
  pub fn setup(&mut self) {
    /*
    (Success, Not ready, device lost)
    VkResult vkGetFenceStatus(
    VkDevice                                    device,
    VkFence                                     fence);
    */ 
    
    /*
    Sets to unsignaled from host
    VkResult vkResetFences(
    VkDevice                                    device,
    uint32_t                                    fenceCount,
    const VkFence*                              pFences);
    */
    
    /*
    VkResult vkWaitForFences(
    VkDevice                                    device,
    uint32_t                                    fenceCount,
    const VkFence*                              pFences,
    VkBool32                                    waitAll,
    uint64_t                                    timeout);
    */
  }
  
  pub fn draw(&mut self) {
    let vk = self.window.device_pointers();
    
    let command_buffer_begin_info = vk::CommandBufferBeginInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      pNext: ptr::null(),
      flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
      pInheritanceInfo: ptr::null(),
    };
    
    unsafe {
      check_errors(vk.BeginCommandBuffer(self.command_buffers[0], &command_buffer_begin_info));
      
      // Add special handles or error codes for endcommandbuffer issues
      check_errors(vk.EndCommandBuffer(self.command_buffers[0])); 
      /*
      let mut submit_info: Vec<vk::SubmitInfo> = Vec::new();
      submit_info.push(vk::SubmitInfo {
        sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
        pNext: ptr::null(),
        waitSemaphoreCount: 1,
        pWaitSemaphores: &self.semaphore,
        pWaitDstStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        commandBufferCount: 1,
        pCommandBuffers: &self.command_buffers[0],
        signalSemaphoreCount: 1,
        pSignalSemaphores: &self.semaphore,
      });
      
      check_errors(vk::QueueSubmit( , 1, &submit_info, self.fence));*/
      
      check_errors(vk.ResetCommandBuffer(self.command_buffers[0], vk::COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT));
    }
  }
  
  pub fn get_events(&mut self) -> &mut winit::EventsLoop {
    self.window.get_events()
  }
  
  fn create_pipelines(vk: &vk::DevicePointers, device: &vk::Device, vertex_shader: &vk::ShaderModule, fragment_shader: &vk::ShaderModule, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D, swapchain_format: &vk::Format, descriptor_set_layout: &vk::DescriptorSetLayout) -> (Vec<vk::Pipeline>, vk::PipelineCache, vk::PipelineLayout) {
    let mut pipelines: Vec<vk::Pipeline> = Vec::with_capacity(1);
    let mut pipeline_layout: vk::PipelineLayout = unsafe { mem::uninitialized() };
    let mut pipeline_cache: vk::PipelineCache = unsafe { mem::uninitialized() };
    
    let mut graphics_pipeline_create_infos: Vec<vk::GraphicsPipelineCreateInfo> = Vec::with_capacity(2);
    let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::with_capacity(2);
    let mut vertex_input_binding_descriptions: Vec<vk::VertexInputBindingDescription> = Vec::with_capacity(1);
    let mut vertex_input_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> = Vec::with_capacity(4);
    
    let topology = vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;
    let polygon_mode = vk::POLYGON_MODE_FILL;
    let enable_depth_clamp = vk::TRUE;
    let cull_mode =  vk::CULL_MODE_BACK_BIT;
    let front_face = vk::FRONT_FACE_COUNTER_CLOCKWISE;
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
        pName: "main".as_ptr() as *const i8,
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
        pName: "main".as_ptr() as *const i8,
        pSpecializationInfo: ptr::null(),
      }
    );
    
    vertex_input_binding_descriptions.push(
      vk::VertexInputBindingDescription {
        binding: 0,
        stride: (mem::size_of::<f32>()*7) as u32,
        inputRate: vk::VERTEX_INPUT_RATE_VERTEX,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: *swapchain_format,
        offset: 0,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: *swapchain_format,
        offset: (mem::size_of::<f32>()*2) as u32,
      }
    );
    
    vertex_input_attribute_descriptions.push(
      vk::VertexInputAttributeDescription {
        location: 2,
        binding: 0,
        format: *swapchain_format,
        offset: (mem::size_of::<f32>()*5) as u32,
      }
    );
    
    let pipeline_vertex_input_state_create_info = {
      vk::PipelineVertexInputStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        vertexBindingDescriptionCount: vertex_input_binding_descriptions.len() as u32,
        pVertexBindingDescriptions: vertex_input_binding_descriptions.as_ptr(),
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
        dynamicStateCount: 1,
        pDynamicStates: &vk::DYNAMIC_STATE_VIEWPORT,
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
        pSetLayouts: descriptor_set_layout,
        pushConstantRangeCount: 0,
        pPushConstantRanges: &push_constant_range,
      }
    };
    
    unsafe {
      vk.CreatePipelineLayout(*device, &pipeline_layout_create_info, ptr::null(), &mut pipeline_layout);
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
        pDepthStencilState: &pipeline_depth_stencil_state_create_info,
        pColorBlendState: &pipeline_colour_blend_state_create_info,
        pDynamicState: &dynamic_state_create_info,
        layout: pipeline_layout,
        renderPass: *render_pass,
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
      check_errors(vk.CreatePipelineCache(*device, &pipeline_cache_create_info, ptr::null(), &mut pipeline_cache));
      check_errors(vk.CreateGraphicsPipelines(*device, pipeline_cache, graphics_pipeline_create_infos.len() as u32, graphics_pipeline_create_infos.as_ptr(), ptr::null(), pipelines.as_mut_ptr()));
      pipelines.set_len(graphics_pipeline_create_infos.len());
    }
    
    (pipelines, pipeline_cache, pipeline_layout)
  }
  
  fn create_descriptor_sets(vk: &vk::DevicePointers, device: &vk::Device, descriptor_set_layout: &vk::DescriptorSetLayout, descriptor_set_pool: &vk::DescriptorPool) -> Vec<vk::DescriptorSet> {
    let mut descriptor_sets: Vec<vk::DescriptorSet> = Vec::with_capacity(1);
    
    let descriptor_set_allocate_info = {
      vk::DescriptorSetAllocateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
        pNext: ptr::null(),
        descriptorPool: *descriptor_set_pool,
        descriptorSetCount: 1,
        pSetLayouts: descriptor_set_layout,
      }
    };
    
    unsafe {
      check_errors(vk.AllocateDescriptorSets(*device, &descriptor_set_allocate_info, descriptor_sets.as_mut_ptr()));
    }
    
    descriptor_sets
  }
  
  fn create_descriptor_pool(vk: &vk::DevicePointers, device: &vk::Device) -> vk::DescriptorPool {
    let mut descriptor_pool: vk::DescriptorPool = unsafe { mem::uninitialized() };
    let mut descriptor_pool_size: Vec<vk::DescriptorPoolSize> = Vec::with_capacity(2);
    
    descriptor_pool_size.push(
      vk::DescriptorPoolSize {
        ty: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        descriptorCount: 1,
      }
    );
    
    descriptor_pool_size.push(
      vk::DescriptorPoolSize {
        ty: vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
        descriptorCount: 1,
      }
    );
    
    let descriptor_pool_create_info = {
      vk::DescriptorPoolCreateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        maxSets: 1,
        poolSizeCount: descriptor_pool_size.len() as u32,
        pPoolSizes: descriptor_pool_size.as_ptr(),
      }
    };
    
    unsafe {
      check_errors(vk.CreateDescriptorPool(*device, &descriptor_pool_create_info, ptr::null(), &mut descriptor_pool));
    }
    
    descriptor_pool
  }
  
  fn create_descriptor_set_layout(vk: &vk::DevicePointers, device: &vk::Device) -> vk::DescriptorSetLayout {
    let mut descriptor_set_layout: vk::DescriptorSetLayout = unsafe { mem::uninitialized() };
    let mut descriptor_bindings: Vec<vk::DescriptorSetLayoutBinding> = Vec::with_capacity(2);
    
    descriptor_bindings.push(
      vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptorType: vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        descriptorCount: 1,
        stageFlags: vk::SHADER_STAGE_VERTEX_BIT,
        pImmutableSamplers: ptr::null(),
      }
    );
    
    descriptor_bindings.push(
      vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptorType: vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
        descriptorCount: 1,
        stageFlags: vk::SHADER_STAGE_FRAGMENT_BIT,
        pImmutableSamplers: ptr::null(),
      }
    );
    
    let descriptor_set_layout_create_info = {
      vk::DescriptorSetLayoutCreateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        bindingCount: descriptor_bindings.len() as u32,
        pBindings: descriptor_bindings.as_ptr(),
      }
    };
    
    unsafe {
      vk.CreateDescriptorSetLayout(*device, &descriptor_set_layout_create_info, ptr::null(), &mut descriptor_set_layout);
    }
    
    descriptor_set_layout
  }
  
  fn create_shaders(vk: &vk::DevicePointers, device: &vk::Device) -> (vk::ShaderModule, vk::ShaderModule) {
    let vertex_shader_data = include_bytes!("../shaders/test_vert.spv");
    let fragment_shader_data = include_bytes!("../shaders/test_frag.spv");
    
    let mut shader_module_vertex: vk::ShaderModule = unsafe { mem::uninitialized() };
    let mut shader_module_fragment: vk::ShaderModule = unsafe { mem::uninitialized() };
    
    let mut vertex_code_size = mem::size_of::<u8>() * vertex_shader_data.len();
    let mut fragment_code_size = mem::size_of::<u8>() * fragment_shader_data.len();
    
    let mut multiple_of_4 = false;
    while !multiple_of_4 {
      if vertex_code_size % 4 == 0 {
        break;
      }
      vertex_code_size += 1;
    }
    
    multiple_of_4 = false;
    while !multiple_of_4 {
      if fragment_code_size % 4 == 0 {
        break;
      }
      fragment_code_size += 1;
    }
    
    let mut vertex_shader_module_create_info = vk::ShaderModuleCreateInfo {
      sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      codeSize: vertex_code_size,
      pCode: vertex_shader_data.as_ptr() as *const u32,
    };
    
    let mut fragment_shader_module_create_info = vk::ShaderModuleCreateInfo {
      sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      codeSize: fragment_code_size,
      pCode: fragment_shader_data.as_ptr() as *const u32,
    };
    
    unsafe {
      vk.CreateShaderModule(*device, &vertex_shader_module_create_info, ptr::null(), &mut shader_module_vertex);
      vk.CreateShaderModule(*device, &fragment_shader_module_create_info, ptr::null(), &mut shader_module_fragment);
    }
    
    (shader_module_vertex, shader_module_fragment)
  }
  
  fn create_frame_buffers(vk: &vk::DevicePointers, device: &vk::Device, render_pass: &vk::RenderPass, swapchain_extent: &vk::Extent2D, image_views: &Vec<vk::ImageView>) -> Vec<vk::Framebuffer> {
    let mut framebuffers: Vec<vk::Framebuffer> = Vec::with_capacity(image_views.len());
    
    for i in 0..image_views.len() {
      let mut framebuffer: vk::Framebuffer = unsafe { mem::uninitialized() };
      
      let framebuffer_create_info = vk::FramebufferCreateInfo {
        sType: vk::STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        renderPass: *render_pass,
        attachmentCount: 1,
        pAttachments: image_views.as_ptr(),
        width: swapchain_extent.width,
        height: swapchain_extent.height,
        layers: 1,
      };
      
      unsafe {
        check_errors(vk.CreateFramebuffer(*device, &framebuffer_create_info, ptr::null(), &mut framebuffer));
      }
      
      framebuffers.push(framebuffer)
    }
    
    framebuffers
  }
  
  fn create_render_pass(vk: &vk::DevicePointers, device: &vk::Device, format: &vk::Format) -> vk::RenderPass {
    
    let mut render_pass: vk::RenderPass = unsafe { mem::uninitialized() };
    
    let attachment_description = vk::AttachmentDescription {
      flags: 0,
      format: *format,
      samples: vk::SAMPLE_COUNT_1_BIT,
      loadOp: vk::ATTACHMENT_LOAD_OP_CLEAR,
      storeOp: vk::ATTACHMENT_STORE_OP_STORE,
      stencilLoadOp: vk::ATTACHMENT_LOAD_OP_DONT_CARE,
      stencilStoreOp: vk::ATTACHMENT_STORE_OP_DONT_CARE,
      initialLayout: vk::IMAGE_LAYOUT_UNDEFINED,
      finalLayout: vk::IMAGE_LAYOUT_PRESENT_SRC_KHR,
    };
    
   // let mut input_attachments: Vec<vk::AttachmentReference>;
    let mut colour_attachments: Vec<vk::AttachmentReference> = Vec::new();
    //let mut resolve_attachmets: Vec<vk::AttachmentReference>;
    
    colour_attachments.push(
      vk::AttachmentReference {
        attachment: 0,
        layout: vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
      }
    );
    
    let subpass_description = vk::SubpassDescription {
      flags: 0,
      pipelineBindPoint: vk::PIPELINE_BIND_POINT_GRAPHICS,
      inputAttachmentCount: 0,//input_attachments.len() as u32,
      pInputAttachments: ptr::null(),//input_attachments,
      colorAttachmentCount: colour_attachments.len() as u32,
      pColorAttachments: colour_attachments.as_ptr(),
      pResolveAttachments: ptr::null(),//resolve_attachmets.len() as u32,
      pDepthStencilAttachment: ptr::null(),//resolve_attachmets,
      preserveAttachmentCount: 0,
      pPreserveAttachments: ptr::null(),
    };
    
    let subpass_dependency = vk::SubpassDependency {
      srcSubpass: vk::SUBPASS_EXTERNAL,
      dstSubpass: 0,
      srcStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
      dstStageMask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
      srcAccessMask: 0,
      dstAccessMask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
      dependencyFlags: vk::DEPENDENCY_BY_REGION_BIT,
    };
    
    let render_pass_create_info = vk::RenderPassCreateInfo {
      sType: vk::STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      attachmentCount: 1,
      pAttachments: &attachment_description,
      subpassCount: 1,
      pSubpasses: &subpass_description,
      dependencyCount: 1,
      pDependencies: &subpass_dependency,
    };
    
    unsafe {
      vk.CreateRenderPass(*device, &render_pass_create_info, ptr::null(), &mut render_pass);
    }
    
    render_pass
  }
  
  fn create_command_buffers(vk: &vk::DevicePointers, device: &vk::Device, command_pool: &vk::CommandPool, num_command_command_buffers: u32) -> Vec<vk::CommandBuffer> {
    let mut command_buffers: Vec<vk::CommandBuffer> = Vec::with_capacity(num_command_command_buffers as usize);
    
    let allocate_command_buffer_info = vk::CommandBufferAllocateInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
      pNext: ptr::null(),
      commandPool: *command_pool,
      level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
      commandBufferCount: num_command_command_buffers,
    };
    
    unsafe {
      check_errors(vk.AllocateCommandBuffers(*device, &allocate_command_buffer_info, command_buffers.as_mut_ptr()));
      command_buffers.set_len(num_command_command_buffers as usize);
    }
    
    command_buffers
  }
  
  fn create_command_pool(vk: &vk::DevicePointers, device: &vk::Device, graphics_family: u32) -> vk::CommandPool {
    let mut command_pool: vk::CommandPool = unsafe { mem::uninitialized() };
    
    let command_pool_info = vk::CommandPoolCreateInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
      pNext: ptr::null(),
      flags: vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,//vk::COMMAND_POOL_CREATE_TRANSIENT_BIT, //to use vkResetCommandBuffer change to vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT
      queueFamilyIndex: graphics_family,
    };
    
    unsafe {
      check_errors(vk.CreateCommandPool(*device, &command_pool_info, ptr::null(), &mut command_pool));
    }
    
    command_pool
  }
  
  fn create_semaphores(vk: &vk::DevicePointers, device: &vk::Device) -> (vk::Semaphore, vk::Semaphore) {
    let mut semaphore_image_available: vk::Semaphore = unsafe { mem::uninitialized() };
    let mut semaphore_render_finished: vk::Semaphore = unsafe { mem::uninitialized() };
    
    let semaphore_info = vk::SemaphoreCreateInfo {
      sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
    };
    
    unsafe {
      check_errors(vk.CreateSemaphore(*device, &semaphore_info, ptr::null(), &mut semaphore_image_available));
      check_errors(vk.CreateSemaphore(*device, &semaphore_info, ptr::null(), &mut semaphore_render_finished));
    }
    
    (semaphore_image_available, semaphore_render_finished)
  }
  
  fn create_fence(vk: &vk::DevicePointers, device: &vk::Device) -> vk::Fence {
    let mut fence: vk::Fence = unsafe { mem::uninitialized() };
    
    let fence_info = vk::FenceCreateInfo {
      sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
    };
    
    unsafe {
      check_errors(vk.CreateFence(*device, &fence_info, ptr::null(), &mut fence));
    }
    
    fence
  }
}

impl Drop for Vulkan {
  fn drop(&mut self) {
    let device = self.window.device();
    let vk = self.window.device_pointers();
    println!("Destroying Command pool, semaphores and fences");
    unsafe {
      vk.DestroyPipelineLayout(*device, self.pipeline_layout, ptr::null());
      vk.DestroyPipelineCache(*device, self.pipeline_cache, ptr::null());
      
      for pipeline in &self.pipelines {
        vk.DestroyPipeline(*device, *pipeline, ptr::null());
      }
      
      vk.DestroyDescriptorPool(*device, self.descriptor_set_pool, ptr::null());
      vk.DestroyDescriptorSetLayout(*device, self.descriptor_set_layout, ptr::null());
      
      vk.DestroyShaderModule(*device, self.vertex_shader, ptr::null());
      vk.DestroyShaderModule(*device, self.fragment_shader, ptr::null());
      
      for framebuffer in &self.framebuffers {
        vk.DestroyFramebuffer(*device, *framebuffer, ptr::null());
      }
      vk.DestroyRenderPass(*device, self.render_pass, ptr::null());
      vk.FreeCommandBuffers(*device, self.command_pool, self.command_buffers.len() as u32, self.command_buffers.as_mut_ptr());
      vk.DestroyCommandPool(*device, self.command_pool, ptr::null());
      vk.DestroySemaphore(*device, self.semaphore_image_available, ptr::null());
      vk.DestroySemaphore(*device, self.semaphore_render_finished, ptr::null());
      vk.DestroyFence(*device, self.fence, ptr::null());
    }
  }
}
