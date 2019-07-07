use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use std::env;
use cgmath::Vector2;

const SETTINGS_LOCATION: &str = "./settings.ini";
const NL: &str = "\n";
const SPACE: &str = " ";
const TRUE: &str = "True";
const FALSE: &str = "False";
const FULLSCREEN: &str = "Fullscreen";
const TEXTURE_MSAA: &str = "TextureMsaa";
const MODEL_MSAA: &str = "ModelMsaa";
const VSYNC: &str = "Vsync";
const FORCE_DPI: &str = "ForceDpi";
const DPI: &str = "Dpi";
const TRIPLE_BUFFERING: &str = "TripleBuffer";
const RESOLUTION: &str = "Resolution";

#[derive(Clone)]
pub struct Settings {
  vsync: bool,
  triple_buffer: bool,
  texture_msaa: u32,
  model_msaa: u32,
  fullscreen: bool,
  _minimum_resolution: [u32; 2],
  resolution: [u32; 2],
  force_dpi: bool,
  dpi: f32,
}

impl Settings {
  pub fn load(minimum_resolution: Vector2<i32>, default_resolution: Vector2<i32>) -> Settings {
    let mut vsync = true;
    let mut triple_buffer = false;
    let mut texture_msaa = 1;
    let mut model_msaa = 1;
    let mut is_fullscreen = false;
    let mut resolution = default_resolution;//[1280, 720];
    let mut force_dpi = false;
    let mut dpi = 1.0;
    
    if let Ok(f) = File::open("./settings.ini") {
      println!("Settings file exists");
      let f = BufReader::new(f);
      
      for line in f.lines() {
          let line = line.expect("Unable to read line");
          let v: Vec<&str> = line.split(" ").collect();
          match v[0] {
            RESOLUTION => {
              let mut temp_res = Vector2::new(0,0);
              if let Ok(x) = v[1].parse::<i32>() {
                temp_res.x = x;
              }
              if let Ok(y) = v[2].parse::<i32>() {
                temp_res.y = y;
              }
              
              if temp_res != Vector2::new(0, 0) && temp_res.x > 0 && temp_res.y > 0 {
                resolution = temp_res;
              }
            }
            FULLSCREEN => {
              match v[1] {
                TRUE => {
                  is_fullscreen = true;
                },
                FALSE => {
                  is_fullscreen = false;
                },
                _ => {}
              }
            },
            TEXTURE_MSAA => {
              if let Ok(s) = v[1].parse::<u32>() {
                texture_msaa = s;
              }
            },
            MODEL_MSAA => {
              if let Ok(s) = v[1].parse::<u32>() {
                model_msaa = s;
              }
            },
            VSYNC => {
              match v[1] {
                TRUE => {
                  vsync = true;
                },
                FALSE => {
                  vsync = false;
                },
                _ => {}
              }
            },
            TRIPLE_BUFFERING => {
              match v[1] {
                TRUE => {
                  triple_buffer = true;
                },
                FALSE => {
                  triple_buffer = false;
                },
                _ => {}
              }
            },
            FORCE_DPI => {
              match v[1] {
                TRUE => {
                  force_dpi = true;
                },
                FALSE => {
                  force_dpi = false;
                },
                _ => {}
              }
            },
            DPI => {
              if let Ok(custom_dpi) = v[1].parse::<f32>() {
                
                dpi = custom_dpi;
              }
            }
            _ => {
              println!("Unknown setting: {:?}", v);
            }
          }
      }
    } else {
      Settings::save_defaults(default_resolution);
    }
    
    if force_dpi {
      Settings::force_dpi(dpi);
    }
    
    Settings {
      vsync: vsync,
      triple_buffer: triple_buffer,
      texture_msaa,
      model_msaa,
      fullscreen: is_fullscreen,
      resolution: [resolution.x.max(minimum_resolution.x) as u32, resolution.y.max(minimum_resolution.y) as u32],
      _minimum_resolution: [minimum_resolution.x as u32, minimum_resolution.y as u32],
      force_dpi: force_dpi,
      dpi: dpi,
    }
  }
  
  pub fn save(&self) {
    let fullscreen = {
      if self.fullscreen {
        TRUE
      } else {
        FALSE
      }
    };
    
    let vsync = {
      if self.vsync {
        TRUE
      } else {
        FALSE
      }
    };
     let triple_buffer = {
      if self.triple_buffer {
        TRUE
      } else {
        FALSE
      }
    };
    
    let force_dpi = {
      if self.force_dpi {
        TRUE
      } else {
        FALSE
      }
    };
    
    let data = RESOLUTION.to_owned() + SPACE + &self.resolution[0].to_string() + 
                  SPACE + &self.resolution[1].to_string() + NL +
                  FULLSCREEN + SPACE + fullscreen + NL + 
                  VSYNC             + SPACE + vsync + NL + 
                  TRIPLE_BUFFERING  + SPACE + triple_buffer + NL + 
                  TEXTURE_MSAA      + SPACE + &self.texture_msaa.to_string() + NL + 
                  MODEL_MSAA        + SPACE + &self.model_msaa.to_string() + NL + 
                  FORCE_DPI         + SPACE + force_dpi + NL + 
                  DPI               + SPACE + &self.dpi.to_string() + NL;
    let f = File::create(SETTINGS_LOCATION).expect("Error: Failed to create settings file");
    let mut f = BufWriter::new(f);
    f.write_all(data.as_bytes()).expect("Unable to write data");
  }
  
  pub fn save_defaults(default_resolution: Vector2<i32>) {
    println!("Settings file not found");
    let data = RESOLUTION.to_owned() + SPACE + &default_resolution.x.to_string() + 
                  SPACE + &default_resolution.y.to_string() + NL +
                  FULLSCREEN + SPACE + FALSE + NL + 
                  VSYNC             + SPACE + TRUE  + NL + 
                  TRIPLE_BUFFERING  + SPACE + FALSE + NL + 
                  TEXTURE_MSAA      + SPACE + "2"   + NL + 
                  MODEL_MSAA        + SPACE + "2"   + NL + 
                  FORCE_DPI         + SPACE + FALSE + NL + 
                  DPI               + SPACE + "1"   + NL;
    let f = File::create(SETTINGS_LOCATION).expect("Error: Failed to create settings file");
    let mut f = BufWriter::new(f);
    f.write_all(data.as_bytes()).expect("Unable to write data");
  }
  
  pub fn force_dpi(dpi_value: f32) {
    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    env::set_var("WINIT_HIDPI_FACTOR", dpi_value.to_string());
    println!("Forcing dpi scale of {}", dpi_value);
  }
  
  pub fn vsync_enabled(&self) -> bool {
    self.vsync
  }
  
  pub fn triple_buffer_enabled(&self) -> bool {
    self.triple_buffer
  }
  
  pub fn set_texture_msaa(&mut self, msaa: u32) {
    self.texture_msaa = msaa;
  }
  
  pub fn set_model_msaa(&mut self, msaa: u32) {
    self.model_msaa = msaa;
  }
  
  pub fn get_texture_msaa(&self) -> u32 {
    self.texture_msaa
  }
  
  pub fn get_model_msaa(&self) -> u32 {
    self.model_msaa
  }
  
  pub fn is_fullscreen(&self) -> bool {
    self.fullscreen
  }
  
  pub fn get_minimum_resolution(&self) -> [u32; 2] {
    self._minimum_resolution
  }
  
  pub fn set_resolution(&mut self, res: Vector2<i32>) {
    self.resolution = [res.x as u32, res.y as u32];
  }
  
  pub fn _set_dpi(&mut self, new_dpi: f32) {
    self.dpi = new_dpi;
  }
  
  pub fn _enable_dpi(&mut self, enable: bool) {
    self.force_dpi = enable;
  }
  
  pub fn set_vsync(&mut self, enable: bool) {
    self.vsync = enable;
  }
  
  pub fn enable_fullscreen(&mut self, enable: bool) {
    self.fullscreen = enable;
  }
  
  pub fn get_resolution(&self) -> [u32; 2] {
    self.resolution
  }
}

impl Drop for Settings {
  fn drop(&mut self) {
    self.save();
  }
}
