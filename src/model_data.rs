
use crate::cgmath::{Zero, Vector3, Vector4};

use csv;

use std::fs::File;
use std::fs;

#[derive(Clone, PartialEq, Debug)]
pub enum CollisionType {
  Cube,
  Sphere,
  Cylinder,
  Point,
  Terrain,
}

impl CollisionType {
  pub fn is_cube(&self) -> bool {
    *self == CollisionType::Cube
  }
  
  pub fn is_terrain(&self) -> bool {
    *self == CollisionType::Terrain 
  }
}

#[derive(Clone, Debug)]
pub struct CollisionInfo {
  pos: Vector3<f32>,
  size: Vector3<f32>,
  quaternion_rotation: Vector4<f32>,
  
  collision_type: Option<CollisionType>,
  
  terrain_data: Vec<Vec<f32>>,
}

#[derive(Clone, Debug)]
pub struct ModelData {
  model: String,
  location: String,
  size: Vector3<f32>,
  collision_info: Vec<CollisionInfo>,
}

impl CollisionInfo {
  pub fn new() -> CollisionInfo {
    CollisionInfo {
      pos: Vector3::zero(),
      size: Vector3::zero(),
      quaternion_rotation: Vector4::new(1.0, 0.0, 0.0, 0.0),
      
      collision_type: None,
      
      terrain_data: Vec::new(),
    }
  }
  
  pub fn offset_position(&self) -> Vector3<f32> {
    self.pos
  }
  
  pub fn size(&self) -> Vector3<f32> {
    self.size
  }
  
  pub fn quaternion(&self) -> Vector4<f32> {
    self.quaternion_rotation
  }
  
  pub fn collision_type(&self) -> Option<CollisionType> {
    self.collision_type.clone()
  }
  
  pub fn terrain_data(&self) -> Vec<Vec<f32>> {
    self.terrain_data.clone()
  }
  
  pub fn add_terrain_data(&mut self, data: Vec<Vec<f32>>) {
    self.collision_type = Some(CollisionType::Terrain);
    self.terrain_data = data;
  }
  
  pub fn add_cube_collision(&mut self, pos: Vector3<f32>, size: Vector3<f32>, rot: Vector4<f32>) {
    self.collision_type = Some(CollisionType::Cube);
    
    self.pos = pos;
    self.size = size;
    self.quaternion_rotation = rot;
  }
}

impl ModelData {
  pub fn new(name: String, location: String) -> ModelData {
    let mut location = location;
    location.pop();
    location.pop();
    location.pop();
    ModelData {
      model: name.to_string(),
      location,
      size: Vector3::zero(),
      collision_info: Vec::new(),
    }
  }
  
  pub fn new_terrain(name: String, size: Vector3<f32>, heights: Vec<Vec<f32>>) -> ModelData {
    let mut collision_info = Vec::new();
    let mut info = CollisionInfo::new();
    info.add_terrain_data(heights);
    collision_info.push(info);
    
    ModelData {
      model: name,
      location: "".to_string(),
      size,
      collision_info,
    }
  }
  
  pub fn name(&self) -> String {
    self.model.to_string()
  }
  
  pub fn size(&self) -> Vector3<f32> {
    self.size
  }
  
  pub fn num_collision_info(&self) -> usize {
    self.collision_info.len()
  }
  
  pub fn collision_info(&self) -> &Vec<CollisionInfo> {
    &self.collision_info
  }
  
  pub fn set_size(&mut self, size: Vector3<f32>) {
    self.size = size;
  }
  
  pub fn get_terrain_data(&self) -> (String, Vec<Vec<f32>>) {
    let mut terrain_data = Vec::new();
    
    for i in 0..self.collision_info.len() {
      let mut is_terrain = false;
      if let Some(kind) = self.collision_info[i].collision_type() {
        if kind.is_terrain() {
          is_terrain = true;
        }
      }
      if is_terrain {
        terrain_data = self.collision_info[i].terrain_data();
        break;
      }
    }
    
    (self.model.to_string(), terrain_data)
  }
  
  pub fn is_terrain_data(&self) -> bool {
    let mut is_terrain = false;
    for i in 0..self.collision_info.len() {
      if let Some(kind) = self.collision_info[i].collision_type() {
        if kind.is_terrain() {
          is_terrain = true;
        }
      }
    }
    
    is_terrain
  }
  
  pub fn add_collision_info(&mut self, info: CollisionInfo) {
    self.collision_info.push(info);
  }
  
  pub fn load_collision_info(&mut self) {
    match File::open(self.location.to_owned() + "csv") {
      Ok(file) => {
        let mut rdr = csv::Reader::from_reader(file);
        
        for object in rdr.records() {
          let mut collision_info = CollisionInfo::new();
          let mut collision_info_found = true;
          
          match object {
            Ok(row) => {
              let mut data: Vec<&str> = row.as_slice().split('\t').collect();
              
              let name: String = data[0].to_string();
              let mut position: Vector3<f32> = Vector3::zero();
              let mut rotation: Vector4<f32> = Vector4::new(1.0, 0.0, 0.0, 0.0);
              let mut size: Vector3<f32> = Vector3::zero();
              
              position.x = data[1].parse().unwrap();
              position.y = data[2].parse().unwrap();
              position.z = data[3].parse().unwrap();
              
              size.x = data[4].parse().unwrap();
              size.y = data[5].parse().unwrap();
              size.z = data[6].parse().unwrap();
              
              rotation.x = data[7].parse().unwrap();
              rotation.y = data[8].parse().unwrap();
              rotation.z = data[9].parse().unwrap();
              rotation.w = data[10].parse().unwrap();
              
              let collision_type = data[11].to_string();
              
              collision_info.add_cube_collision(position, size, rotation);
            },
            Err(e) => {
              collision_info_found = false;
            },
          }
          
          if collision_info_found {
            self.collision_info.push(collision_info);
          }
        }
      },
      Err(e) => {
        println!("No collision data for: {} : num info: {}", self.model.to_string(), self.collision_info.len());
      }
    }
  }
}






