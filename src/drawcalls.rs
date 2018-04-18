use font::GenericCharacter;
use font::GenericFont;

use graphics;

use cgmath::Deg;
use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix2;
use cgmath::Matrix4;
use cgmath::InnerSpace;

use std::collections::HashMap;

#[derive(Clone, PartialEq)]
enum DrawType {
  SQUARE,
  CUSTOM_VAO,
  UPDATE_VAO,
  TEXT,
  MODEL,
  INSTANCED,
}

#[derive(Clone)]
pub struct DrawCall {
  position: Vector3<f32>,
  rotation: Vector3<f32>,
  size: Vector2<f32>,
  texture: String,
  colour: Vector4<f32>,
  outline_colour: Vector3<f32>,
  text: String,
  text_wrapping: i32,
  centered: bool,
  edge_width: Vector4<f32>,
  draw_type: DrawType,
  new_vao: (Vec<graphics::Vertex2d>, Vec<u16>),
}

impl DrawCall {
  pub fn update_vao(verticies: Vec<graphics::Vertex2d>, indices: Vec<u16>, custom_vao: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(0.0, 0.0, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      size: Vector2::new(0.0, 0.0),
      texture: String::from(""),
      colour: Vector4::new(0.0, 0.0, 0.0, -1.0),
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: custom_vao,
      text_wrapping: 0,
      centered: false,
      edge_width: Vector4::new(0.1, 0.1, 0.1, 0.1),
      draw_type: DrawType::UPDATE_VAO,
      new_vao: (verticies, indices),
    }
  }
  
  pub fn instanced_textured_draw(x: f32, y: f32, texture: String, reference: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(x, y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      size: Vector2::new(0.0, 0.0),
      texture: texture,
      colour: Vector4::new(1.0, 1.0, 1.0, -1.0),
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: reference, // Instance reference name
      text_wrapping: 0,
      centered: false,
      edge_width: Vector4::new(0.0, 0.0, 0.0, 0.0),
      draw_type: DrawType::INSTANCED,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn new_draw(x: f32, y: f32, z: f32) -> DrawCall {
    DrawCall {
      position: Vector3::new(x, y, z),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      size: Vector2::new(0.0, 0.0),
      texture: String::from(""),
      colour: Vector4::new(0.0, 0.0, 0.0, -1.0),
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: String::from(""),
      text_wrapping: 0,
      centered: false,
      edge_width: Vector4::new(0.1, 0.1, 0.1, 0.1),
      draw_type: DrawType::SQUARE,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn new_custom_textured_draw(x: f32, y: f32, texture: String, custom_vao: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(x, y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      size: Vector2::new(0.0, 0.0),
      texture: texture,
      colour: Vector4::new(0.0, 0.0, 0.0, -1.0),
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: custom_vao,
      text_wrapping: 0,
      centered: false,
      edge_width: Vector4::new(0.1, 0.1, 0.1, 0.1),
      draw_type: DrawType::SQUARE,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn new_custom_draw(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32, custom_vao: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(x, y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      size: Vector2::new(0.0, 0.0),
      texture: String::from(""),
      colour: Vector4::new(r, g, b, a),
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: custom_vao,
      text_wrapping: 0,
      centered: false,
      edge_width: Vector4::new(0.1, 0.1, 0.1, 0.1),
      draw_type: DrawType::CUSTOM_VAO,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn text(x: f32, y: f32, text: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(x, y, 0.0),
      rotation: Vector3::new(0.0, 0.0, 0.0),
      size: Vector2::new(128.0, 128.0),
      texture: String::from("Arial"),
      colour: Vector4::new(0.0, 0.0, 0.0, -1.0),
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: text,
      text_wrapping: 0,
      centered: false,
      edge_width: Vector4::new(0.5, 0.1, 0.1, 0.1),
      draw_type: DrawType::TEXT,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn model(x: f32, y: f32, z: f32) -> DrawCall {
    DrawCall {
      position: Vector3::new(x, y, z),
      rotation: Vector3::new(0.0, 0.0, 0.0),
      size: Vector2::new(0.0, 0.0),
      texture: String::from(""),
      colour: Vector4::new(0.0, 0.0, 0.0, -1.0),
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: String::from(""),
      text_wrapping: 0,
      centered: false,
      edge_width: Vector4::new(0.0, 0.0, 0.0, 0.0),
      draw_type: DrawType::MODEL,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn texture(x: f32, y: f32, texture: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(x, y, 0.0),
      rotation: Vector3::new(90.0, 0.0, 0.0),
      size: Vector2::new(0.0, 0.0),
      texture: texture,
      colour: Vector4::new(1.0, 1.0, 1.0, -1.0),
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: String::from(""),
      text_wrapping: 0,
      centered: false,
      edge_width: Vector4::new(0.0, 0.0, 0.0, 0.0),
      draw_type: DrawType::SQUARE,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn center_text(mut self) -> DrawCall {
    self.centered = true;
    self
  }
  
  pub fn with_colour(mut self, r: f32, g: f32, b: f32, a: f32) -> DrawCall {
    self.colour = Vector4::new(r, g, b, a);
    self
  }
  
  pub fn with_texture(mut self, texture: String) -> DrawCall {
    self.texture = texture;
    self
  }
  
  pub fn with_font(mut self, font: String) -> DrawCall {
    self.texture = font;
    self
  }
  
  pub fn with_scale(mut self, x_scale: f32, y_scale: f32) -> DrawCall {
    self.size.x = x_scale;
    self.size.y = y_scale;
    self
  }
  
  pub fn with_2d_rotation(mut self, rot: f32) -> DrawCall {
    self.rotation.x = rot;
    self
  }
  
  pub fn with_x_rotation(mut self, x: f32) -> DrawCall {
    self.rotation.x = x;
    self
  }
  
  pub fn with_y_rotation(mut self, y: f32) -> DrawCall {
    self.rotation.y = y;
    self
  }
  
  pub fn with_z_rotation(mut self, z: f32) -> DrawCall {
    self.rotation.z = z;
    self
  }

  pub fn with_text(mut self, text: String) -> DrawCall {
    self.text = text;
    self  
  }

  pub fn with_outline_colour(mut self, r: f32, g: f32, b: f32) -> DrawCall {
    self.outline_colour = Vector3::new(r, g, b);
    self
  }
  
  pub fn with_text_wrap(mut self, wrap_length: i32) -> DrawCall {
    self.text_wrapping = wrap_length;
    self
  }
  
  pub fn with_text_edge_info(mut self, fatness: f32, edge_fade: f32, outline_fatness: f32, outline_fade: f32) -> DrawCall {
    self.edge_width = Vector4::new(fatness, edge_fade, outline_fatness, outline_fade);
    self
  }
  
  pub fn new_plain_text(text: String, x: f32, y: f32, size: f32, wrap_length: i32, colour: Vector4<f32>, centered: bool, font: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(x,y, 0.0), 
      rotation: Vector3::new(0.0, 0.0, 0.0),
      size: Vector2::new(size,size), 
      texture: font, 
      colour: colour,
      outline_colour: Vector3::new(0.0, 0.0, 0.0),
      text: text,
      text_wrapping: wrap_length,
      centered: centered,
      edge_width: Vector4::new(0.5, 0.1, 0.1, 0.1),
      draw_type: DrawType::TEXT,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn new_outlined_text(text: String, x: f32, y: f32, size: f32, wrap_length: i32, colour: Vector4<f32>, outline_colour: Vector3<f32>, centered: bool, font: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(x,y, 0.0), 
      rotation: Vector3::new(0.0, 0.0, 0.0),
      size: Vector2::new(size,size), 
      texture: font, 
      colour: colour,
      outline_colour: outline_colour,
      text: text,
      text_wrapping: wrap_length,
      centered: centered,
      edge_width: Vector4::new(0.5, 0.1, 0.7, 0.1),
      draw_type: DrawType::TEXT,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn new_custom_text(text: String, x: f32, y: f32, size: f32, wrap_length: i32, colour: Vector4<f32>, outline_colour: Vector3<f32>, edge_width: Vector4<f32>, centered: bool, font: String) -> DrawCall {
    DrawCall {
      position: Vector3::new(x,y,0.0),
      rotation: Vector3::new(0.0, 0.0, 0.0),
      size: Vector2::new(size,size),
      texture: font,
      colour: colour,
      outline_colour: outline_colour,
      text: text,
      text_wrapping: wrap_length,
      centered: centered,
      edge_width: edge_width, // (Fatness, Edge fade, outline Fatness, outline fade away)
      // GLOW EFFECT (0.4, 0.1, 0.4, 0.6)
      draw_type: DrawType::TEXT,
      new_vao: (Vec::new(), Vec::new()),
    }
  }
  
  pub fn get_text(&self) -> &String {
    &self.text
  }
  
  pub fn get_text_clone(&self) -> String {
    self.text.clone()
  }
  
  pub fn get_translation(&self) -> Vector3<f32> {
    self.position
  }
  
  pub fn get_texture(&self) -> &String {
    &self.texture
  }
  
  pub fn get_colour(&self) -> Vector4<f32> {
    self.colour
  }
  
  pub fn get_outline_colour(&self) -> Vector3<f32> {
    self.outline_colour
  }
  
  pub fn get_edge_width(&self) -> Vector4<f32> {
    self.edge_width
  }
  
  pub fn get_size(&self) -> Vector2<f32> {
    self.size
  }
  
  pub fn get_x_size(&self) -> f32 {
    self.size.x
  }
  
  pub fn get_wrap_length(&self) -> i32 {
    self.text_wrapping
  }
  
  pub fn set_x(&mut self, new_x: f32) {
    self.position.x = new_x;
  }
  
  pub fn set_y(&mut self, new_y: f32) {
    self.position.y = new_y;
  }
  
  pub fn is_centered(&self) -> bool {
    self.centered
  }
  
  pub fn get_texture_unref(&self) -> String {
    (*self.texture).to_string()
  }
  
  pub fn get_x_rotation(&self) -> f32 {
    self.rotation.x
  }
  
  pub fn get_y_rotation(&self) -> f32 {
    self.rotation.y
  }
  
  pub fn get_z_rotation(&self) -> f32 {
    self.rotation.z
  }
  
  pub fn get_new_vertices(&self) -> Vec<graphics::Vertex2d> {
    self.new_vao.clone().0
  }
  
  pub fn get_new_indices(&self) -> Vec<u16> {
    self.new_vao.clone().1
  }
  
  pub fn get_instance_reference(&self) -> String {
    self.text.clone()
  }
  
  pub fn is_text(&self) -> bool {
    self.draw_type == DrawType::TEXT
  }
  
  pub fn is_3d_model(&self) -> bool {
    self.draw_type == DrawType::MODEL
  }
  
  pub fn is_custom_vao(&self) -> bool {
    self.draw_type == DrawType::CUSTOM_VAO
  }
  
  pub fn is_vao_update(&self) -> bool {
    self.draw_type == DrawType::UPDATE_VAO
  }
  
  pub fn is_instanced(&self) -> bool {
    self.draw_type == DrawType::INSTANCED
  }
} 

pub struct DrawMath {
  
}

impl DrawMath {
  pub fn get_text_length(text: String, size: f32, font: String, fonts: HashMap<String, GenericFont>) -> f32 {
    let mut length = 0.0;
    
    for letter in text.as_bytes() {
      let c = fonts.get(&font).unwrap().get_character(*letter as i32);
      
      length+=c.get_advance() as f32 * (size/640.0); 
    }
    
    length
  }
  
  pub fn setup_correct_wrapping(draw: DrawCall, fonts: HashMap<String, GenericFont>) -> Vec<DrawCall> {
  
    let mut new_draw_calls: Vec<DrawCall> = Vec::new();
    
    let init_translation = {
      let mut temp = 0.0;
      if draw.is_centered() {
        temp = DrawMath::get_text_length(draw.get_text().clone(), draw.get_size().x, draw.get_texture().clone(), fonts.clone())*0.5;
      }
      draw.get_translation().x-temp
    };
    
    let mut translation = draw.get_translation();
    translation.x = init_translation;
    
    //let mut position = 0;
    //let mut number_of_words = 0;
    //let finish_at_next_space = 0;
    //let mut last_space_position = 0;
    
    let mut y_offset = 0.0;
    
    let size = draw.get_x_size();
    
    if draw.get_wrap_length() <= 0 {
      for letter in draw.get_text().as_bytes() {
        let c = fonts.get(draw.get_texture()).expect("Unkown Text type").get_character(*letter as i32);
        
        //let size = draw.get_x_size();
        
        if *letter != ' ' as u8 {
          new_draw_calls.push(DrawCall::new_custom_text((*letter as char).to_string(), translation.x, translation.y+y_offset, size, 0, draw.get_colour(), draw.get_outline_colour(), draw.get_edge_width(), false, draw.get_texture_unref()));
        }
        translation.x+=c.get_advance() as f32 * (size/640.0); 
      }
    } else {
      // for wrapping
      for letter in draw.get_text().as_bytes() {
        let c = fonts.get(draw.get_texture()).unwrap().get_character(*letter as i32);
        
        if *letter == ' ' as u8 {
          let distance = translation.x - init_translation;
          if distance > draw.get_wrap_length() as f32 {
            // new line
            translation.x = init_translation;
            y_offset += (size/10.0) * -1.0;//-32.0
          
           /* let temp_diff = new_draw_calls[last_space_position-number_of_words]
            for i in last_space_position-number_of_words..position-num_of_words {
              
            }*/
          } else {
            translation.x+=c.get_advance() as f32 * (size/640.0); 
          }

          //last_space_position = position;
         // number_of_words += 1;
        } else {
                
          new_draw_calls.push(DrawCall::new_custom_text((*letter as char).to_string(), translation.x, translation.y+y_offset, size, 0, draw.get_colour(), draw.get_outline_colour(), draw.get_edge_width(), false, draw.get_texture_unref()));
        
          translation.x+=c.get_advance() as f32 * (size/640.0); 
        }        
        //position += 1;
      }
    }
              
    new_draw_calls
  }
  
  pub fn calculate_texture_model(translation: Vector3<f32>, size: Vector2<f32>, rotation: f32) -> Matrix4<f32> {
    let axis_z = Vector3::new(0.0, 0.0, 1.0).normalize();
    let rotation: Matrix4<f32> = Matrix4::from_axis_angle(axis_z, Deg(450.0-rotation));
    
    let mut model = Matrix4::from_translation(translation)*rotation;
    model = model * Matrix4::from_nonuniform_scale(size.x, size.y, 1.0);
    model
  }
  
  pub fn calculate_text_model(translation: Vector3<f32>, size: f32, c: &GenericCharacter, letter: u8) -> Matrix4<f32> {
    let x_offset: f32 = c.get_offset_x()*size;
    let y_offset: f32 = {
      let mut temp = 0.0;
      
      if letter == '\'' as u8 ||
         letter == '\"' as u8 {
        temp = -size/16.333333333;
        //println!("The letter is '");
      }
      
      if letter == 'p' as u8 ||
         letter == 'y' as u8 ||
         letter == 'g' as u8 ||
         letter == 'j' as u8 ||
         letter == 'q' as u8 ||
         letter == '@' as u8 ||
         letter == '$' as u8 {
        temp = c.get_offset_y()*size;
      }
      
      if letter == '-' as u8 {
       temp = -c.get_offset_y()*0.5*size;
      }
      temp
    };
              
    let mut model = Matrix4::from_translation(Vector3::new(
                                  translation.x + x_offset, 
                                  translation.y - y_offset, 
                                  translation.z));
    model = model * Matrix4::from_nonuniform_scale(size, size, 1.0);
    
    model
  }
  
   pub fn calculate_text_uv(c: &GenericCharacter) -> Vector4<f32> {
     let x: f32 = c.get_x() as f32;
     let y: f32 = c.get_y() as f32;
     let x_w: f32 = x + c.get_width() as f32;
     let y_h: f32 = y + c.get_height() as f32;
     
     Vector4::new(x, y, x_w, y_h)
   }
   
   pub fn rotate(angle: f32) -> (f32, f32) {
     let mut crnt_pos = Vector2::new(0.0, 1.0);
     
     let mut rotation_mat = Matrix2::new(
       angle.cos(), -(angle.sin()),
       angle.sin(), angle.cos()
     );
     
     let new_pos = rotation_mat * crnt_pos;
     
     (new_pos.x, new_pos.y)
   }
   
   pub fn calculate_y_rotation(y_rotation: f32) -> (f32, f32) {
     let mut x_rot = 0.0;
     let mut z_rot = 0.0;
    
     let q1 = 90.0;
     let q2 = 180.0;
     let q3 = 270.0;
    
     let mut angle_y = y_rotation;
    
     if angle_y < 0.0 {
       angle_y = 360.0 + angle_y;
     }
     
     if angle_y > 360.0 {
       angle_y = angle_y - 360.0;
     }
     
     if angle_y < q1 {
       z_rot = 1.0 - (angle_y/90.0);
       x_rot = angle_y/90.0;
     } else if angle_y < q2 {
       angle_y -= q1;
       z_rot = -(angle_y/90.0);
       x_rot = 1.0-(angle_y/90.0);
     } else if angle_y < q3 {
       angle_y -= q2;
       z_rot = (angle_y/90.0) - 1.0;
       x_rot = -(angle_y/90.0);
     } else {
       angle_y -= q3;
       z_rot = angle_y/90.0;
       x_rot = angle_y/90.0 - 1.0;
     }
     
     (x_rot, z_rot)
   }
   
   pub fn intersection(center: Vector2<f32>, radius: f32, p1: Vector2<f32>, p2: Vector2<f32>) -> ((f32, f32), (f32, f32)) {
     let mut dx = p2.x - p1.x;
     let mut dy = p2.y - p1.y;
     let mut radius = radius;
     if dx < 1.0 && dx > -1.0 {
       if dx <= 0.0 {
         dx = -1.0;
       } else {
         dx = 1.0;
       }
     }
     
     if dy < 1.0 && dy > -1.0 {
       if dy <= 0.0 {
         dy = -1.0;
       } else {
         dy = 1.0;
       }
     }
     
     let a = dx*dx + dy*dy;
     let b = 2.0* (dx * (p1.x - center.x) + dy * (p1.y - center.y));
     let mut c = (p1.x - center.x)*(p1.x - center.x) + (p1.y - center.y)*(p1.y - center.y) - radius*radius;
     
     let mut discriminit = b*b - 4.0*a*c;
     if discriminit < 0.0 {
       radius *= 2.0;
       c = (p1.x - center.x)*(p1.x - center.x) + (p1.y - center.y)*(p1.y - center.y) - radius*radius;
       discriminit = b*b - 4.0*a*c;
     }
     
     let t1 = (-b + discriminit.sqrt()) / (2.0 * a);
     let t2 = (-b - discriminit.sqrt()) / (2.0 * a);
     
     ((dx * t1 + p1.x, dy * t1 + p1.y), (dx * t2 + p1.x, dy* t2 + p1.y))
   }
   
   /// Simple collision between two cicles given
   /// a Vector3(center_x, center_y, raidus)
   ///
   /// # Examples
   /// 
   /// Simple example with circles that do collide.
   ///
   /// ```
   /// # extern crate maat_engine;
   /// # extern crate cgmath;
   /// let a = cgmath::Vector3::new(1.0, 1.0, 5.0);
   /// let b = cgmath::Vector3::new(-1.0, -1.0, 4.0);
   /// assert!(maat_engine::drawcalls::DrawMath::circle_collision(a, b));
   /// ```
   ///
   /// Simple eample with circle that dont collide.
   /// 
   /// ```
   /// # extern crate maat_engine;
   /// # extern crate cgmath;
   /// let a = cgmath::Vector3::new(10.0, 10.0, 5.0);
   /// let b = cgmath::Vector3::new(-10.0, -10.0, 4.0);
   /// assert!(!maat_engine::drawcalls::DrawMath::circle_collision(a, b));
   /// ```
   /// 
   pub fn circle_collision(a: Vector3<f32>, b: Vector3<f32>) -> bool {
     let dist = a.z + b.z;
     let dx = b.x - a.x;
     let dy = b.y - a.y;
     
     if dx*dx + dy*dy < dist*dist {
       return true
     }
     
     false
   }
   
   /// Simple collision between two box's given
   /// a Vector4(center_x, center_y, width, height)
   ///
   /// # Examples
   /// 
   /// Simple example with box's that do collide.
   ///
   /// ```
   /// # extern crate maat_engine;
   /// # extern crate cgmath;
   /// let a = cgmath::Vector4::new(1.0, 1.0, 5.0, 5.0);
   /// let b = cgmath::Vector4::new(-1.0, -1.0, 4.0, 4.0);
   /// assert!(maat_engine::drawcalls::DrawMath::box_collision(a, b));
   /// ```
   ///
   /// Simple eample with circle that dont collide.
   /// 
   /// ```
   /// # extern crate maat_engine;
   /// # extern crate cgmath;
   /// let a = cgmath::Vector4::new(10.0, 10.0, 5.0, 5.0);
   /// let b = cgmath::Vector4::new(-10.0, -10.0, 4.0, 4.0);
   /// assert!(!maat_engine::drawcalls::DrawMath::box_collision(a, b));
   /// ```
   /// 
   pub fn box_collision(a: Vector4<f32>, b: Vector4<f32>) -> bool {
     if a.x+a.z*0.5 < b.x-b.z*0.5 || a.x-a.z*0.5 > b.x+b.z*0.5 {
       return false
     }
     if a.y+a.w*0.5 < b.y-b.w*0.5 || a.y-a.w*0.5 > b.y+b.w*0.5 {
       return false   
     }
     true
   }
   
   pub fn min(a: f32, b: f32) -> f32 {
     if a > b {
       return b;
     }
     a
   }
   
  pub fn max(a: f32, b: f32) -> f32 {
     if a < b {
       return b;
     }
     a
   }
}
