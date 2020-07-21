use crate::cgmath;
use crate::cgmath::{Vector2, Matrix4, ortho};

pub enum _Direction {
  Left,
  Right,
  Up,
  Down,
}

#[derive(Clone, PartialEq)]
pub struct OrthoCamera {
  position: Vector2<f32>,
  top: f32,
  right: f32,
  
  move_speed: f32,
  zoom: f32,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + t * (b-a)
}

impl OrthoCamera {
  pub fn new(width: f32, height: f32) -> OrthoCamera {
    OrthoCamera {
      position: Vector2::new(0.0, 0.0),
      top: height,
      right: width,
      
      move_speed: 5.0,
      zoom: 1.0,
    }
  }
  
  pub fn window_resized(&mut self, width: f32, height: f32) {
    self.top = height;
    self.right = width;
  }
  
  pub fn _process_movement(&mut self, direction: _Direction, delta_time: f32) {
    match direction {
      _Direction::Right => {
        self.position.x += self.move_speed*delta_time;
      },
      _Direction::Left => {
        self.position.x -= self.move_speed*delta_time;
      },
      _Direction::Up => {
        self.position.y += self.move_speed*delta_time;
      },
      _Direction::Down => {
        self.position.y -= self.move_speed*delta_time;
      },
    }
  }
  
  pub fn _get_view_matrix(&self) -> Matrix4<f32> {
    ortho(self.position.x, self.right+self.position.x, self.top+self.position.y, self.position.y, -1.0, 1.0)
  }
  
  pub fn _get_raw_view_matrix(&self) -> Matrix4<f32> {
    ortho(0.0, self.right, self.top, 0.0, -1.0, 1.0)
  }
  
  pub fn get_position(&self) -> Vector2<f32> {
    self.position
  }
  
  pub fn get_top(&self) -> f32 {
    self.top
  }
  
  pub fn get_right(&self) -> f32 {
    self.right
  }
  
  pub fn reset(&mut self, width: f32, height: f32) {
    self.position = Vector2::new(0.0, 0.0);
    self.right = width;
    self.top = height;
  }
  
  pub fn lerp_to_position(&mut self, goal_pos: Vector2<f32>, vel: Vector2<f32>) {
    self.position = Vector2::new(lerp(self.position.x, goal_pos.x, vel.x), lerp(self.position.y, goal_pos.y, vel.y));
  }
  
  pub fn lerp_to_size(&mut self, goal_size: Vector2<f32>, vel: Vector2<f32>) {
    self.right = lerp(self.right, goal_size.x, vel.x);
    self.top = lerp(self.top, goal_size.y, vel.y);
  }
}
