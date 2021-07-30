use std::fs::File;
use std::io::{BufRead, BufReader};

use ash::vk;
use image;

use crate::shader_handlers::TextureHandler;
use crate::vkwrapper::Image as vkImage;
use crate::vkwrapper::{DescriptorPoolBuilder, DescriptorSet, DescriptorWriter, Sampler, Vulkan};

#[derive(Clone)]
pub struct FontChar {
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
  pub x_offset: f32,
  pub y_offset: f32,
  pub x_advance: f32, // not used?
  pub page: u32,
}

impl FontChar {
  pub fn new_empty() -> FontChar {
    FontChar {
      x: 0.0,
      y: 0.0,
      width: 0.0,
      height: 0.0,
      x_offset: 0.0,
      y_offset: 0.0,
      x_advance: 0.0,
      page: 0,
    }
  }
}

pub struct Font {
  chars: Vec<FontChar>,
  texture: vkImage,
  descriptor_set: DescriptorSet,
  descriptor_pool: vk::DescriptorPool,

  width: u32,
  height: u32,
  // line height of font
  pub line_height: u32,

  // size of font (width)
  pub size: u32,

  pub min_offset_y: f32,
  pub avg_xadvance: f32,
}

impl Font {
  pub fn new(vulkan: &mut Vulkan, sampler: &Sampler) -> Font {
    Font::load_font(vulkan, sampler)
  }

  fn load_font(vulkan: &mut Vulkan, sampler: &Sampler) -> Font {
    let location = "./fonts/dejavasans"; //DOSVGA"; //SourceCodePro";

    let image = image::open(location.to_owned() + ".png")
      .expect(&("Failed to load font: ".to_string() + location))
      .fliph()
      .to_rgba8();
    let image_width = image.width() as f32;
    let image_height = image.height() as f32;

    let mut min_off_y = 100000.0;
    let mut xadvance_sum = 0.0;

    let mut line_height = 1;

    let mut font_chars = Vec::new();

    let file = File::open(location.to_owned() + ".fnt").unwrap();
    let buffer_reader = BufReader::new(file);

    let mut num_chars = 0;
    for line in buffer_reader.lines() {
      let line = line.unwrap();
      let mut segments: Vec<&str> = line.split(' ').filter(|s| *s != "").collect();

      let segment_0 = segments.remove(0);

      if segment_0.contains("char") && !segments[0].contains("count") {
        let idx = Font::value_from_string_pair(segments.remove(0).to_string()) as usize;

        while idx + 1 > font_chars.len() {
          font_chars.push(FontChar::new_empty());
        }

        let x = Font::value_from_string_pair(segments[0].to_string()) as f32 / 512.0;
        let y = Font::value_from_string_pair(segments[1].to_string()) as f32 / 512.0;
        let width = Font::value_from_string_pair(segments[2].to_string()) as f32 / 512.0;
        let height = Font::value_from_string_pair(segments[3].to_string()) as f32 / 512.0;
        let x_offset = Font::value_from_string_pair(segments[4].to_string()) as f32 / 512.0;
        let y_offset = Font::value_from_string_pair(segments[5].to_string()) as f32 / 512.0;
        let x_advance =
          width / (Font::value_from_string_pair(segments[6].to_string()) as f32 / 512.0);
        let page = Font::value_from_string_pair(segments[7].to_string()) as u32;

        if y_offset < min_off_y {
          min_off_y = y_offset;
        }
        xadvance_sum += x_advance as f32;

        font_chars[idx] = FontChar {
          x,
          y,
          width,
          height,
          x_offset,
          y_offset,
          x_advance, // not used?
          page,
        };

        num_chars += 1;
      } else {
        if segment_0.contains("common") {
          line_height = Font::value_from_string_pair(segments[0].to_string()) as u32;
        }
      }
    }

    let avg_advance = xadvance_sum / num_chars as f32;

    let descriptor_pool = DescriptorPoolBuilder::new()
      .num_combined_image_samplers(1)
      .build(vulkan.device());

    let font_texture = TextureHandler::create_device_local_texture_from_image(vulkan, image);
    let font_descriptor_set = DescriptorSet::builder()
      .combined_image_sampler_fragment()
      .build(vulkan.device(), &descriptor_pool);
    let font_descriptor_set_writer =
      DescriptorWriter::builder().update_image(&font_texture, &sampler, &font_descriptor_set);

    font_descriptor_set_writer.build(vulkan.device());

    Font {
      chars: font_chars,
      texture: font_texture,
      descriptor_set: font_descriptor_set,
      descriptor_pool,

      width: image_width as u32,
      height: image_height as u32,

      line_height,
      size: 71, // TODO: Load from file

      min_offset_y: min_off_y,
      avg_xadvance: avg_advance,
    }
  }

  pub fn get_font_data(&self) -> (Vec<FontChar>, u32, u32) {
    (self.chars.clone(), self.line_height, self.size)
  }

  pub fn descriptor(&self) -> &DescriptorSet {
    &self.descriptor_set
  }

  fn value_from_string_pair(string: String) -> i32 {
    string
      .split('=')
      .collect::<Vec<&str>>()
      .last()
      .unwrap()
      .parse::<i32>()
      .unwrap()
  }

  pub fn generate_letter_draws(
    &mut self,
    text_size: f32,
    text: String,
  ) -> Vec<(f32, f32, f32, f32, f32, f32, f32, f32)> {
    let mut text_data = Vec::new();

    let mut pos_x = 0.0;

    for c in text.chars() {
      let char_info = &mut self.chars[c as i32 as usize];

      if char_info.width == 0.0 || char_info.height == 0.0 {
        pos_x += text_size;
        continue
      }

      let w_h_ratio = char_info.height / char_info.width;

      let height = text_size;
      let width = text_size * w_h_ratio;

      let x = pos_x + char_info.x_offset * width;
      let y = (1.0 - char_info.y_offset) * height - height;

      let uv_x0 = char_info.x;
      let uv_y0 = 1.0 - char_info.y;
      let uv_x1 = char_info.x + char_info.width;
      let uv_y1 = (1.0 - char_info.y) - char_info.height;

      text_data.push((
        x,
        y,
        width,
        height,
        1.0 - uv_x0,
        1.0 - uv_y1,
        1.0 - uv_x1,
        1.0 - uv_y0,
      ));

      pos_x += char_info.x_advance * width;
    }

    text_data
  }
}
