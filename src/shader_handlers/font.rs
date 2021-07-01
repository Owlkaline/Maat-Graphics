use std::fs::File;
use std::io::{BufRead, BufReader};

use image;
use ash::vk;

use crate::modules::{Vulkan, Buffer, DescriptorSet, Sampler, DescriptorPoolBuilder, DescriptorWriter};
use crate::modules::Image as vkImage;
use crate::shader_handlers::{TextureHandler, ComboVertex};

#[derive(Debug)]
pub struct FontChar {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
  pub x_offset: i32,
  pub y_offset: i32,
  pub x_advance: i32,
  pub page: u32,
}

impl FontChar {
  pub fn new_empty() -> FontChar {
    FontChar {
      x: 0,
      y: 0,
      width: 0,
      height: 0,
      x_offset: 0,
      y_offset: 0,
      x_advance: 0,
      page: 0,
    }
  }
}

pub struct  Font {
  chars: Vec<FontChar>,
  texture: vkImage,
  descriptor_set: DescriptorSet,
  descriptor_pool: vk::DescriptorPool,
}

impl Font {
  pub fn new(vulkan: &mut Vulkan, sampler: &Sampler) -> Font {
    Font::load_font(vulkan, sampler)
  }
  
  fn load_font(vulkan: &mut Vulkan, sampler: &Sampler) -> Font {
    let location = "./fonts/DOSVGA";
    
    let mut font_chars = Vec::new();
    
    let file = File::open(location.to_owned() + ".fnt").unwrap();
    let buffer_reader = BufReader::new(file);
    
    for line in buffer_reader.lines() {
      let line = line.unwrap();
      let mut segments: Vec<&str> = line.split(' ').filter(|s| *s != "").collect();
      
      if segments.remove(0).contains("char") && !segments[0].contains("count") {
        let idx = Font::value_from_string_pair(segments.remove(0).to_string()) as usize;
        
        while(idx+1 > font_chars.len()) {
          font_chars.push(FontChar::new_empty());
        }
        
        font_chars[idx] = FontChar {
          x: Font::value_from_string_pair(segments[0].to_string()) as u32,
          y: Font::value_from_string_pair(segments[1].to_string()) as u32,
          width: Font::value_from_string_pair(segments[2].to_string()) as u32,
          height: Font::value_from_string_pair(segments[3].to_string()) as u32,
          x_offset: Font::value_from_string_pair(segments[4].to_string()),
          y_offset: Font::value_from_string_pair(segments[5].to_string()),
          x_advance: Font::value_from_string_pair(segments[6].to_string()),
          page: Font::value_from_string_pair(segments[7].to_string()) as u32,
        };
      }
    }
    
    let descriptor_pool = DescriptorPoolBuilder::new()
                                              .num_combined_image_samplers(1)
                                              .build(vulkan.device());
    
    let image = image::open(location.to_owned() + ".png").expect(&("Failed to load font: ".to_string() + location)).fliph().to_rgba8();
    let font_texture = TextureHandler::create_device_local_texture_from_image(vulkan, image);
    let font_descriptor_set = DescriptorSet::builder()
                                      .combined_image_sampler_fragment()
                                      .build(vulkan.device(), &descriptor_pool);
    let font_descriptor_set_writer = DescriptorWriter::builder()
                                                       .update_image(&font_texture, &sampler, &font_descriptor_set);
    
    font_descriptor_set_writer.build(vulkan.device());
    
    Font {
      chars: font_chars,
      texture: font_texture,
      descriptor_set: font_descriptor_set,
      descriptor_pool,
    }
  }
  
  pub fn descriptor(&self) -> &DescriptorSet {
    &self.descriptor_set
  }
  
  fn value_from_string_pair(string: String) -> i32 {
    string.split('=').collect::<Vec<&str>>().last().unwrap().parse::<i32>().unwrap()
  }
  
  pub fn generate_letter_draws(&mut self, text_size: f32, text: String) -> Vec<(f32, f32, f32, f32, f32, f32, f32, f32)> {
    let mut text_data = Vec::new();
    
    let mut index_offset = 0;
    
    let w = self.texture.width() as f32;
    
    let mut pos_x = 0.0;
    let mut pos_y;
    
    let mut base_size = 36.0;
    
    for c in text.chars() {
      let mut char_info = &mut self.chars[c as i32 as usize];
      
      if char_info.width == 0 {
        char_info.width = 36;
      }
      
      let charw = char_info.width as f32 / base_size;
      let dimx = text_size * charw;
      let charh = char_info.height as f32 / base_size;
      let dimy = text_size * charh;
      
      let us = (w-char_info.x as f32)/ w;
      let ue = ((w-char_info.x as f32) - char_info.width as f32) / w;
      let ts = char_info.y as f32 / w;
      let te = (char_info.y as f32 + char_info.height as f32) / w;
      
      let xo = char_info.x_offset as f32 / base_size;
      let yo = char_info.y_offset as f32 / base_size;
      
      pos_y = -yo;
      
      let x = pos_x + xo;
      let y = pos_y;
      let width = dimx;
      let height = dimy;
      let uv_x0 = ue;
      let uv_x1 = us;
      let uv_y0 = ts;
      let uv_y1 = te;
      
      text_data.push((x, y, width, height, uv_x0, uv_x1, uv_y0, uv_y1));
      
      let advance = char_info.x_advance as f32/base_size * text_size;
      pos_x += advance;
    }
    
    text_data
  }
  
  pub fn generate_text(&mut self, vulkan: &mut Vulkan, text_size: f32, text: &str) -> (Buffer::<u32>, Buffer::<ComboVertex>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let mut index_offset = 0;
    
    let w = self.texture.width() as f32;
    
    let mut pos_x = 0.0;
    let mut pos_y;
    
    let mut base_size = 36.0;
    
    let text = text.to_string() + " ";
    for c in text.chars() {
      let mut char_info = &mut self.chars[c as i32 as usize];
      
      if char_info.width == 0 {
        char_info.width = 36;
      }
      
      let charw = char_info.width as f32 / base_size;
      let dimx = text_size * charw;
      let charh = char_info.height as f32 / base_size;
      let dimy = text_size * charh;
      
      let us = (w-char_info.x as f32)/ w;
      let ue = ((w-char_info.x as f32) - char_info.width as f32) / w;
      let ts = char_info.y as f32 / w;
      let te = (char_info.y as f32 + char_info.height as f32) / w;
      
      let xo = char_info.x_offset as f32 / base_size;
      let yo = char_info.y_offset as f32 / base_size;
      
      pos_y = -yo;
      
      let z = -1.0;
      let w = 1.0;
      
      vertices.push(
        ComboVertex {
          pos: [pos_x + dimx + xo, pos_y + dimy, z, w],
          colour: [1.0, 1.0, 1.0, 1.0],
          uv: [ue, ts],
        }
      );
      
      vertices.push(
        ComboVertex {
          pos: [pos_x + xo, pos_y + dimy, z, w],
          colour: [1.0, 1.0, 1.0, 1.0],
          uv: [us, ts],
        }
      );
      
      vertices.push(
        ComboVertex {
          pos: [pos_x + xo, pos_y, z, w],
          colour: [1.0, 1.0, 1.0, 1.0],
          uv: [us, te],
        }
      );
      
      vertices.push(
        ComboVertex {
          pos: [pos_x + dimx + xo, pos_y, z, w],
          colour: [1.0, 1.0, 1.0, 1.0],
          uv: [ue, te],
        }
      );
      
      let letter_indices = vec!(0, 1, 2, 2, 3, 0);
      for letter in letter_indices {
        indices.push(index_offset + letter as i32 as u32);
      }
      index_offset += 4;
      
      let advance = char_info.x_advance as f32/base_size * text_size;
      pos_x += advance;
    }
    
    //for v in &mut vertices {
     // v.pos[0] -= pos_x / 2.0;
    //  v.pos[1] -= 0.5;
    //}
    
    let text_index_buffer = Buffer::<u32>::new_index(&vulkan.device(), indices);
    let text_vertex_buffer = Buffer::<ComboVertex>::new_vertex(vulkan.device(), vertices);
    
    (text_index_buffer, text_vertex_buffer)
  }
}









