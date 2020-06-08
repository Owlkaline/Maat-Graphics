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
const MAX_RESOLUTION: &str = "MaxMonitorResolution";
const BORDERLESS: &str = "Borderless";
const MONITOR: &str = "Monitor";

pub const RESOLUTIONS: [(u32, u32, &str); 99] = [
  (2160, 1080, "(1:2)"),
  (240, 160, "(3:2)"),
  (360, 240, "(3:2)"),
  (480, 320, "(3:2)"),
  (720, 480, "(3:2)"),
  (960, 640, "(3:2)"),
  (1152, 768, "(3:2)"),
  (1280, 864, "(3:2)"),
  (1440, 960, "(3:2)"),
  (1600, 1024, "(3:2)"),
  (1920, 1280, "(3:2)"),
  (2160, 1440, "(3:2)"),
  (2400, 1600, "(3:2)"),
  (2880, 1920, "(3:2)"),
  (3240, 2160, "(3:2)"),
  (3840, 2560, "(3:2)"),
  (160, 120, "(4:3)"),
  (192, 144, "(4:3)"),
  (320, 240, "(4:3)"),
  (480, 360, "(4:3)"),
  (640, 480, "(4:3)"),
  (768, 576, "(4:3)"),
  (800, 600, "(4:3)"),
  (960, 720, "(4:3)"),
  (1024, 768, "(4:3)"),
  (1152, 864, "(4:3)"),
  (1200, 900, "(4:3)"),
  (1280, 960, "(4:3)"),
  (1400, 1050, "(4:3)"),
  (1440, 1080, "(4:3)"),
  (1600, 1200, "(4:3)"),
  (1920, 1440, "(4:3)"),
  (2048, 1536, "(4:3)"),
  (2560, 1920, "(4:3)"),
  (2880, 2160, "(4:3)"),
  (3200, 2400, "(4:3)"),
  (4096, 3072, "(4:3)"),
  (400, 240, "(5:3)"),
  (800, 480, "(5:3)"),
  (1280, 768, "(5:3)"),
  (400, 240, "(5:3)"),
  (750, 600, "(5:4)"),
  (960, 768, "(5:4)"),
  (1280, 1024, "(5:4)"),
  (1500, 1200, "(5:4)"),
  (2560, 2048, "(5:4)"),
  (3840, 1600, "(12:5)"),
  (256, 144, "(16:9)"),
  (432, 240, "(16:9)"),
  (640, 360, "(16:9)"),
  (854, 480, "(16:9)"),
  (960, 540, "(16:9)"),
  (1024, 576, "(16:9)"),
  (1136, 640, "(16:9)"),
  (1280, 720, "(16:9)"),
  (1366, 768, "(16:9)"),
  (1536, 864, "(16:9)"),
  (1600, 900, "(16:9)"),
  (1920, 1080, "(16:9)"),
  (2048, 1152, "(16:9)"),
  (2560, 1440, "(16:9)"),
  (2880, 1620, "(16:9)"),
  (3200, 1800, "(16:9)"),
  (3840, 2160, "(16:9)"),
  (5120, 2880, "(16:9)"),
  (7680, 4320, "(16:9)"),
  (320, 200, "(16:10)"),
  (384, 240, "(16:10)"),
  (768, 480, "(16:10)"),
  (1024, 640, "(16:10)"),
  (1152, 720, "(16:10)"),
  (1280, 800, "(16:10)"),
  (1440, 900, "(16:10)"),
  (1680, 1050, "(16:10)"),
  (1920, 1200, "(16:10)"),
  (2304, 1440, "(16:10)"),
  (2560, 1600, "(16:10)"),
  (2880, 1800, "(16:10)"),
  (3072, 1920, "(16:10)"),
  (3840, 2400, "(16:10)"),
  (4096, 2560, "(16:10)"),
  (960, 480, "(18:9)"),
  (1440, 720, "(18:9)"),
  (2880, 1440, "(18:9)"),
  (4320, 2160, "(18:9)"),
  (5760, 2880, "(18:9)"),
  (2960, 1440, "(18.5:9)"),
  (3120, 1440, "(19.5:9)"),
  (2560, 1080, "(21:9)"),
  (5120, 2160, "(21:9)"),
  (3200, 2048, "(25:16)"),
  (3840, 1080, "(32:9)"),
  (5120, 1440, "(32:9)"),
  (3840, 1200, "(32:10)"),
  (5760, 1600, "(36:10)"),
  (3440, 1440, "(43:18)"),
  (1024, 600, "(128:75)"),
  (2048, 1080, "(256:135)"),
  (4096, 2160, "(256:135)"),
];


#[derive(Clone)]
pub struct Settings {
  vsync: bool,
  triple_buffer: bool,
  texture_msaa: u32,
  model_msaa: u32,
  fullscreen: bool,
  borderless: bool,
  _minimum_resolution: [u32; 2],
  max_monitor_resolution: [u32; 2],
  resolution: [u32; 2],
  force_dpi: bool,
  dpi: f32,
  monitor: usize,
}

impl Settings {
  pub fn load() -> Settings {
    let mut vsync = true;
    let mut triple_buffer = false;
    let mut texture_msaa = 1;
    let mut model_msaa = 1;
    let mut is_fullscreen = false;
    let mut is_borderless = true;
    let mut monitor = 0;
    let mut resolution: [u32; 2] = [1280, 720];
    let mut max_monitor_resolution = [1920, 1080];
    let mut force_dpi = false;
    let mut dpi = 1.0;
    
    if let Ok(f) = File::open("./settings.ini") {
      println!("Settings file exists");
      let f = BufReader::new(f);
      
      for line in f.lines() {
          let line = line.expect("Unable to read line");
          let v: Vec<&str> = line.split(" ").collect();
          match v[0] {
            MAX_RESOLUTION => {
              let mut temp_res = [0,0];
              if let Ok(x) = v[1].parse::<u32>() {
                temp_res[0] = x;
              }
              if let Ok(y) = v[2].parse::<u32>() {
                temp_res[1] = y;
              }
              
              if temp_res[0] > 0 && temp_res[1] > 0 {
                max_monitor_resolution = temp_res;
              }
            },
            RESOLUTION => {
              let mut temp_res = [0,0];
              if let Ok(x) = v[1].parse::<u32>() {
                temp_res[0] = x;
              }
              if let Ok(y) = v[2].parse::<u32>() {
                temp_res[1] = y;
              }
              
              if temp_res[0] > 0 && temp_res[1] > 0 {
                resolution = temp_res;
              }
            },
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
            BORDERLESS => {
              match v[1] {
                TRUE => {
                  is_borderless = true;
                },
                FALSE => {
                  is_borderless = false;
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
            },
            MONITOR => {
              if let Ok(m) = v[1].parse::<usize>() {
                monitor = m;
              }
            },
            _ => {
              println!("Unknown setting: {:?}", v);
            }
          }
      }
    } else {
      Settings::save_defaults();
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
      borderless: is_borderless,
      resolution,
      _minimum_resolution: [800, 640],
      max_monitor_resolution,
      force_dpi: force_dpi,
      dpi: dpi,
      monitor,
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
    
    let borderless = {
      if self.borderless {
        TRUE
      } else {
        FALSE
      }
    };
    
    let data = RESOLUTION.to_owned() + SPACE + &self.resolution[0].to_string() + 
                  SPACE + &self.resolution[1].to_string() + NL +
                  MAX_RESOLUTION    + SPACE + &self.max_monitor_resolution[0].to_string() + SPACE + &self.max_monitor_resolution[1].to_string() + NL +
                  FULLSCREEN        + SPACE + fullscreen + NL + 
                  BORDERLESS        + SPACE + borderless + NL + 
                  MONITOR           + SPACE + &self.monitor.to_string() + NL + 
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
  
  pub fn save_defaults() {
    println!("Settings file not found");
    let data = RESOLUTION.to_owned() + SPACE + &(1280).to_string() + 
                  SPACE + &(720).to_string() + NL +
                  MAX_RESOLUTION    + SPACE + &(1920).to_string() + SPACE + &(1080).to_string() + NL +
                  FULLSCREEN        + SPACE + FALSE + NL + 
                  BORDERLESS        + SPACE + TRUE + NL + 
                  MONITOR           + SPACE + "0" + NL +
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
  
  pub fn resolutions() -> [(u32, u32, &'static str); 99] {
    RESOLUTIONS
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
  
  pub fn max_monitor_resolution(&self) -> [u32; 2] {
    self.max_monitor_resolution
  }
  
  pub fn set_max_monitor_resolution(&mut self, max_res: Vector2<i32>) {
    self.max_monitor_resolution = [max_res.x as u32, max_res.y as u32];
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
  
  pub fn get_monitor_idx(&self) -> usize {
    self.monitor
  }
  
  pub fn is_borderless(&self) -> bool {
    self.borderless
  }
}

impl Drop for Settings {
  fn drop(&mut self) {
    self.save();
  }
}
