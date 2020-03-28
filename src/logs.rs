use std::fs::File;
use std::io::{BufWriter, Write};

use crate::cgmath::Vector2;

macro_rules! log_unwrap_or_panic {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => { log.panic_msg("test"); panic!("test"); },
        }
    }
}

pub struct Logs {
  last_error: String,
  error_log: BufWriter<File>,
}

impl Logs {
  pub fn new() -> Logs {
    let f = File::create("./maat_graphics.log").expect("Error: Failed to create settings file");
    let f = BufWriter::new(f);
    
    Logs {
      last_error: "No Errors".to_string(),
      error_log: f,
    }
  }
  
  pub fn system_msg(&mut self, err: &str) {
    self.last_error = err.to_string();
    if let Err(_) = self.error_log.write(&("System: ".to_owned() + &err + "\n").as_bytes()) {
      println!("Writting logs failed");
    }
  }
  
  pub fn warning_msg(&mut self, err: &str) {
    self.last_error = err.to_string();
    if let Err(_) = self.error_log.write(&("Warning: ".to_owned() + &err + "\n").as_bytes()) {
      println!("Writting logs failed");
    }
  }
  
  pub fn error_msg(&mut self, err: &str) {
    self.last_error = err.to_string();
    if let Err(_) = self.error_log.write(&("Error: ".to_owned() + &err + "\n").as_bytes()) {
      println!("Writting logs failed");
    }
  }
  
  pub fn panic_msg(&mut self, err: &str) {
    self.last_error = err.to_string();
    if let Err(_) = self.error_log.write(&("Panic: ".to_owned() + &err + "\n").as_bytes()) {
      println!("Writting logs failed");
    }
  }
}

