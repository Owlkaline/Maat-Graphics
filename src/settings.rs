use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
//use std::io::prelude::*;
//use std::path::Path;
//use std::str::FromStr;

//use std::fs;

const SETTINGS_LOCATION: &str = "./settings.ini";
const NL: &str = "\n";
const SPACE: &str = " ";
const TRUE: &str = "True";
const FALSE: &str = "False";
const VULKAN: &str = "Vulkan";
const FULLSCREEN: &str = "Fullscreen";
const MSAA: &str = "MSAA";
const VSYNC: &str = "VSYNC";

pub struct Settings {
  vsync: bool,
  vulkan: bool,
  samples: u32,
  fullscreen: bool,
  resolution: [u32; 2],
}

impl Settings {
  pub fn load() -> Settings {
    let mut vsync = true;
    let mut samples = 1;
    let mut use_vulkan = false;
    let mut is_fullscreen = false;
    let mut resolution = [1280, 800];
    
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
                _ => {
                  
                }
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
                _ => {
                  
                }
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
                _ => {
                  
                }
              }
            },
            _ => {
              println!("Unknown setting: {:?}", v);
            }
          }
      }
    } else {
      println!("Settings file not found");
       let data = VULKAN.to_owned() + SPACE + FALSE + NL + 
                  FULLSCREEN        + SPACE + FALSE + NL +
                  MSAA              + SPACE + "0"   + NL;
       let f = File::create(SETTINGS_LOCATION).expect("Error: Failed to create settings file");
       let mut f = BufWriter::new(f);
       f.write_all(data.as_bytes()).expect("Unable to write data");
    }
    
    
    //GLint max_samples;
    //glGetIntegerv(GL_MAX_SAMPLES, &max_samples);
    
    Settings {
      vsync: vsync,
      vulkan: use_vulkan,
      samples: samples,
      fullscreen: is_fullscreen,
      resolution: resolution,
    }
  }
  
  pub fn vulkan_enabled(&self) -> bool {
    self.vulkan
  }
  
  pub fn vsync_enabled(&self) -> bool {
    self.vsync
  }
  
  pub fn get_msaa(&self) -> u32 {
    self.samples
  }
  
  pub fn is_fullscreen(&self) -> bool {
    self.fullscreen
  }
  
  pub fn get_minimum_resolution(&self) -> [u32; 2] {
    [1280, 800]
  }
  
  pub fn get_resolution(&mut self) -> [u32; 2] {
    if self.resolution < self.get_minimum_resolution() {
      self.resolution = self.get_minimum_resolution();
    }    
    self.resolution
  }
}

/*
*/
