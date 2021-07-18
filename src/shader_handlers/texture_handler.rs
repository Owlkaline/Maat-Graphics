use std::mem;
use std::io::Cursor;
use std::collections::HashMap;

use ash::vk;

use crate::offset_of;

use crate::modules::{Vulkan, Image, ImageBuilder, Shader, Sampler, Buffer, DescriptorSet, DescriptorPoolBuilder,
                     DescriptorWriter, GraphicsPipelineBuilder};
use crate::shader_handlers::{Font, font::FontChar};

use crate::ash::version::DeviceV1_0;

#[derive(Clone, Debug, Copy)]
pub struct ComboVertex {
  pub pos: [f32; 4],
  pub colour: [f32; 4],
  pub uv: [f32; 2],
}

#[derive(Clone, Debug, Copy)]
struct UniformBuffer {
  colour: [f32; 4],
}

pub struct TextureHandler {
  descriptor_pool: vk::DescriptorPool,
  sampler: Sampler,
  uniform_buffer: Buffer<UniformBuffer>,
  uniform_descriptor: DescriptorSet,
  font: Font,
  //text_shader: Shader<ComboVertex>,
  letter_shader: Shader<ComboVertex>,
  combo_shader: Shader<ComboVertex>,
  combo_index_buffer: Buffer<u32>,
  combo_vertex_buffer: Buffer<ComboVertex>,
  strings: HashMap<String, (Buffer<u32>, Buffer<ComboVertex>)>,
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
    
    let strings = HashMap::new();
    
    let uniform_data = vec![
      UniformBuffer {
        colour: [screen_size.width as f32, screen_size.height as f32, 0.0, 0.0],
      }
    ];
    
    let uniform_buffer = Buffer::<UniformBuffer>::new_uniform_buffer(vulkan.device(), &uniform_data);
    
    let descriptor_set0 = DescriptorSet::builder()
                                      .uniform_buffer_vertex()
                                      .build(vulkan.device(), &descriptor_pool);
    
    let descriptor_set1 = DescriptorSet::builder()
                                      .combined_image_sampler_fragment()
                                      .build(vulkan.device(), &descriptor_pool);
    
    let uniform_descriptor_set_writer = DescriptorWriter::builder()
                                                        .update_uniform_buffer(&uniform_buffer, &descriptor_set0);
    
    uniform_descriptor_set_writer.build(vulkan.device());
    
    let (/*text_shader,*/ letter_shader, combo_shader, combo_index_buffer, combo_vertex_buffer) = TextureHandler::create_combo_shader(&vulkan, 
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
    
    TextureHandler {
      descriptor_pool,
      sampler,
      uniform_buffer,
      uniform_descriptor: descriptor_set0,
      font,
      //text_shader,
      letter_shader,
      combo_shader,
      combo_index_buffer,
      combo_vertex_buffer,
      strings,
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
                        &self.combo_shader,
                        &self.combo_vertex_buffer,
                        &self.combo_index_buffer,
                        data);
  }
  
  pub fn draw_text(&mut self, vulkan: &mut Vulkan, data: Vec<f32>, text: &str, _texture: &str) {
    let mut data = data;
    
    while data.len() < 16 {
      data.push(0.0);
    }
    
    let text_size = data[2].max(0.1);
    /*
    if let Some((string_index, string_vertex)) = self.strings.get(texture) {
      let descriptor = self.font.descriptor();
      
      vulkan.draw_texture(&descriptor,
                          &self.text_shader,
                          &string_vertex,
                          &string_index,
                          data);
    } else {*/
      let letter_data = self.font.generate_letter_draws(text_size, text.to_string());
      
      let x = data[0];
      let y = data[1];
      
      for (x_offset, y_offset, width, height, uvx0, uvy0, uvx1, uvy1) in letter_data {
        let descriptor = self.font.descriptor();
        
        data[0] = x + x_offset;
        data[1] = y + y_offset;
        data[2] = width;
        data[3] = height;
        
        data[12] = uvx0;
        data[13] = uvy0;
        data[14] = uvx1;
        data[15] = uvy1;
        
        vulkan.draw_texture(&descriptor,
                            &self.letter_shader,
                            &self.combo_vertex_buffer,
                            &self.combo_index_buffer,
                            data.clone());
      }
    //}
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
  
  fn create_combo_shader(vulkan: &Vulkan, descriptor_sets: &Vec<DescriptorSet>) -> (/*Shader<ComboVertex>,*/ Shader<ComboVertex>, Shader<ComboVertex>, Buffer<u32>, Buffer<ComboVertex>) {
    let combo_index_buffer_data = vec![0, 1, 2, 3, 4, 5];//vec![3, 2, 0, 2, 0, 1];
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
                                      &layouts);
    /*
    let text_shader = Shader::new(vulkan.device(),
                              Cursor::new(&include_bytes!("../../shaders/sdf_vert.spv")[..]),
                              Cursor::new(&include_bytes!("../../shaders/sdf_frag.spv")[..]),
                              combo_vertex, 
                              vec!(offset_of!(ComboVertex, pos) as u32, 
                                   offset_of!(ComboVertex, colour) as u32,
                                   offset_of!(ComboVertex, uv) as u32), 
                              &graphics_pipeline_builder,
                              vulkan.texture_renderpass(),
                              vulkan.viewports(), 
                              vulkan.scissors(),
                              &layouts);*/
    
    let letter_shader = Shader::new(vulkan.device(),
                              Cursor::new(&include_bytes!("../../shaders/letter_sdf_vert.spv")[..]),
                              Cursor::new(&include_bytes!("../../shaders/sdf_frag.spv")[..]),
                              combo_vertex, 
                              vec!(offset_of!(ComboVertex, pos) as u32, 
                                   offset_of!(ComboVertex, colour) as u32,
                                   offset_of!(ComboVertex, uv) as u32), 
                              &graphics_pipeline_builder,
                              vulkan.texture_renderpass(),
                              vulkan.viewports(), 
                              vulkan.scissors(),
                              &layouts);
    
    (/*text_shader, */letter_shader, combo_shader, combo_index_buffer, combo_vertex_buffer)
  }
}


