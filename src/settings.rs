use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
//use std::io::prelude::*;
//use std::path::Path;
//use std::str::FromStr;

//use std::fs;

use std::env;

const SETTINGS_LOCATION: &str = "./settings.ini";
const NL: &str = "\n";
const SPACE: &str = " ";
const TRUE: &str = "True";
const FALSE: &str = "False";
const VULKAN: &str = "Vulkan";
const FULLSCREEN: &str = "Fullscreen";
const MSAA: &str = "MSAA";
const VSYNC: &str = "Vsync";
const FORCE_DPI: &str = "ForceDpi";
const DPI: &str = "Dpi";
const TRIPLE_BUFFERING: &str = "TripleBuffer";

pub struct Settings {
  vsync: bool,
  triple_buffer: bool,
  vulkan: bool,
  samples: u32,
  fullscreen: bool,
  resolution: [u32; 2],
  force_dpi: bool,
  dpi: f32,
}

impl Settings {
  pub fn load() -> Settings {
    let mut vsync = true;
    let mut triple_buffer = false;
    let mut samples = 4;
    let mut use_vulkan = true;
    let mut is_fullscreen = false;
    let mut resolution = [1280, 720];
    let mut force_dpi = false;
    let mut dpi = 1.0;
    
    if let Ok(f) = File::open("./settings.ini") {
      println!("Settings file exists");
      let f = BufReader::new(f);
      
      for line in f.lines() {
          let line = line.expect("Unable to read line");
          let v: Vec<&str> = line.split(" ").collect();
          match v[0] {
            VULKAN => {
              match v[1] {
                TRUE => {
                  use_vulkan = true;
                },
                FALSE => {
                  use_vulkan = false;
                },
                _ => {}
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
            MSAA => {
              if let Ok(s) = v[1].parse::<u32>() {
                samples = s;
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
      println!("Settings file not found");
       let data = VULKAN.to_owned() + SPACE + TRUE  + NL + 
                  FULLSCREEN        + SPACE + FALSE + NL + 
                  VSYNC             + SPACE + TRUE  + NL + 
                  TRIPLE_BUFFERING  + SPACE + FALSE + NL + 
                  MSAA              + SPACE + "4"   + NL + 
                  FORCE_DPI         + SPACE + FALSE + NL + 
                  DPI               + SPACE + "1"   + NL;
       let f = File::create(SETTINGS_LOCATION).expect("Error: Failed to create settings file");
       let mut f = BufWriter::new(f);
       f.write_all(data.as_bytes()).expect("Unable to write data");
    }
    
    if force_dpi {
      Settings::force_dpi(dpi);
    }
    
    Settings {
      vsync: vsync,
      triple_buffer: triple_buffer,
      vulkan: use_vulkan,
      samples: samples,
      fullscreen: is_fullscreen,
      resolution: resolution,
      force_dpi: force_dpi,
      dpi: dpi,
    }
  }
  
  pub fn force_dpi(dpi_value: f32) {
    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    env::set_var("WINIT_SCALE_FACTOR", dpi_value.to_string());
    println!("Forcing dpi scale of {}", dpi_value);
  }
  
  pub fn vulkan_enabled(&self) -> bool {
    self.vulkan
  }
  
  pub fn vsync_enabled(&self) -> bool {
    self.vsync
  }
  
  pub fn triple_buffer_enabled(&self) -> bool {
    self.triple_buffer
  }
  
  pub fn get_msaa(&self) -> u32 {
    self.samples
  }
  
  pub fn is_fullscreen(&self) -> bool {
    self.fullscreen
  }
  
  pub fn get_minimum_resolution(&self) -> [u32; 2] {
    [1280, 720]
  }
  
  pub fn get_resolution(&mut self) -> [u32; 2] {
    if self.resolution < self.get_minimum_resolution() {
      self.resolution = self.get_minimum_resolution();
    }    
    self.resolution
  }
}
