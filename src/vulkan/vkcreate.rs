use graphics::Vertex2d;

fn create_renderpasses(device: Arc<Device>, samples: u32, dimensions: [u32; 2]) {
  let texture_renderpass = vulkan_2d::create_texturebuffer(device.clone(), dimensions, samples);
  let forwardbuffer_renderpass = vulkan_3d::create_forwardbuffer(device.clone(), dimensions, samples);
  let final_renderpass = vulkan_2d::create_finalbuffer(device.clone(), dimensions);
  
}

fn create_pipelines(device: Arc<Device>) {
  let vs_plain = vs_plain::Shader::load(device).expect("failed to create shader module");
  let vs_forwardbuffer_3d = vs_forwardbuffer_3d::Shader::load(device).expect("failed to create shader module");
  let fs_forwardbuffer_3d = fs_forwardbuffer_3d::Shader::load(device.clone()).expect("failed to create shader module");
  let vs_texture = vs_texture::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_texture = fs_texture::Shader::load(device.clone()).expect("failed to create shader module");
  let vs_text = vs_text::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_text = fs_text::Shader::load(device.clone()).expect("failed to create shader module");
  let vs_final = vs_post_final::Shader::load(device.clone()).expect("failed to create shader module");
  let fs_final = fs_post_final::Shader::load(device.clone()).expect("failed to create shader module");
  
  let final_pipeline = Some(Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_final.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs_final.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(self.vkpost.final_renderpass.clone().unwrap() , 0).unwrap())
        .build(self.window.get_device())
        .unwrap()));
  
}

fn create_samplers() {
  
}

fn create_attachments() {
  
}

fn create_vertex_buffers() {
  
}

fn create_uniform_buffers() {
  
}

fn create_command_buffers() {
  
}
