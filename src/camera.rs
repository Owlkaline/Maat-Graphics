use cgmath;
use cgmath::Vector3;
use cgmath::Point3;
use cgmath::Angle;
use cgmath::EuclideanSpace;
use cgmath::prelude::InnerSpace;
use cgmath::Matrix4;

pub enum Direction {
  Forward,
  Backward,
  Left,
  Right,
}

#[derive(Clone)]
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
      move_speed: 1.0,
      mouse_sensitivity: 0.0,
      zoom: 90.0,
    }
  }

  pub fn default_gl() -> Camera {
    Camera {
      position: Vector3::new(0.0, 0.0, 0.0),
      front: Vector3::new(0.0, 0.0, 1.0),
      up: Vector3::new(0.0, 1.0, 0.0),
      right: Vector3::new(1.0, 0.0, 0.0),
  
      world_up: Vector3::new(0.0, -1.0, 0.0),
  
      yaw: 0.0,
      pitch: 0.0,
      move_speed: 1.0,
      mouse_sensitivity: 0.0,
      zoom: 90.0,
    }
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
  
  pub fn process_movement(&mut self, direction: Direction) {  
    match direction {
      Direction::Forward => {
        self.position += self.front*self.move_speed;
      },
      Direction::Backward => {
        self.position -= self.front*self.move_speed; 
      },
      Direction::Right => {
        self.position += self.right*self.move_speed;
      },
      Direction::Left => {
        self.position -= self.right*self.move_speed;
      }
    }
  }
    
  pub fn get_view_matrix(&self) -> Matrix4<f32> {
    Matrix4::look_at(Point3::from_vec(self.position), Point3::from_vec(self.position +
                     self.front), self.up)
  }  
}
