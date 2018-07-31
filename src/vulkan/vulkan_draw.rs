use vulkano::memory;
use vulkano::format;
use vulkano::sampler;
use vulkano::framebuffer;
use vulkano::device::Queue;
use vulkano::device::Device;
use vulkano::image as vkimage;
use vulkano::buffer::cpu_pool;
use vulkano::buffer::BufferAccess;
use vulkano::image::ImmutableImage;
use vulkano::instance::QueueFamily;
use vulkano::descriptor::descriptor_set;
use vulkano::pipeline::viewport::Viewport;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::pipeline::GraphicsPipelineAbstract;

use std::sync::Arc;
use std::collections::HashMap;

use cgmath::Vector3;
use cgmath::Matrix4;

use vulkan::rawvk::{Mesh, Model, DynamicModel, vs_3d, vs_text, vs_texture, fs_lights};
use drawcalls;
use drawcalls::DrawCall;
use font::GenericFont;

pub fn draw_lightpass(tmp_cmd_buffer: AutoCommandBufferBuilder,
               pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
               vertex_buffer: Arc<BufferAccess + Send + Sync>,
               colour_attachment: Arc<vkimage::AttachmentImage>,
               normal_attachment: Arc<vkimage::AttachmentImage>,
               position_attachment: Arc<vkimage::AttachmentImage>,
               uv_attachment: Arc<vkimage::AttachmentImage>,
               mr_attachment: Arc<vkimage::AttachmentImage>,
               view_matrix: Matrix4<f32>,
               camera: Vector3<f32>,
               dimensions: [u32; 2]) -> AutoCommandBufferBuilder {
  let mut tmp_cmd_buffer = tmp_cmd_buffer;
  
  let push_constants = fs_lights::ty::PushConstants {
    view: view_matrix.into(),
    camera_pos: camera.into(),
  };
  
  let set_3d = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                  .add_image(colour_attachment).unwrap()
                  .add_image(normal_attachment).unwrap()
                  .add_image(position_attachment).unwrap()
                  .add_image(uv_attachment).unwrap()
                  .add_image(mr_attachment).unwrap()
                  .build().unwrap()
  );
  
  let cb = tmp_cmd_buffer;
  
  tmp_cmd_buffer = cb.draw(
               pipeline.clone(),
                 DynamicState {
                   line_width: None,
                   viewports: Some(vec![Viewport {
                     origin: [0.0, 0.0],
                     dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                     depth_range: 0.0 .. 1.0,
                   }]),
                   scissors: None,
                 },
                 vec!(vertex_buffer.clone()),
                 set_3d, push_constants).unwrap();
  
  tmp_cmd_buffer
}

pub fn draw_bloompass(tmp_cmd_buffer: AutoCommandBufferBuilder,
               pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
               vertex_buffer: Arc<BufferAccess + Send + Sync>,
               colour_attachment: Arc<vkimage::AttachmentImage>,
               dimensions: [u32; 2]) -> AutoCommandBufferBuilder {
  let mut tmp_cmd_buffer = tmp_cmd_buffer;
  
  let set_3d = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                  .add_image(colour_attachment).unwrap()
                  .build().unwrap()
  );
  
  let cb = tmp_cmd_buffer;
  
  tmp_cmd_buffer = cb.draw(
               pipeline.clone(),
                 DynamicState {
                   line_width: None,
                   viewports: Some(vec![Viewport {
                     origin: [0.0, 0.0],
                     dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                     depth_range: 0.0 .. 1.0,
                   }]),
                   scissors: None,
                 },
                 vec!(vertex_buffer.clone()),
                 set_3d, ()).unwrap();
  
  tmp_cmd_buffer
}

pub fn draw_3d(tmp_cmd_buffer: AutoCommandBufferBuilder, draw: &DrawCall,
               models: &HashMap<String, Vec<Mesh>>, projection: Matrix4<f32>,
               view_matrix: Matrix4<f32>,
               pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
               uniform_subbuffer: cpu_pool::CpuBufferPoolSubbuffer<vs_3d::ty::Data, Arc<memory::pool::StdMemoryPool>>,
               dimensions: [u32; 2]) -> (AutoCommandBufferBuilder, u32) {
  let mut tmp_cmd_buffer = tmp_cmd_buffer;
  let mut num_drawcalls = 0;
  
  if let Some(model) = models.get(draw.get_texture()) {
    let set_3d = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                  .add_buffer(uniform_subbuffer).unwrap()
                  .build().unwrap()
    );
    
    for i in 0..model.len() {
      num_drawcalls += 1;
      let material_set = model[i].material_desctriptor.clone();
      
      let cb = tmp_cmd_buffer;
      
      tmp_cmd_buffer = cb.draw_indexed(
               pipeline.clone(),
                 DynamicState {
                   line_width: None,
                   viewports: Some(vec![Viewport {
                     origin: [0.0, 0.0],
                     dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                     depth_range: 0.0 .. 1.0,
                   }]),
                   scissors: None,
                 },
                 model[i].vertex_buffer.clone().unwrap(),
                 model[i].index_buffer.clone().unwrap(), 
                 (set_3d.clone(), material_set.clone()), ()).unwrap();
    }
  } else {
    println!("Error: Model {} doesn't exist", draw.get_texture());
  }
  
  (tmp_cmd_buffer, num_drawcalls)
}

pub fn draw_texture(draw: &DrawCall,
                 textures: &HashMap<String, Arc<ImmutableImage<format::R8G8B8A8Unorm>>>,
                 vao: &Model, custom_vao: &HashMap<String, Model>, 
                 custom_dynamic_vao: &HashMap<String, DynamicModel>,
                 projection: Matrix4<f32>, sampler: Arc<sampler::Sampler>,
                 uniform_subbuffer: cpu_pool::CpuBufferPoolSubbuffer<vs_texture::ty::Data, Arc<memory::pool::StdMemoryPool>>,
                 pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
                 subpass: framebuffer::Subpass<Arc<RenderPassAbstract + Send + Sync>>,
                 device: Arc<Device>, queue: Arc<Queue>, queue_family: QueueFamily, dimensions: [u32; 2]) -> (Option<AutoCommandBufferBuilder>, u32) {
  // Texture
  let (temp_tex, _) = vkimage::immutable::ImmutableImage::from_iter([0u8, 0u8, 0u8, 0u8].iter().cloned(),
                                        vkimage::Dimensions::Dim2d { width: 1, height: 1 },
                                        format::R8G8B8A8Unorm, queue)
                                        .expect("Failed to create immutable image");
  
  let mut texture = temp_tex;
  
  if draw.get_texture() != &String::from("") {
    if textures.contains_key(draw.get_texture()) {
      texture = textures.get(draw.get_texture()).unwrap().clone();
    } else {
      println!("Texture not found: {}", draw.get_texture());
    }
  }
  
  let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                             .add_sampled_image(texture, sampler.clone()).unwrap()
                             .add_buffer(uniform_subbuffer.clone()).unwrap()
                             .build().unwrap());
  
  let mut tmp_cmd_buffer = None;
  
  if draw.is_custom_vao() {
    if custom_vao.contains_key(draw.get_text()) {
      let vertex_buffer = custom_vao.get(draw.get_text()).unwrap()
                            .vertex_buffer.clone()
                            .expect("Error: Unwrapping static custom vertex buffer failed!");
      let index_buffer = custom_vao.get(draw.get_text()).unwrap()
                             .index_buffer.clone()
                             .expect("Error: Unwrapping static custom index buffer failed!");
      tmp_cmd_buffer = Some(AutoCommandBufferBuilder::secondary_graphics_one_time_submit(device, queue_family, subpass).unwrap()
                                .draw_indexed(pipeline.clone(),
                                    DynamicState {
                                      line_width: None,
                                      viewports: Some(vec![Viewport {
                                        origin: [0.0, 0.0],
                                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                        depth_range: 0.0 .. 1.0,
                                      }]),
                                      scissors: None,
                                    },
                                    vertex_buffer,
                                    index_buffer,
                                    uniform_set, ()).unwrap());
    } else if custom_dynamic_vao.contains_key(draw.get_text()) {
      let vertex_buffer = custom_dynamic_vao.get(draw.get_text()).unwrap()
                              .vertex_buffer.clone()
                              .expect("Error: Unwrapping static custom vertex buffer failed!");
      let index_buffer = custom_dynamic_vao.get(draw.get_text()).unwrap()
                             .index_buffer.clone()
                             .expect("Error: Unwrapping static custom index buffer failed!");
      
      tmp_cmd_buffer = Some(AutoCommandBufferBuilder::secondary_graphics_one_time_submit(device, queue_family, subpass).unwrap()
                                .draw_indexed(pipeline.clone(),
                                    DynamicState {
                                      line_width: None,
                                      viewports: Some(vec![Viewport {
                                        origin: [0.0, 0.0],
                                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                        depth_range: 0.0 .. 1.0,
                                      }]),
                                      scissors: None,
                                    },
                                    vertex_buffer,
                                    index_buffer,
                                    uniform_set, ()).unwrap());
    } else {
      println!("Error: custom vao {:?} does not exist!", draw.get_text());
    }
  } else {
    let vertex_buffer = vao.vertex_buffer.clone().expect("Error: Unwrapping vertex buffer failed!");
    let index_buffer = vao.index_buffer.clone().expect("Error: Unwrapping index buffer failed!");
    tmp_cmd_buffer = Some(AutoCommandBufferBuilder::secondary_graphics_one_time_submit(device, queue_family, subpass).unwrap()
                                 .draw_indexed(pipeline.clone(),
                                    DynamicState {
                                      line_width: None,
                                      viewports: Some(vec![Viewport {
                                        origin: [0.0, 0.0],
                                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                        depth_range: 0.0 .. 1.0,
                                      }]),
                                      scissors: None,
                                    },
                                    vertex_buffer,
                                    index_buffer,
                                    uniform_set, ()).unwrap());
  }
  
  (tmp_cmd_buffer, 1)
}

pub fn draw_text(draw: &DrawCall,
                 textures: &HashMap<String, Arc<ImmutableImage<format::R8G8B8A8Unorm>>>,
                 projection: Matrix4<f32>, vao: &Model, sampler: Arc<sampler::Sampler>,
                 uniform_buffer: &cpu_pool::CpuBufferPool<vs_text::ty::Data>,
                 pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
                 fonts: &HashMap<String, GenericFont>,
                 subpass: framebuffer::Subpass<Arc<RenderPassAbstract + Send + Sync>>,
                 device: Arc<Device>, queue_family: QueueFamily, dimensions: [u32; 2]) -> (Vec<AutoCommandBufferBuilder>, u32) {
  let mut num_drawcalls = 0;
  let mut tmp_cmd_buffer = Vec::new();
  
  let wrapped_draw = drawcalls::setup_correct_wrapping(draw.clone(), fonts.clone());
  let size = draw.get_x_size();
  
  if !fonts.contains_key(draw.get_texture()) || !textures.contains_key(draw.get_texture()) {
    println!("Error: text couldn't draw, Texture: {:?}", draw.get_texture());
    return (tmp_cmd_buffer, 0)
  }
  
  let vertex_buffer = vao.vertex_buffer.clone()
                                       .expect("Error: Unwrapping text vertex buffer failed!");
  let index_buffer = vao.index_buffer.clone()
                                     .expect("Error: Unwrapping text index buffer failed!");
  
  for letter in wrapped_draw {
    let char_letter = {
      letter.get_text().as_bytes()[0] 
    };
    
    let c = fonts.get(draw.get_texture()).unwrap().get_character(char_letter as i32);
    
    let model = drawcalls::calculate_text_model(letter.get_translation(), size, &c.clone(), char_letter);
    let letter_uv = drawcalls::calculate_text_uv(&c.clone());
    let colour = letter.get_colour();
    let outline = letter.get_outline_colour();
    let edge_width = letter.get_edge_width(); 
    
    let uniform_buffer_text_subbuffer = {
      let uniform_data = vs_text::ty::Data {
        outlineColour: outline.into(),
        colour: colour.into(),
        edge_width: edge_width.into(),
        letter_uv: letter_uv.into(),
        model: model.into(),
        projection: projection.into(),
      };
      uniform_buffer.next(uniform_data).unwrap()
    };
    
    let uniform_set = Arc::new(descriptor_set::PersistentDescriptorSet::start(pipeline.clone(), 0)
                               .add_sampled_image(textures.get(draw.get_texture()).unwrap().clone(), sampler.clone()).unwrap()
                               .add_buffer(uniform_buffer_text_subbuffer.clone()).unwrap()
                               .build().unwrap());
    
    num_drawcalls += 1;
    tmp_cmd_buffer.push(AutoCommandBufferBuilder::secondary_graphics_one_time_submit(device.clone(), queue_family, subpass.clone()).unwrap()
                          .draw_indexed(pipeline.clone(),
                            DynamicState {
                              line_width: None,
                              viewports: Some(vec![Viewport {
                                origin: [0.0, 0.0],
                                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                depth_range: 0.0 .. 1.0,
                              }]),
                              scissors: None,
                            },
                            vertex_buffer.clone(),
                            index_buffer.clone(),
                            uniform_set, ()).unwrap());
  }
  
  (tmp_cmd_buffer, num_drawcalls)
}
