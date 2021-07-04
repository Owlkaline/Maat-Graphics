use std::fs::File;
use std::io::{BufRead, BufReader};

use image;
use ash::vk;

use crate::modules::{Vulkan, Buffer, DescriptorSet, Sampler, DescriptorPoolBuilder, DescriptorWriter};
use crate::modules::Image as vkImage;
use crate::shader_handlers::{TextureHandler, ComboVertex};

#[derive(Debug)]
pub struct FontChar {
  pub x1: f32,
  pub y1: f32,
  pub x2: f32,
  pub y2: f32,
  pub width: u32,
  pub height: u32,
  pub x_offset: i32,
  pub y_offset: i32,
  pub x_advance: i32, // not used?
  pub page: u32,
}

impl FontChar {
  pub fn new_empty() -> FontChar {
    FontChar {
      x1: 0.0,
      y1: 0.0,
      x2: 0.0,
      y2: 0.0,
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
  
  width: u32,
  height: u32,
  // line height of font
  pub line_height: u32,
  
  // size of font (width)
  pub size: u32,
  
  pub min_offset_y: i32,
  pub avg_xadvance: f32,
}

impl Font {
  pub fn new(vulkan: &mut Vulkan, sampler: &Sampler) -> Font {
    Font::load_font(vulkan, sampler)
  }
  
  fn load_font(vulkan: &mut Vulkan, sampler: &Sampler) -> Font {
    let location = "./fonts/SourceCodePro";
    
    let image = image::open(location.to_owned() + ".png").expect(&("Failed to load font: ".to_string() + location)).fliph().to_rgba8();
    let image_width = image.width() as f32;
    let image_height = image.height() as f32;
    
    let mut min_off_y = 100000;
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
        
        while(idx+1 > font_chars.len()) {
          font_chars.push(FontChar::new_empty());
        }
        
        let mut x = Font::value_from_string_pair(segments[0].to_string()) as u32;
        let mut y = Font::value_from_string_pair(segments[1].to_string()) as u32;
        let mut width = Font::value_from_string_pair(segments[2].to_string()) as u32;
        let mut height = Font::value_from_string_pair(segments[3].to_string()) as u32;
        let mut x_offset = Font::value_from_string_pair(segments[4].to_string());
        let mut y_offset = Font::value_from_string_pair(segments[5].to_string());
        let mut x_advance = Font::value_from_string_pair(segments[6].to_string());
        let mut page = Font::value_from_string_pair(segments[7].to_string()) as u32;
        
        let x1  = x as f32;//x as f32 / image_width;
        let x2  = x as f32;//(x as f32 + width as f32) / image_width;
        let y1  = y as f32;//y as f32 / image_height;
        let y2  = y as f32;//(y as f32 + height as f32) / image_height;
        if y_offset < min_off_y {
          min_off_y = y_offset;
        }
        xadvance_sum += x_advance as f32;
        
        font_chars[idx] = FontChar {
          x1,
          y1,
          x2,
          y2,
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
    let font_descriptor_set_writer = DescriptorWriter::builder()
                                                       .update_image(&font_texture, &sampler, &font_descriptor_set);
    
    font_descriptor_set_writer.build(vulkan.device());
    
    Font {
      chars: font_chars,
      texture: font_texture,
      descriptor_set: font_descriptor_set,
      descriptor_pool,
      
      width: image_width as u32,
      height: image_height as u32,
      
      line_height,
      size: 64, // TODO: Load from file
      
      min_offset_y: min_off_y,
      avg_xadvance: avg_advance,
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
    
    let mut letter_idx = 0;
    
    let mut pos_x = 0.0;
    
    for c in text.chars() {
      let mut char_info = &mut self.chars[c as i32 as usize];
      
      let w = self.texture.width() as f32;
      
      let line_height = self.line_height as f32;
      
      let width = text_size * (char_info.width as f32 / self.avg_xadvance);
      let height = text_size * (char_info.height as f32 / line_height);
      
      let x_offset = text_size * (char_info.x_offset as f32 / self.avg_xadvance);
      let y_offset = text_size * ((char_info.y_offset as f32/self.size as f32) / line_height);
      
      let x = pos_x - x_offset;
      //println!("{}: char_height: {} y_offset: {} line height: {}", c, char_info.height, char_info.y_offset, line_height);
      let y = y_offset;
      
      let us = (w-char_info.x1 as f32)/ w;
      let ue = ((w-char_info.x1 as f32) - char_info.width as f32) / w;
      let ts = char_info.y1 as f32 / w;
      let te = (char_info.y1 as f32 + char_info.height as f32) / w;
      
      let uv_x0 = ue;
      let uv_x1 = us;
      let uv_y0 = ts;
      let uv_y1 = te;
      
      text_data.push((x, -y, width, height, uv_x0, uv_x1, uv_y0, uv_y1));
      
      pos_x += text_size * (char_info.x_advance as f32 / self.size as f32);
      letter_idx += 1;
    }
    
    /*
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
    }*/
    
    text_data
  }
  /*
  pub fn generate_text(&mut self, vulkan: &mut Vulkan, text_size: f32, text: &str) -> (Buffer::<u32>, Buffer::<ComboVertex>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    vertices.push(
      ComboVertex {
        pos: [0.0, 0.0, 0.0, 0.0],
        colour: [1.0, 1.0, 1.0, 1.0],
        uv: [0.0, 0.0],
      }
    );
    indices.push(0);
    
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
  }*/
}









