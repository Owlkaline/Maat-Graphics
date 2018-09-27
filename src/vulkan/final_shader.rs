use vulkano::buffer::BufferUsage;
use vulkano::buffer::ImmutableBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;

use vulkano::sampler;
use vulkano::sync::NowFuture;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use vulkano::image as vkimage;
use vulkano::image::SwapchainImage;

use vulkano::device::Queue;
use vulkano::device::Device;

use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;

use vulkano::format;
use vulkano::format::ClearValue;

use vulkano::pipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

use graphics::Vertex2d;
use vulkan::TextureShader;
use math;

use winit;

use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Matrix4;

use std::sync::Arc;

mod vs_final {
  #[derive(VulkanoShader)]
  #[ty = "vertex"]
  #[path = "src/shaders/glsl/VkFinal.vert"]
  struct _Dummy;
}

mod fs_final {
  #[derive(VulkanoShader)]
  #[ty = "fragment"]
  #[path = "src/shaders/glsl/VkFinal.frag"]
  struct _Dummy;
}

pub struct FinalShader {
  renderpass: Arc<RenderPassAbstract + Send + Sync>,
  pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
  framebuffer: Option<Vec<Arc<framebuffer::FramebufferAbstract + Send + Sync + Send + Sync>>>,
  uniformbuffer: CpuBufferPool<vs_final::ty::Data>,
  
  vertex_buffer: Arc<ImmutableBuffer<[Vertex2d]>>,
  index_buffer: Arc<ImmutableBuffer<[u32]>>,
  
  sampler: Arc<sampler::Sampler>,
}

impl FinalShader {
  pub fn create(device: Arc<Device>, queue: Arc<Queue>, swapchain_format: format::Format) -> (FinalShader, Vec<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>) {
    let uniformbuffer = CpuBufferPool::<vs_final::ty::Data>::new(device.clone(), BufferUsage::uniform_buffer());
    
    let vs_final = vs_final::Shader::load(device.clone()).expect("failed to create shader module");
    let fs_final = fs_final::Shader::load(device.clone()).expect("failed to create shader module");
    
    let renderpass = Arc::new(single_pass_renderpass!(device.clone(),
      attachments: {
        out_colour: {
          load: DontCare,
          store: Store,
          format: swapchain_format,
          samples: 1,
        }
      },
      pass: {
        color: [out_colour],
        depth_stencil: {},
        resolve: [],
      }
    ).unwrap());
    
    let pipeline = Arc::new(pipeline::GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2d>()
        .vertex_shader(vs_final.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        //.viewports_scissors_dynamic(1)
        .fragment_shader(fs_final.main_entry_point(), ())
        .blend_alpha_blending()
        .render_pass(framebuffer::Subpass::from(renderpass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());
    
    let (vertex_buffer, future_vtx) = TextureShader::create_vertex(Arc::clone(&queue));
    let (idx_buffer, future_idx) = TextureShader::create_index(queue);
    
    let sampler = sampler::Sampler::new(device.clone(), sampler::Filter::Linear,
                                                   sampler::Filter::Linear, 
                                                   sampler::MipmapMode::Nearest,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   sampler::SamplerAddressMode::ClampToEdge,
                                                   0.0, 1.0, 0.0, 0.0).unwrap();
    
    (
      FinalShader {
        renderpass: renderpass,
        pipeline: pipeline,
        framebuffer: None,
        uniformbuffer: uniformbuffer,
        
        vertex_buffer: vertex_buffer,
        index_buffer: idx_buffer,
        sampler: sampler,
      },
      vec!(future_idx, future_vtx)
    )
  }
  
  pub fn empty_framebuffer(&mut self) {
    self.framebuffer = None;
  }
  
  pub fn recreate_framebuffer(&mut self, images: &Vec<Arc<SwapchainImage<winit::Window>>>) {
    if self.framebuffer.is_none() {
      let new_framebuffer = Some(images.iter().map( |image| {
             let fb = framebuffer::Framebuffer::start(self.renderpass.clone())
                      .add(image.clone()).unwrap()
                      .build().unwrap();
             Arc::new(fb) as Arc<framebuffer::FramebufferAbstract + Send + Sync>
             }).collect::<Vec<_>>());
      self.framebuffer = new_framebuffer;
    }
  }
  
  pub fn begin_renderpass(&mut self, cb: AutoCommandBufferBuilder, secondary: bool, image_num: usize) -> AutoCommandBufferBuilder {
    cb.begin_render_pass(self.framebuffer.as_ref().unwrap()[image_num].clone(), secondary, vec![ClearValue::None]).unwrap()
  }
  
  pub fn draw(&mut self, cb: AutoCommandBufferBuilder, dynamic_state: &DynamicState, dimensions: [f32; 2], texture_projection: Matrix4<f32>, texture_image: Arc<vkimage::AttachmentImage>) -> AutoCommandBufferBuilder {
    
    let model = math::calculate_texture_model(Vector3::new(dimensions[0] as f32*0.5, dimensions[1] as f32*0.5, 0.0), Vector2::new(dimensions[0] as f32, dimensions[1] as f32), 90.0);
    
    let uniform_data = vs_final::ty::Data {
      projection: texture_projection.into(),
    };
    
    let push_constants = vs_final::ty::PushConstants {
      model: model.into(),
    };
    
    let uniform_subbuffer = self.uniformbuffer.next(uniform_data).unwrap();
    let vertex = self.vertex_buffer.clone();
    let index = self.index_buffer.clone();
    let pipeline = self.pipeline.clone();
    
    let descriptor_set = Arc::new(PersistentDescriptorSet::start(pipeline.clone(), 0)
                              .add_buffer(uniform_subbuffer.clone()).unwrap()
                              .add_sampled_image(Arc::clone(&texture_image), Arc::clone(&self.sampler)).unwrap()
                              .build().unwrap());
    
    cb.draw_indexed(pipeline, dynamic_state, vec!(vertex), index, descriptor_set, push_constants).unwrap()
  }
  
  pub fn end_renderpass(&mut self, cb: AutoCommandBufferBuilder) -> AutoCommandBufferBuilder {
    cb.end_render_pass().unwrap()
  }
}
