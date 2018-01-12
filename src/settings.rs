//use std::fs::File;
//use std::io::prelude::*;
//use std::path::Path;
//use std::str::FromStr;

pub struct Settings {
  vulkan: bool,
  fullscreen: bool,
  resolution: [u32; 2],
}

impl Settings {
  pub fn load() -> Settings {
  /*  let settings_path = "settings.ini";
    
    if !Path::new(settings_path).exists() {
      // Create new file
      println!("Creating new settings");
      let mut file = File::create(settings_path);
      file.unwrap().write_all(b"1\n0\n");
    }
    
    let mut file = File::open(settings_path);
    let mut contents = String::new();
    file.unwrap().read_to_string(&mut contents);
    
    let use_vulkan = settings_data.split("\n");
    
    for var in use_vulkan.into_iter() {
      let test = FromStr::from_str(var).unwrap();
      println!("fullscreen {}", FromStr::from_str(test).unwrap());
      break;
    }
    /*let mut i = 0;
    for line in char_data.into_iter() {
    
    }*/
    
    println!("contents {}", contents);*/
    
    Settings {
      vulkan: true,
      fullscreen: false,
      resolution: [1280, 800],
    }
  }
  
  pub fn vulkan_enabled(&self) -> bool {
    self.vulkan
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
