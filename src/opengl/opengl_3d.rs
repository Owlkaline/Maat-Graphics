use gl;
use gl::types::*;

use image;

use std::mem;

pub fn create_texture_from_dynamicimage(data: Option<image::DynamicImage>) -> Option<GLuint> {
  let mut final_texture: Option<GLuint> = None;
  
  let mut texture_id = 0;
  
  if data.is_some() {
    unsafe {
      gl::GenTextures(1, &mut texture_id);
      
      gl::BindTexture(gl::TEXTURE_2D, texture_id);
      
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
      
      let texture_img = data.clone().unwrap().to_rgba();
      let dim = texture_img.dimensions();
      let image_data = texture_img.into_raw().clone();
      gl::TexImage2D(gl::TEXTURE_2D, 0,
                    gl::RGBA as GLint,
                    dim.0 as GLsizei,
                    dim.1 as GLsizei,
                    0, gl::RGBA, gl::UNSIGNED_BYTE,
                    mem::transmute(&image_data[0]));
      gl::GenerateMipmap(gl::TEXTURE_2D);
      
      gl::BindTexture(gl::TEXTURE_2D, 0);
      final_texture = Some(texture_id);
    }
  }
  
  final_texture
}
