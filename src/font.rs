use std::string::String;
use std::str::FromStr;

#[derive(Clone)]
pub struct GenericCharacter {
  char_id: i32,
  x: f32,
  y: f32,
  width: f32,
  height: f32,
  offset_x: f32,
  offset_y: f32,
  advance: f32,
}

#[derive(Clone)]
pub struct GenericFont {
  characters: Vec<GenericCharacter>,
}

impl GenericCharacter {
  pub fn new() -> GenericCharacter {
    GenericCharacter {
      char_id: 0,
      x: 0.0,
      y: 0.0,
      width: 0.0,
      height: 0.0,
      offset_x: 0.0,
      offset_y: 0.0,
      advance: 0.0,
    }
  }
  
  pub fn get_char(&self) -> i32 {
    self.char_id
  }
  
  pub fn get_x(&self) -> f32 {
    self.x
  }
  
  pub fn get_y(&self) -> f32 {
    self.y
  }
  
  pub fn get_width(&self) -> f32 {
    self.width
  }
  
  pub fn get_height(&self) -> f32 {
    self.height
  }
  
  pub fn get_offset_x(&self) -> f32 {
    self.offset_x
  }
  
  pub fn get_offset_y(&self) -> f32 {
    self.offset_y
  }
  
  pub fn get_advance(&self) -> f32 {
    self.advance
  }
  
  pub fn set_char(&mut self, char_id: i32) {
    self.char_id = char_id;
  }
  
  pub fn set_x(&mut self, x: f32) {
    self.x = x;
  }
  
  pub fn set_y(&mut self, y: f32) {
    self.y = y;
  }
  
  pub fn set_width(&mut self, width: f32) {
    self.width = width;
  }
  
  pub fn set_height(&mut self, height: f32) {
    self.height = height;
  }
  
  pub fn set_offset_x(&mut self, offset_x: f32) {
    self.offset_x = offset_x;
  }
  
  pub fn set_offset_y(&mut self, offset_y: f32) {
    self.offset_y = offset_y;
  }
  
  pub fn set_advance(&mut self, advance: f32) {
    self.advance = advance;
  }
}

impl GenericFont {
  pub fn new() -> GenericFont {
    GenericFont {
      characters: Vec::with_capacity(42),
    }
  }
  
  pub fn get_character(&self, char_id: i32) -> &GenericCharacter {
    for c in &self.characters {
      if c.char_id == char_id {
        return c
      }
    }
    return &self.characters[0]
  }
  
  pub fn load_font(&mut self, font_location: &[u8]) {
    self.load_characters(font_location);
  }
  
  fn load_characters(&mut self, data: &[u8]) {
    let font_data = String::from_utf8_lossy(data);
    
    let char_data = font_data.split("\n");
    
    let mut i = 0;
    for line in char_data.into_iter() {
      // Number of characters      
      if i >= 4 {
        let mut j = 0;
        let mut character = GenericCharacter::new();
        for semi_value in line.split("=").into_iter() {
          if j > 0 && j < 9 {
            let mut got_value = false;
            for value in semi_value.split(" ").into_iter() {
              if !got_value {
                let value: f32 = FromStr::from_str(value).unwrap();
                match j {
                  1 => character.set_char(value as i32),
                  2 => character.set_x(value/512.0),
                  3 => character.set_y(value/512.0),
                  4 => character.set_width(value/512.0),
                  5 => character.set_height(value/512.0),
                  6 => character.set_offset_x(value/512.0),
                  7 => character.set_offset_y(value/512.0),
                  8 => character.set_advance(value),
                  _ => (),
                }
                got_value = true;
              }
            }
          }
          j+=1;
        }
        self.characters.push(character);
      }
      i+=1;
    }
  }
}
