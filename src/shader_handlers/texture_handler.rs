use std::collections::HashMap;
use std::io::Cursor;
use std::mem;

use ash::vk;

use crate::offset_of;
use crate::shader_handlers::font::{FontType, GuiText, TextMaster};
use crate::vkwrapper::{
  Buffer, DescriptorPoolBuilder, DescriptorSet, DescriptorWriter, GraphicsPipelineBuilder, Image,
  ImageBuilder, Sampler, Shader, VkDevice, Vulkan,
};
use crate::Draw;

use glam::{Vec2, Vec3Swizzles, Vec4};

const MAX_INSTANCES: usize = 8196;

#[derive(Clone, Debug, Copy)]
pub struct ComboVertex {
  pub pos: [f32; 4],
  pub colour: [f32; 4],
  pub uv: [f32; 2],
}

#[derive(Clone, Debug, Copy)]
pub struct InstancedComboData {
  //pub pos: [f32; 4],
  //pub colour: [f32; 4],
  //pub uv: [f32; 4],
  pub pos_scale: [f32; 4],                        // x, y, scale_x, scale_y
  pub other_colour: [f32; 4],                     // r g b a
  pub is_textured_rotation_overlay_mix: [f32; 4], // is_textured, rotation, overlay_mix, empty
  pub sprite_sheet: [f32; 4],                     // rows, texture number, empty
  pub flip_xy: [f32; 4],                          // flip x y
  pub overlay_colour: [f32; 4],                   // overlay colour
  pub attrib6: [f32; 4],
  pub camera_intensity_time: [f32; 4], // camera x y, intensity time
}

impl InstancedComboData {
  pub fn new() -> InstancedComboData {
    InstancedComboData {
      // pos: [0.0; 4],
      // colour: [0.0; 4],
      // uv: [0.0; 4],
      pos_scale: [0.0; 4],                        // x, y, scale_x, scale_y
      other_colour: [0.0; 4],                     // r g b a
      is_textured_rotation_overlay_mix: [0.0; 4], // is_textured, rotation, overlay_mix, empty
      sprite_sheet: [0.0; 4],                     // rows, texture number, empty
      flip_xy: [0.0; 4],                          // flip x y
      overlay_colour: [0.0; 4],                   // overlay colour
      attrib6: [0.0; 4],
      camera_intensity_time: [0.0; 4],
    } // camera x y, intensity time
  }

  pub fn from_data(data: &[f32]) -> InstancedComboData {
    InstancedComboData {
      pos_scale: [data[0], data[1], data[2], data[3]],
      other_colour: [data[4], data[6], data[6], data[7]],
      is_textured_rotation_overlay_mix: [data[8], data[9], data[10], data[11]],
      sprite_sheet: [data[12], data[13], data[14], data[15]],
      flip_xy: [data[16], data[17], data[18], data[19]],
      overlay_colour: [data[20], data[21], data[22], data[23]],
      attrib6: [data[24], data[25], data[26], data[27]],
      camera_intensity_time: [data[28], data[29], data[30], data[31]],
      // overlay_colour: [data[32], data[33], data[34], data[35]],
      // attrib6: [data[36], data[37], data[38], data[39]],
      // camera_intensity_time: [data[40], data[41], data[42], data[43]],
    } // camera x y, intensity time
  }
}

#[derive(Clone, Copy)]
pub struct InstancedTextData {
  pub pos: [f32; 2],
  pub size: [f32; 2],
  pub uv: [f32; 4],
  pub text_height: f32,
  pub colour: [f32; 4],
  //  pub outline_colour: [f32; 4],
  //  pub width_edge: [f32; 4],
}

impl InstancedTextData {
  pub fn new() -> InstancedTextData {
    InstancedTextData {
      pos: [0.0; 2],
      size: [0.0; 2],
      uv: [0.0; 4],
      text_height: 0.0,
      colour: [0.0; 4],
      //      outline_colour: [0.0; 4],
      //      width_edge: [0.0; 4],
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

  text_master: TextMaster,
  text_this_draw: Vec<GuiText>,

  //font: Font,

  //letter_shader: Shader<ComboVertex>,
  //instanced_letter_shader: Shader<ComboVertex>,
  //instanced_letter_buffer: Buffer<InstancedTextData>,
  combo_shader: Shader<ComboVertex>,
  combo_index_buffer: Buffer<u32>,
  combo_vertex_buffer: Buffer<ComboVertex>,
  instanced_combo_shader: Shader<ComboVertex>,
  instanced_combo_buffer: HashMap<String, (String, Buffer<InstancedComboData>)>,

  textures: HashMap<String, (Image, DescriptorSet)>,
  dummy_texture: (Image, DescriptorSet),

  window_size: [f32; 2],

  camera_position: Vec2,
}

impl TextureHandler {
  pub fn new<T: Into<String>>(
    vulkan: &mut Vulkan,
    screen_size: vk::Extent2D,
    font_location: T,
  ) -> TextureHandler {
    let descriptor_pool = DescriptorPoolBuilder::new()
      .num_combined_image_samplers(50)
      .num_uniform_buffers(50)
      .build(vulkan.device());

    let sampler = Sampler::builder()
      .min_filter_linear()
      .mag_filter_nearest()
      .address_mode_mirrored_repeat()
      .mipmap_mode_linear()
      .border_colour_float_opaque_white()
      .compare_op_never()
      .build(vulkan.device());

    let uniform_data = vec![TextureUniformBuffer {
      window_size: [screen_size.width as f32, screen_size.height as f32],
    }];

    let uniform_buffer =
      Buffer::<TextureUniformBuffer>::new_uniform_buffer(vulkan.device(), &uniform_data);

    let descriptor_set0 = DescriptorSet::builder()
      .uniform_buffer_vertex()
      .build(vulkan.device(), &descriptor_pool);

    let descriptor_set1 = DescriptorSet::builder()
      .combined_image_sampler_fragment()
      .build(vulkan.device(), &descriptor_pool);

    let uniform_descriptor_set_writer =
      DescriptorWriter::builder().update_buffer(&uniform_buffer, &descriptor_set0);

    uniform_descriptor_set_writer.build(vulkan.device());

    let (
      //_letter_shader,
      //_instanced_letter_shader,
      combo_shader,
      combo_index_buffer,
      combo_vertex_buffer,
      instanced_combo_shader,
      // instanced_combo_buffer,
    ) = TextureHandler::create_combo_shader(
      &vulkan,
      &vec![descriptor_set0.clone(), descriptor_set1.clone()],
    );

    let checked_image = TextureHandler::create_checked_image();
    let dummy_texture =
      TextureHandler::create_device_local_texture_from_image(vulkan, checked_image);
    let dummy_descriptor_set = DescriptorSet::builder()
      .combined_image_sampler_fragment()
      .build(vulkan.device(), &descriptor_pool);
    let dummy_descriptor_set_writer =
      DescriptorWriter::builder().update_image(&dummy_texture, &sampler, &dummy_descriptor_set);

    dummy_descriptor_set_writer.build(vulkan.device());

    //let dummy_instanced_data = vec![InstancedTextData::new(); MAX_INSTANCES];
    //let _instanced_letter_buffer =
    //  Buffer::<InstancedTextData>::new_vertex(vulkan.device(), dummy_instanced_data);

    let font = FontType::new(font_location.into(), &sampler, vulkan);

    let mut text_master = TextMaster::new(vulkan, font);

    let gui_text = GuiText::new(
      "The quick brown fox jumps over the lazy dog.".to_string(),
      1400.0,
      Vec2::new(100.0, 600.0),
      Vec2::splat(0.0),
      Vec4::new(0.0, 0.0, 1.0, 1.0),
      HashMap::new(),
      100000000.0,
      false,
    );

    let gui_text_clone = gui_text.clone();
    text_master.load_text(gui_text, vulkan);

    TextureHandler {
      descriptor_pool,
      sampler,

      uniform_buffer,
      uniform_descriptor: descriptor_set0,

      text_master,
      text_this_draw: vec![gui_text_clone],

      //font,

      //letter_shader,
      //instanced_letter_shader,
      //instanced_letter_buffer,
      combo_shader,
      combo_index_buffer,
      combo_vertex_buffer,
      instanced_combo_shader,
      instanced_combo_buffer: HashMap::new(),

      //strings,
      textures: HashMap::new(),
      dummy_texture: (dummy_texture, dummy_descriptor_set),

      window_size: [screen_size.width as f32, screen_size.height as f32],

      camera_position: Vec2::splat(0.0),
    }
  }

  //pub fn get_font_data(&self) -> GlyphCache {
  //  self.font.get_font_data()
  //}

  pub fn set_camera_location(&mut self, pos: Vec2) {
    self.camera_position = pos;
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
    self.instanced_combo_shader.destroy(vulkan.device());
    // self.instanced_combo_buffer.destroy(vulkan.device());

    unsafe {
      vulkan
        .device()
        .internal()
        .destroy_descriptor_pool(self.descriptor_pool, None);
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
    self.window_size = [width as f32, height as f32];
    let mut data = self.uniform_buffer.data()[0];
    data.window_size = [width as f32, height as f32];
    self.uniform_buffer.update_data(device, vec![data]);

    let uniform_descriptor_set_writer =
      DescriptorWriter::builder().update_buffer(&self.uniform_buffer, &self.uniform_descriptor);

    uniform_descriptor_set_writer.build(device);
  }

  pub fn create_instance_render_buffer<T: Into<String>>(
    &mut self,
    vulkan: &mut Vulkan,
    buffer_name: T,
    texture: T,
  ) {
    let instance_data = vec![InstancedComboData::new(); MAX_INSTANCES];
    let instanced_combo_buffer =
      Buffer::<InstancedComboData>::new_vertex(vulkan.device(), instance_data);

    self
      .instanced_combo_buffer
      .insert(buffer_name.into(), (texture.into(), instanced_combo_buffer));
  }

  pub fn load_texture<T: Into<String>>(&mut self, vulkan: &mut Vulkan, texture_ref: T, texture: T) {
    let texture = texture.into();

    let image = image::open(&texture)
      .expect(&("Failed to load texture: ".to_string() + &texture))
      .fliph()
      .to_rgba8();

    let dl_texture = TextureHandler::create_device_local_texture_from_image(vulkan, image);

    let descriptor_sets = DescriptorSet::builder()
      .combined_image_sampler_fragment()
      .build(vulkan.device(), &self.descriptor_pool);
    let descriptor_set_writer =
      DescriptorWriter::builder().update_image(&dl_texture, &self.sampler, &descriptor_sets);

    descriptor_set_writer.build(vulkan.device());

    self
      .textures
      .insert(texture_ref.into(), (dl_texture, descriptor_sets));
  }

  pub fn draw(&mut self, vulkan: &mut Vulkan, mut data: Vec<f32>, texture: &str) {
    let texture_descriptor = {
      if let Some((_, texture_descriptor)) = self.textures.get(texture) {
        texture_descriptor
      } else {
        &self.dummy_texture.1
      }
    };

    let last_idx = data.len() - 4;
    data[last_idx] = self.camera_position.x;
    data[last_idx + 1] = self.camera_position.y;

    vulkan.draw_texture(
      &texture_descriptor,
      &self.uniform_descriptor,
      &self.combo_shader,
      &self.combo_vertex_buffer,
      &self.combo_index_buffer,
      None as Option<&Buffer<u32>>,
      0,
      data,
    );
  }

  pub fn add_draw(&mut self, vulkan: &mut Vulkan, mut data: Vec<f32>, texture: &str) {
    let texture_descriptor = {
      if let Some((_, texture_descriptor)) = self.textures.get(texture) {
        texture_descriptor
      } else {
        &self.dummy_texture.1
      }
    };

    let last_idx = data.len() - 4;
    data[last_idx] = self.camera_position.x;
    data[last_idx + 1] = self.camera_position.y;

    vulkan.draw_texture(
      &texture_descriptor,
      &self.uniform_descriptor,
      &self.combo_shader,
      &self.combo_vertex_buffer,
      &self.combo_index_buffer,
      None as Option<&Buffer<u32>>,
      0,
      data,
    );
  }

  pub fn add_text_data(&mut self, mut draw: Draw, vulkan: &mut Vulkan) {
    let size = draw.get_scale().x;
    let position = draw.get_position().xy();
    let colour = draw.get_colour();
    let wrap = draw.get_wrap();
    let centered = draw.get_centered();

    if let Some(raw_text) = draw.get_text() {
      if raw_text.len() == 0
        || raw_text
          .split(' ')
          .map(|s| s.to_string())
          .filter(|s| s == "")
          .collect::<Vec<String>>()
          .len()
          > raw_text.len()
      {
        return;
      }

      let text = GuiText::new(
        raw_text.to_string(),
        size,
        position,
        self.camera_position,
        colour,
        draw.get_coloured_words(),
        wrap,
        centered,
      );

      let text_clone = text.clone();
      self.text_this_draw.push(text_clone);
      self.text_master.load_text(text, vulkan);
    }
  }

  pub fn draw_new_text(&mut self, vulkan: &mut Vulkan) {
    let (texts, vertex_buffers) = self.text_master.text();
    for text in texts {
      let pos = text.position();
      let data = vec![
        pos.x,
        pos.y,
        self.window_size[0],
        self.window_size[1],
        text.camera().x,
        text.camera().y,
      ];

      let descriptor = self.text_master.font().descriptor();
      let font_shader = self.text_master.font().shader();

      vulkan.draw_text(
        &descriptor,
        font_shader,
        &vertex_buffers.get(&text.text()).unwrap(),
        data,
      );
    }

    self.text_master.remove_all_text(vulkan.device());
    //self
    //  .text_master
    //  .remove_unused_text(self.text_this_draw.drain(..).collect(), vulkan.device());
  }

  pub fn draw_instanced_texture(&mut self, vulkan: &mut Vulkan, buffer: &str) {
    if let Some((texture, buffer)) = self.instanced_combo_buffer.get_mut(buffer) {
      let instance_count = buffer.data.len();

      buffer.update_with_internal_data(vulkan.device());

      let texture_descriptor = {
        if let Some((_, texture_descriptor)) = self.textures.get(texture) {
          texture_descriptor
        } else {
          &self.dummy_texture.1
        }
      };

      vulkan.draw_texture(
        &texture_descriptor,
        &self.uniform_descriptor,
        &self.instanced_combo_shader,
        &self.combo_vertex_buffer,
        &self.combo_index_buffer,
        Some(&buffer),
        instance_count,
        vec![self.window_size[0], self.window_size[1]],
      );
    }
    //  let descriptor = self.font.descriptor();

    //  self
    //    .instanced_letter_buffer
    //    .update_data(vulkan.device(), self.instanced_letter_buffer.data.clone());

    //  vulkan.draw_texture(
    //    &descriptor,
    //    &self.uniform_descriptor,
    //    &self.instanced_letter_shader,
    //    &self.combo_vertex_buffer,
    //    &self.combo_index_buffer,
    //    Some(&self.instanced_letter_buffer),
    //    instance_count,
    //    vec![],
    //  );
    //}
  }

  pub fn add_instanced_texture(&mut self, data: Vec<f32>, buffer_name: &str) {
    let mut data = data;
    let last_idx = data.len() - 4;
    data[last_idx] = self.camera_position.x;
    data[last_idx + 1] = self.camera_position.y;

    if let Some((_, buffer)) = self.instanced_combo_buffer.get_mut(buffer_name) {
      if buffer.data.len() < MAX_INSTANCES {
        buffer.data.push(InstancedComboData::from_data(&data));
      }
    }

    //  let text_size = data[2].max(0.1);
    //  let letter_data = self.font.generate_letter_draws(text_size, text.to_string());

    //  let x = data[0];
    //  let y = data[1];

    //  for (x_offset, y_offset, width, height, uvx0, uvy0, uvx1, uvy1, kerning_x, kerning_y) in
    //    letter_data
    //  {
    //    let pos_x = x + x_offset;
    //    let pos_y = y + y_offset;

    //    self.instanced_letter_buffer.data[*idx].pos = [pos_x, pos_y];
    //    self.instanced_letter_buffer.data[*idx].size = [width, height];
    //    self.instanced_letter_buffer.data[*idx].uv = [uvx0, uvy0, uvx1, uvy1];
    //    self.instanced_letter_buffer.data[*idx].text_height = data[2];
    //    self.instanced_letter_buffer.data[*idx].colour = [kerning_x, kerning_y, data[6], data[7]];
    //    //self.instanced_letter_buffer.data[*idx].text_height = data[2];
    //    //self.instanced_letter_buffer.data[*idx].colour = [data[4], data[5], data[6], data[7]];
    //    //      self.instanced_letter_buffer.data[*idx].outline_colour =
    //    //        [data[8], data[9], data[10], data[11]];
    //    //      self.instanced_letter_buffer.data[*idx].width_edge = [data[12], data[13], data[14], data[15]];

    //    *idx += 1;
    //  }
  }

  //pub fn draw_instanced_text(&mut self, vulkan: &mut Vulkan, instance_count: usize) {
  //  let descriptor = self.font.descriptor();

  //  self
  //    .instanced_letter_buffer
  //    .update_data(vulkan.device(), self.instanced_letter_buffer.data.clone());

  //  vulkan.draw_texture(
  //    &descriptor,
  //    &self.uniform_descriptor,
  //    &self.instanced_letter_shader,
  //    &self.combo_vertex_buffer,
  //    &self.combo_index_buffer,
  //    Some(&self.instanced_letter_buffer),
  //    instance_count,
  //    vec![],
  //  );
  //}

  //pub fn add_text_data(&mut self, idx: &mut usize, data: Vec<f32>, text: &str, _texture: &str) {
  //  let mut data = data;

  //  while data.len() < 16 {
  //    data.push(0.0);
  //  }

  //  let text_size = data[2].max(0.1);
  //  let letter_data = self.font.generate_letter_draws(text_size, text.to_string());

  //  let x = data[0];
  //  let y = data[1];

  //  for (x_offset, y_offset, width, height, uvx0, uvy0, uvx1, uvy1, kerning_x, kerning_y) in
  //    letter_data
  //  {
  //    let pos_x = x + x_offset;
  //    let pos_y = y + y_offset;

  //    self.instanced_letter_buffer.data[*idx].pos = [pos_x, pos_y];
  //    self.instanced_letter_buffer.data[*idx].size = [width, height];
  //    self.instanced_letter_buffer.data[*idx].uv = [uvx0, uvy0, uvx1, uvy1];
  //    self.instanced_letter_buffer.data[*idx].text_height = data[2];
  //    self.instanced_letter_buffer.data[*idx].colour = [kerning_x, kerning_y, data[6], data[7]];
  //    //self.instanced_letter_buffer.data[*idx].text_height = data[2];
  //    //self.instanced_letter_buffer.data[*idx].colour = [data[4], data[5], data[6], data[7]];
  //    //      self.instanced_letter_buffer.data[*idx].outline_colour =
  //    //        [data[8], data[9], data[10], data[11]];
  //    //      self.instanced_letter_buffer.data[*idx].width_edge = [data[12], data[13], data[14], data[15]];

  //    *idx += 1;
  //  }
  //}

  pub fn create_checked_image() -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    image::ImageBuffer::from_fn(2, 2, |x, y| {
      if (x + y) % 2 == 0 {
        image::Rgba([0, 0, 0, 255])
      } else {
        image::Rgba([255, 255, 255, 255])
      }
    })
  }

  pub fn create_device_local_texture_from_image(
    vulkan: &mut Vulkan,
    image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
  ) -> Image {
    let dimensions = image.dimensions();
    let image_data = image.into_raw();

    let src_buffer = Buffer::<u8>::new_image(vulkan.device(), image_data);
    let dst_image = ImageBuilder::new(vk::Format::A8B8G8R8_SRGB_PACK32, 1, 1)
      .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
      .set_dimensions(dimensions.0, dimensions.1)
      .build_device_local(vulkan.device());

    vulkan.copy_buffer_to_device_local_image(&src_buffer, &dst_image);

    dst_image
  }

  fn create_combo_shader(
    vulkan: &Vulkan,
    descriptor_sets: &Vec<DescriptorSet>,
  ) -> (
    //Shader<ComboVertex>,
    //Shader<ComboVertex>,
    Shader<ComboVertex>,
    Buffer<u32>,
    Buffer<ComboVertex>,
    Shader<ComboVertex>,
  ) {
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
      },
    ];

    let combo_vertex = ComboVertex {
      pos: [0.0, 0.0, 0.0, 0.0],
      colour: [0.0, 0.0, 0.0, 0.0],
      uv: [0.0, 0.0],
    };
    //let instaced_text = InstancedTextData::new();
    let instanced_combo = InstancedComboData::new();

    let combo_index_buffer = Buffer::<u32>::new_index(&vulkan.device(), combo_index_buffer_data);
    let combo_vertex_buffer = Buffer::<ComboVertex>::new_vertex(vulkan.device(), combo_vertices);

    // let instanced_combo_buffer =
    //   Buffer::<InstancedComboData>::new_vertex(vulkan.device(), instance_data);

    let graphics_pipeline_builder = GraphicsPipelineBuilder::new()
      .topology_triangle_list()
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

    let combo_shader = Shader::new(
      vulkan.device(),
      Cursor::new(&include_bytes!("../../shaders/combo_vert.spv")[..]),
      Cursor::new(&include_bytes!("../../shaders/combo_frag.spv")[..]),
      combo_vertex,
      vec![
        offset_of!(ComboVertex, pos) as u32,
        offset_of!(ComboVertex, colour) as u32,
        offset_of!(ComboVertex, uv) as u32,
      ],
      &graphics_pipeline_builder,
      vulkan.texture_renderpass(),
      vulkan.viewports(),
      vulkan.scissors(),
      &layouts,
      None as Option<(InstancedTextData, Vec<u32>)>,
    );

    let instanced_combo_shader = Shader::new(
      vulkan.device(),
      Cursor::new(&include_bytes!("../../shaders/instanced_combo_vert.spv")[..]),
      Cursor::new(&include_bytes!("../../shaders/combo_frag.spv")[..]),
      combo_vertex,
      vec![
        offset_of!(ComboVertex, pos) as u32,
        offset_of!(ComboVertex, colour) as u32,
        offset_of!(ComboVertex, uv) as u32,
      ],
      &graphics_pipeline_builder,
      vulkan.texture_renderpass(),
      vulkan.viewports(),
      vulkan.scissors(),
      &layouts,
      Some((
        instanced_combo,
        vec![
          // offset_of!(InstancedComboData, pos) as u32,
          // offset_of!(InstancedComboData, colour) as u32,
          // offset_of!(InstancedComboData, uv) as u32,
          offset_of!(InstancedComboData, pos_scale) as u32, // x, y, scale_x, scale_y
          offset_of!(InstancedComboData, other_colour) as u32, // r g b a
          offset_of!(InstancedComboData, is_textured_rotation_overlay_mix) as u32, // is_textured, rotation, overlay_mix, empty
          offset_of!(InstancedComboData, sprite_sheet) as u32, // rows, texture number, empty
          offset_of!(InstancedComboData, flip_xy) as u32,      // flip x y
          offset_of!(InstancedComboData, overlay_colour) as u32, // overlay colour
          offset_of!(InstancedComboData, attrib6) as u32,
          offset_of!(InstancedComboData, camera_intensity_time) as u32, // camera x y, intensity time
        ],
      )),
    );

    //let letter_shader = Shader::new(
    //  vulkan.device(),
    //  Cursor::new(&include_bytes!("../../shaders/letter_sdf_vert.spv")[..]),
    //  Cursor::new(&include_bytes!("../../shaders/letter_sdf_frag.spv")[..]),
    //  combo_vertex,
    //  vec![
    //    offset_of!(ComboVertex, pos) as u32,
    //    offset_of!(ComboVertex, colour) as u32,
    //    offset_of!(ComboVertex, uv) as u32,
    //  ],
    //  &graphics_pipeline_builder,
    //  vulkan.texture_renderpass(),
    //  vulkan.viewports(),
    //  vulkan.scissors(),
    //  &layouts,
    //  None as Option<(InstancedTextData, Vec<u32>)>,
    //);
    //let instanced_letter_shader = Shader::new(
    //  vulkan.device(),
    //  Cursor::new(&include_bytes!("../../shaders/instanced_letter_sdf_vert.spv")[..]),
    //  Cursor::new(&include_bytes!("../../shaders/letter_sdf_frag.spv")[..]),
    //  combo_vertex,
    //  vec![
    //    offset_of!(ComboVertex, pos) as u32,
    //    offset_of!(ComboVertex, colour) as u32,
    //    offset_of!(ComboVertex, uv) as u32,
    //  ],
    //  &graphics_pipeline_builder,
    //  vulkan.texture_renderpass(),
    //  vulkan.viewports(),
    //  vulkan.scissors(),
    //  &layouts,
    //  Some((
    //    instaced_text,
    //    vec![
    //      offset_of!(InstancedTextData, pos) as u32,
    //      offset_of!(InstancedTextData, size) as u32,
    //      offset_of!(InstancedTextData, uv) as u32,
    //      //          offset_of!(InstancedTextData, text_height) as u32,
    //      //          offset_of!(InstancedTextData, colour) as u32,
    //      //          offset_of!(InstancedTextData, outline_colour) as u32,
    //      //          offset_of!(InstancedTextData, width_edge) as u32,
    //    ],
    //  )),
    //);

    (
      //letter_shader,
      //instanced_letter_shader,
      combo_shader,
      combo_index_buffer,
      combo_vertex_buffer,
      instanced_combo_shader,
      //instanced_combo_buffer,
    )
  }
}
