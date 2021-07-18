use std::mem;
use std::io::Cursor;
use std::collections::HashMap;

use ash::vk;

use crate::offset_of;

use crate::modules::{Vulkan, VkDevice, Image, ImageBuilder, Shader, Sampler, Buffer, DescriptorSet, DescriptorPoolBuilder,
                     DescriptorWriter, GraphicsPipelineBuilder};
use crate::shader_handlers::{Font, font::FontChar};

use crate::ash::version::DeviceV1_0;

const MAX_INSTANCES: usize = 8192;

#[derive(Clone, Debug, Copy)]
pub struct ComboVertex {
  pub pos: [f32; 4],
  pub colour: [f32; 4],
  pub uv: [f32; 2],
}

#[derive(Clone, Copy)]
pub struct InstancedTextData {
  pub pos: [f32; 2],
  pub size: [f32; 2],
  pub uv: [f32; 4],
  pub text_height: f32,
  pub colour: [f32; 4],
  pub outline_colour: [f32; 4],
  pub width_edge: [f32; 4],
}

impl InstancedTextData {
  pub fn new() -> InstancedTextData {
    InstancedTextData {
       pos: [0.0; 2],
       size: [0.0; 2],
       uv: [0.0; 4],
       text_height: 0.0,
       colour: [0.0; 4],
       outline_colour: [0.0; 4],
       width_edge: [0.0; 4],
    }
  }
}

#[derive(Clone, Debug, Copy)]
struct TextureUniformBuffer {
  window_size: [f32; 2],
}

pub struct TextureHandler {
  descriptor_pool: vk::DescriptorPool,
  sampler: Sampler,
  
  uniform_buffer: Buffer<TextureUniformBuffer>,
  uniform_descriptor: DescriptorSet,
  
  font: Font,
  
  letter_shader: Shader<ComboVertex>,
  instanced_letter_shader: Shader<ComboVertex>,
  instanced_letter_buffer: Buffer<InstancedTextData>,

  combo_shader: Shader<ComboVertex>,
  combo_index_buffer: Buffer<u32>,
  combo_vertex_buffer: Buffer<ComboVertex>,
  
  textures: HashMap<String, (Image, DescriptorSet)>,
  dummy_texture: (Image, DescriptorSet),
}

impl TextureHandler {
  pub fn new(vulkan: &mut Vulkan, screen_size: vk::Extent2D) -> TextureHandler {
    let descriptor_pool = DescriptorPoolBuilder::new()
                                              .num_combined_image_samplers(5)
                                              .num_uniform_buffers(5)
                                              .build(vulkan.device());
    
    let sampler = Sampler::builder()
                           .min_filter_linear()
                           .mag_filter_nearest()
                           .address_mode_mirrored_repeat()
                           .mipmap_mode_linear()
                           .border_colour_float_opaque_white()
                           .compare_op_never()
                           .build(vulkan.device());
    
    let font = Font::new(vulkan, &sampler);
    
    let uniform_data = vec![
      TextureUniformBuffer {
        window_size: [screen_size.width as f32, screen_size.height as f32],
      }
    ];
    
    let uniform_buffer = Buffer::<TextureUniformBuffer>::new_uniform_buffer(vulkan.device(), &uniform_data);
    
    let descriptor_set0 = DescriptorSet::builder()
                                      .uniform_buffer_vertex()
                                      .build(vulkan.device(), &descriptor_pool);
    
    let descriptor_set1 = DescriptorSet::builder()
                                      .combined_image_sampler_fragment()
                                      .build(vulkan.device(), &descriptor_pool);
    
    let uniform_descriptor_set_writer = DescriptorWriter::builder()
                                                        .update_uniform_buffer(&uniform_buffer, &descriptor_set0);
    
    uniform_descriptor_set_writer.build(vulkan.device());
    
    let (letter_shader, instanced_letter_shader, combo_shader, combo_index_buffer, combo_vertex_buffer) = 
        TextureHandler::create_combo_shader(&vulkan, 
                                            &vec![descriptor_set0.clone(), 
                                                  descriptor_set1.clone()]);
    
    let checked_image = TextureHandler::create_checked_image();
    let dummy_texture = TextureHandler::create_device_local_texture_from_image(vulkan, checked_image);
    let dummy_descriptor_set = DescriptorSet::builder()
                                      .combined_image_sampler_fragment()
                                      .build(vulkan.device(), &descriptor_pool);
    let dummy_descriptor_set_writer = DescriptorWriter::builder()
                                                        .update_image(&dummy_texture, &sampler, &dummy_descriptor_set);
    
    dummy_descriptor_set_writer.build(vulkan.device());
    
    let dummy_instanced_data = vec![InstancedTextData::new(); MAX_INSTANCES]; 
    let instanced_letter_buffer = Buffer::<InstancedTextData>::new_vertex(vulkan.device(), dummy_instanced_data);

    TextureHandler {
      descriptor_pool,
      sampler,
      
      uniform_buffer,
      uniform_descriptor: descriptor_set0,
      
      font,
      
      letter_shader,
      instanced_letter_shader,
      instanced_letter_buffer,

      combo_shader,
      combo_index_buffer,
      combo_vertex_buffer,
      
      //strings,
      textures: HashMap::new(),
      dummy_texture: (dummy_texture, dummy_descriptor_set),
    }
  }
  
  pub fn get_font_data(&self) -> (Vec<FontChar>, u32, u32) {
    self.font.get_font_data()
  }
  
  pub fn destroy(&mut self, vulkan: &mut Vulkan) {
    for (_, (image, descriptor)) in self.textures.drain().take(1) {
      image.destroy(vulkan.device());
      descriptor.destroy(vulkan.device());
    }
    
    self.dummy_texture.0.destroy(vulkan.device());
    self.dummy_texture.1.destroy(vulkan.device());
    
    self.combo_shader.destroy(vulkan.device());
    self.combo_index_buffer.destroy(vulkan.device());
    self.combo_vertex_buffer.destroy(vulkan.device());
    
    unsafe {
      vulkan.device().destroy_descriptor_pool(self.descriptor_pool, None);
    }
    
    self.sampler.destroy(vulkan.device());
  }
  
  pub fn shader(&self) -> &Shader<ComboVertex> {
    &self.combo_shader
  }
  
  pub fn uniform_descriptor(&self) -> &DescriptorSet {
    &self.uniform_descriptor
  }

  pub fn update_uniform_buffer(&mut self, device: &VkDevice, width: u32, height: u32) {
    let mut data = self.uniform_buffer.data()[0];
    data.window_size = [width as f32, height as f32];
    self.uniform_buffer.update_data(device, vec!(data)); 
    
    let uniform_descriptor_set_writer = DescriptorWriter::builder()
                                                        .update_uniform_buffer(&self.uniform_buffer, &self.uniform_descriptor);
    
    uniform_descriptor_set_writer.build(device);
  }
  
  pub fn load_texture<T: Into<String>>(&mut self, vulkan: &mut Vulkan, texture_ref: T, texture: T) {
    let texture = texture.into();
    
    let image = image::open(&texture).expect(&("Failed to load texture: ".to_string() + &texture)).fliph().to_rgba8();
    
    let dl_texture = TextureHandler::create_device_local_texture_from_image(vulkan, image);
    
    let descriptor_sets = DescriptorSet::builder()
                                      .combined_image_sampler_fragment()
                                      .build(vulkan.device(), &self.descriptor_pool);
    let descriptor_set_writer = DescriptorWriter::builder()
                                                  .update_image(&dl_texture, &self.sampler, &descriptor_sets);
    
    descriptor_set_writer.build(vulkan.device());
    
    self.textures.insert(texture_ref.into(), (dl_texture, descriptor_sets));
  }
  
  pub fn draw(&mut self, vulkan: &mut Vulkan, data: Vec<f32>, texture: &str) {
    let texture_descriptor = {
      if let Some((_, texture_descriptor)) = self.textures.get(texture) {
        texture_descriptor
      } else {
        &self.dummy_texture.1
      }
    };
    
    vulkan.draw_texture(&texture_descriptor,
                        &self.uniform_descriptor,
                        &self.combo_shader,
                        &self.combo_vertex_buffer,
                        &self.combo_index_buffer,
                        None as Option<&Buffer<u32>>,
                        0,
                        data);
  }
  
  pub fn draw_instanced_text(&mut self, vulkan: &mut Vulkan, instance_count: usize) {
    let descriptor = self.font.descriptor();
    
    self.instanced_letter_buffer.update_data(vulkan.device(), self.instanced_letter_buffer.data.clone()); 

    vulkan.draw_texture(&descriptor,
                        &self.uniform_descriptor,
                        &self.instanced_letter_shader,
                        &self.combo_vertex_buffer,
                        &self.combo_index_buffer,
                        Some(&self.instanced_letter_buffer),
                        instance_count,
                        vec!());
  }

  pub fn add_text_data(&mut self, idx: &mut usize, data: Vec<f32>, text: &str, _texture: &str) {
     let mut data = data;
    
    while data.len() < 16 {
      data.push(0.0);
    }
    
    let text_size = data[2].max(0.1);
    let letter_data = self.font.generate_letter_draws(text_size, text.to_string());
      
    let x = data[0];
    let y = data[1];
    
    for (x_offset, y_offset, width, height, uvx0, uvy0, uvx1, uvy1) in letter_data {
      let pos_x = x + x_offset;
      let pos_y = y + y_offset;
        
      self.instanced_letter_buffer.data[*idx].pos = [pos_x, pos_y];
      self.instanced_letter_buffer.data[*idx].size = [width, height];
      self.instanced_letter_buffer.data[*idx].uv = [uvx0, uvy0, uvx1, uvy1];
      self.instanced_letter_buffer.data[*idx].text_height = data[2];
      self.instanced_letter_buffer.data[*idx].colour = [data[4], data[5], data[6], data[7]]; 
      self.instanced_letter_buffer.data[*idx].outline_colour = [data[8], data[9], data[10], data[11]];
      self.instanced_letter_buffer.data[*idx].width_edge = [data[12], data[13], data[14], data[15]];
  
      *idx += 1;
    }     
  }
  
  pub fn create_checked_image() -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    image::ImageBuffer::from_fn(2, 2, |x, y| {
      if (x + y) % 2 == 0 {
        image::Rgba([0, 0, 0, 255])
      } else {
        image::Rgba([255, 255, 255, 255])
      }
    })
  }
  
  pub fn create_device_local_texture_from_image(vulkan: &mut Vulkan, image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> Image {
    let dimensions = image.dimensions();
    let image_data = image.into_raw();
    
    let src_buffer = Buffer::<u8>::new_image(vulkan.device(), image_data);
    let dst_image = ImageBuilder::new(vk::Format::R8G8B8A8_UNORM, 1, 1)
                                     .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                                     .set_dimensions(dimensions.0, dimensions.1)
                                     .build_device_local(vulkan.device());
    
    vulkan.copy_buffer_to_device_local_image(&src_buffer, &dst_image);
    
    dst_image
  }
  
  fn create_combo_shader(vulkan: &Vulkan, descriptor_sets: &Vec<DescriptorSet>) -> (Shader<ComboVertex>, Shader<ComboVertex>, Shader<ComboVertex>, Buffer<u32>, Buffer<ComboVertex>) {
    let combo_index_buffer_data = vec![0, 1, 2, 3, 4, 5];
    let z = -1.0;
    let combo_vertices = vec![
      ComboVertex {
          pos: [1.0, 1.0, z, 1.0],
          colour: [0.0, 1.0, 0.0, 1.0],
          uv: [0.0, 0.0],
      },
      ComboVertex {
          pos: [0.0, 1.0, z, 1.0],
          colour: [0.0, 0.0, 1.0, 1.0],
          uv: [1.0, 0.0],
      },
      ComboVertex {
          pos: [0.0, 0.0, z, 1.0],
          colour: [1.0, 0.0, 0.0, 1.0],
          uv: [1.0, 1.0],
      },
      
      ComboVertex {
          pos: [0.0, 0.0, z, 1.0],
          colour: [0.0, 1.0, 0.0, 1.0],
          uv: [1.0, 1.0],
      },
      ComboVertex {
          pos: [1.0, 0.0, z, 1.0],
          colour: [0.0, 0.0, 1.0, 1.0],
          uv: [0.0, 1.0],
      },
      ComboVertex {
          pos: [1.0, 1.0, z, 1.0],
          colour: [1.0, 0.0, 0.0, 1.0],
          uv: [0.0, 0.0],
      }
    ];
    
    let combo_vertex = ComboVertex {
      pos: [0.0, 0.0, 0.0, 0.0],
      colour: [0.0, 0.0, 0.0, 0.0],
      uv: [0.0, 0.0],
    };
    let instaced_text = InstancedTextData::new();
    
    let combo_index_buffer = Buffer::<u32>::new_index(&vulkan.device(), combo_index_buffer_data);
    let combo_vertex_buffer = Buffer::<ComboVertex>::new_vertex(vulkan.device(), combo_vertices);
    
    let graphics_pipeline_builder = GraphicsPipelineBuilder::new().topology_triangle_list()
                                                                      .front_face_counter_clockwise()
                                                                      .polygon_mode_fill()
                                                                      .samples_1();
    
    let layouts = {
      let mut sets = Vec::new();
      for i in 0..descriptor_sets.len() {
        sets.push(descriptor_sets[i].layouts()[0]);
      }
      
      sets
    };
    
    let combo_shader = Shader::new(vulkan.device(),
                                      Cursor::new(&include_bytes!("../../shaders/combo_vert.spv")[..]),
                                      Cursor::new(&include_bytes!("../../shaders/combo_frag.spv")[..]),
                                      combo_vertex, 
                                      vec!(offset_of!(ComboVertex, pos) as u32, 
                                           offset_of!(ComboVertex, colour) as u32,
                                           offset_of!(ComboVertex, uv) as u32), 
                                      &graphics_pipeline_builder,
                                      vulkan.texture_renderpass(),
                                      vulkan.viewports(), 
                                      vulkan.scissors(),
                                      &layouts,
                                      None as Option<(InstancedTextData, Vec<u32>)>);
    
    let letter_shader = Shader::new(vulkan.device(),
                              Cursor::new(&include_bytes!("../../shaders/letter_sdf_vert.spv")[..]),
                              Cursor::new(&include_bytes!("../../shaders/letter_sdf_frag.spv")[..]),
                              combo_vertex, 
                              vec!(offset_of!(ComboVertex, pos) as u32, 
                                   offset_of!(ComboVertex, colour) as u32,
                                   offset_of!(ComboVertex, uv) as u32), 
                              &graphics_pipeline_builder,
                              vulkan.texture_renderpass(),
                              vulkan.viewports(), 
                              vulkan.scissors(),
                              &layouts,
                              None as Option<(InstancedTextData, Vec<u32>)>);
    let instanced_letter_shader = Shader::new(vulkan.device(),
                              Cursor::new(&include_bytes!("../../shaders/instanced_letter_sdf_vert.spv")[..]),
                              Cursor::new(&include_bytes!("../../shaders/letter_sdf_frag.spv")[..]),
                              combo_vertex, 
                              vec!(offset_of!(ComboVertex, pos) as u32, 
                                   offset_of!(ComboVertex, colour) as u32,
                                   offset_of!(ComboVertex, uv) as u32), 
                              &graphics_pipeline_builder,
                              vulkan.texture_renderpass(),
                              vulkan.viewports(), 
                              vulkan.scissors(),
                              &layouts,
                              Some((
                                instaced_text,
                                vec!(offset_of!(InstancedTextData, pos) as u32,
                                     offset_of!(InstancedTextData, size) as u32,
                                     offset_of!(InstancedTextData, uv) as u32,
                                     offset_of!(InstancedTextData, text_height) as u32,
                                     offset_of!(InstancedTextData, colour) as u32,
                                     offset_of!(InstancedTextData, outline_colour) as u32,
                                     offset_of!(InstancedTextData, width_edge) as u32,
                                    ),
                              ))
                            );

    (letter_shader, instanced_letter_shader, combo_shader, combo_index_buffer, combo_vertex_buffer)
  }
}


