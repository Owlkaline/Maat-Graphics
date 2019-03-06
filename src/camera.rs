use cgmath;
use cgmath::Vector3;
use cgmath::Point3;
use cgmath::Angle;
use cgmath::EuclideanSpace;
use cgmath::prelude::InnerSpace;
use cgmath::Matrix4;

#[derive(Clone, PartialEq)]
pub enum Direction {
  Forward,
  Backward,
  Left,
  Right,
  Up,
  Down,
  PositiveX,
  NegativeX,
  PositiveY,
  NegativeY,
  PositiveZ,
  NegativeZ,
  YAlignedForward,
  YAlignedBackward,
  YAlignedLeft,
  YAlignedRight
}

#[derive(Clone, PartialEq)]
pub struct Camera {
  position: Vector3<f32>,
  front: Vector3<f32>,
  up: Vector3<f32>,
  right: Vector3<f32>,
  
  world_up: Vector3<f32>,
  
  yaw: f32,
  pitch: f32,
  move_speed: f32,
  mouse_sensitivity: f32,
  zoom: f32,
}

impl Camera {
  pub fn new(position: Vector3<f32>, front: Vector3<f32>, up: Vector3<f32>,
             move_speed: f32, mouse_sensitivity: f32) -> Camera {
    Camera {
      position: position,
      front: front,
      up: up,
      right: Vector3::new(1.0, 0.0, 0.0),
  
      world_up: Vector3::new(0.0, -1.0, 0.0),
  
      yaw: 0.0,
      pitch: 0.0,
      move_speed: move_speed,
      mouse_sensitivity: mouse_sensitivity,
      zoom: 90.0,
    }
  }
  
  pub fn default_vk() -> Camera {
    Camera {
      position: Vector3::new(0.0, 0.0, 0.0),
      front: Vector3::new(0.0, 0.0, 1.0),
      up: Vector3::new(0.0, -1.0, 0.0),
      right: Vector3::new(1.0, 0.0, 0.0),
     
      world_up: Vector3::new(0.0, -1.0, 0.0),
      
      yaw: 0.0,
      pitch: 0.0,
      move_speed: 5.0,
      mouse_sensitivity: 1.0,
      zoom: 90.0,
    }
  }
  
  pub fn print_details(&self) {
    println!("x: {}, y: {}, z: {}, pitch: {}, yaw: {}", self.position.x, self.position.y, self.position.z,
                                                        self.pitch, self.yaw);
  }
  
  pub fn set_mouse_sensitivity(&mut self, new_sensitivity: f32) {
    self.mouse_sensitivity = new_sensitivity;
  }
  
  pub fn set_move_speed(&mut self, speed: f32) {
    self.move_speed = speed;
  }
  
  pub fn set_position(&mut self, pos: Vector3<f32>) {
    self.position = pos;
  }
  
  pub fn set_pitch(&mut self, pitch: f32) {
    self.pitch = pitch;
    self.update_camera_vector();
  }
  
  pub fn set_yaw(&mut self, yaw: f32) {
    self.yaw = yaw;
    self.update_camera_vector();
  }
  
  fn update_camera_vector(&mut self) {
    let mut front = Vector3::new(0.0, 0.0, 0.0);
      
    front.x = cgmath::Deg(self.yaw).cos() * cgmath::Deg(self.pitch).cos();
    front.y = cgmath::Deg(self.pitch).sin();
    front.z = cgmath::Deg(self.yaw).sin() * cgmath::Deg(self.pitch).cos();
    self.front = InnerSpace::normalize(front);
    
    self.right = InnerSpace::normalize(self.front.cross(self.world_up));
    self.up = InnerSpace::normalize(self.right.cross(self.front));
  }
  
  pub fn process_mouse_movement(&mut self, x_offset: f32, y_offset: f32) {
    let x_offset = x_offset * self.mouse_sensitivity;
    let y_offset = y_offset * self.mouse_sensitivity;
    
    self.yaw += x_offset;
    self.pitch += y_offset;
    
    // constrain pitch
    if self.pitch > 89.0 {
      self.pitch = 89.0;
    }
    if self.pitch < -89.0 {
      self.pitch = -89.0;
    }
    
    self.update_camera_vector();
  }
  
  pub fn process_movement(&mut self, direction: Direction, delta_time: f32) {
    match direction {
      Direction::Forward => {
        self.position += self.front*self.move_speed*delta_time;
      },
      Direction::Backward => {
        self.position -= self.front*self.move_speed*delta_time; 
      },
      Direction::Right => {
        self.position += self.right*self.move_speed*delta_time;
      },
      Direction::Left => {
        self.position -= self.right*self.move_speed*delta_time;
      },
      Direction::Up => {
        self.position += self.up*self.move_speed*delta_time;
      },
      Direction::Down => {
        self.position -= self.up*self.move_speed*delta_time;
      },
      Direction::PositiveX => {
        self.position.x += self.move_speed * delta_time;
      },
      Direction::NegativeX => {
        self.position.x -= self.move_speed * delta_time;
      }
      Direction::PositiveY => {
        self.position.y += self.move_speed * delta_time;
      },
      Direction::NegativeY => {
        self.position.y -= self.move_speed * delta_time;
      }
      Direction::PositiveZ => {
        self.position.z += self.move_speed * delta_time;
      },
      Direction::NegativeZ => {
        self.position.z -= self.move_speed * delta_time;
      },
      Direction::YAlignedForward => {
        let mut t_camera = self.clone();
        t_camera.set_pitch(0.0);
        t_camera.process_movement(Direction::Forward, delta_time);
        self.position = t_camera.get_position();
      },
      Direction::YAlignedBackward => {
        let mut t_camera = self.clone();
        t_camera.set_pitch(0.0);
        t_camera.process_movement(Direction::Backward, delta_time);
        self.position = t_camera.get_position();
      },
      Direction::YAlignedLeft => {
        let mut t_camera = self.clone();
        t_camera.set_pitch(0.0);
        t_camera.process_movement(Direction::Left, delta_time);
        self.position = t_camera.get_position();
      },
      Direction::YAlignedRight => {
        let mut t_camera = self.clone();
        t_camera.set_pitch(0.0);
        t_camera.process_movement(Direction::Right, delta_time);
        self.position = t_camera.get_position();
      }
    }
  }
  
  pub fn get_front(&self) -> Vector3<f32> {
    self.front
  }
  
  pub fn get_look_at(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
    (self.position, self.position+self.front, self.up)
  }
  
  pub fn get_view_matrix(&self) -> Matrix4<f32> {
    Matrix4::look_at(Point3::from_vec(self.position), Point3::from_vec(self.position +
                     self.front), self.up)
  }
  
  pub fn get_position(&self) -> Vector3<f32> {
    self.position
  }
}
