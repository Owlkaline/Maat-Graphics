use crate::math;

use cgmath;
use cgmath::{dot, InnerSpace, SquareMatrix, PerspectiveFov, Deg, Vector2, Vector3, Vector4, Point3, Angle, 
             EuclideanSpace, Matrix3, Matrix4};

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
  
  target: Vector3<f32>,
  horz_angle: f32,
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
      
      target: Vector3::new(0.0, 0.0, 0.0),
      horz_angle: 0.0,
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
      target: Vector3::new(0.0, 0.0, 0.0),
      horz_angle: 0.0,
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
  
  // Orbiting Camera stuff
  //
  //
  
  pub fn set_target(&mut self, point: Vector3<f32>) {
    self.target = point;
  }
  
  pub fn change_zoom(&mut self, zoom_delta: f32, delta_time: f32) {
    self.zoom += zoom_delta*delta_time;
    if self.zoom < 0.0 {
      self.zoom = 0.0;
    }
  }
  
  pub fn process_orbiting_camera_movement(&mut self, x_offset: f32, y_offset: f32) {
    let x_offset = x_offset * self.mouse_sensitivity;
    let y_offset = y_offset * self.mouse_sensitivity;
    
    self.rotate_camera_horizontally(x_offset);
    self.rotate_camera_vertically(y_offset);
  }
  
  pub fn rotate_camera_horizontally(&mut self, angle_delta: f32) {
    let rotation_matrix = Matrix4::from_axis_angle(Vector3::new(0.0, -1.0, 0.0), Deg(angle_delta));
    
    self.horz_angle += angle_delta;
    self.front = Vector3::new(
                   self.front.x * rotation_matrix[0][0] + self.front.y * rotation_matrix[0][1] + self.front.z*rotation_matrix[0][2],
                   self.front.x * rotation_matrix[1][0] + self.front.y * rotation_matrix[1][1] + self.front.z*rotation_matrix[1][2],
                   self.front.x * rotation_matrix[2][0] + self.front.y * rotation_matrix[2][1] + self.front.z*rotation_matrix[2][2]
                   );
    self.up = Vector3::new(
                   self.up.x * rotation_matrix[0][0] + self.up.y * rotation_matrix[0][1] + self.up.z*rotation_matrix[0][2],
                   self.up.x * rotation_matrix[1][0] + self.up.y * rotation_matrix[1][1] + self.up.z*rotation_matrix[1][2],
                   self.up.x * rotation_matrix[2][0] + self.up.y * rotation_matrix[2][1] + self.up.z*rotation_matrix[2][2]
                   );
    self.right = Vector3::new(
                   self.right.x * rotation_matrix[0][0] + self.right.y * rotation_matrix[0][1] + self.right.z*rotation_matrix[0][2],
                   self.right.x * rotation_matrix[1][0] + self.right.y * rotation_matrix[1][1] + self.right.z*rotation_matrix[1][2],
                   self.right.x * rotation_matrix[2][0] + self.right.y * rotation_matrix[2][1] + self.right.z*rotation_matrix[2][2]
                   );
    self.position = self.target - self.zoom * self.front;
  }
  
  pub fn rotate_camera_vertically(&mut self, mut angle_delta: f32) {
    let old_angle = self.pitch;
    self.pitch += angle_delta;
    if self.pitch > 89.0 {
      self.pitch = 89.0;
      angle_delta = 89.0 - old_angle;
    }
    if self.pitch < -89.0 {
      self.pitch = -89.0;
      angle_delta = -89.0 - old_angle;
    }
    
    let rotation_matrix = Matrix4::from_axis_angle(self.right.normalize(), Deg(angle_delta));
    
    self.front = Vector3::new(
                   self.front.x * rotation_matrix[0][0] + self.front.y * rotation_matrix[0][1] + self.front.z*rotation_matrix[0][2],
                   self.front.x * rotation_matrix[1][0] + self.front.y * rotation_matrix[1][1] + self.front.z*rotation_matrix[1][2],
                   self.front.x * rotation_matrix[2][0] + self.front.y * rotation_matrix[2][1] + self.front.z*rotation_matrix[2][2]
                   );
    self.up = Vector3::new(
                   self.up.x * rotation_matrix[0][0] + self.up.y * rotation_matrix[0][1] + self.up.z*rotation_matrix[0][2],
                   self.up.x * rotation_matrix[1][0] + self.up.y * rotation_matrix[1][1] + self.up.z*rotation_matrix[1][2],
                   self.up.x * rotation_matrix[2][0] + self.up.y * rotation_matrix[2][1] + self.up.z*rotation_matrix[2][2]
                   );
    self.position = self.target - self.zoom * self.front;
  }
  
  // First Person Camera
  //
  //
  
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
  
  // Useful camera functions
  //
  //
  
  // Fix world to screen size at different aspect ratios
  pub fn world_to_screen_coords(&self, position: Vector3<f32>, window_dim: Vector2<f32>) -> Vector2<f32> {
    let aspect = window_dim.y / window_dim.x;
    
    let position = Vector4::new(position.x, position.y, position.z,1.0);
   // let view = self.get_view_matrix();
    let view = Camera::create_view_matrix(self.position, self.position +
                     self.front, self.up);
    let perspective = self.get_perspective_matrix(aspect);
    
    let mut screen_coords = (perspective * view * position).normalize();
    screen_coords.x /= screen_coords.w;
    screen_coords.y /= screen_coords.w;
    screen_coords.z /= screen_coords.w;
    screen_coords.w = 1.0;
    screen_coords.x *= aspect*aspect;
    //println!("coords: {:?}", screen_coords);
    
    // full aspect 0.396875 x -6 to 6    // 1/12
    // wind aspect 0.5625 x -3 to 3      // 1/6
    // wind aspect 0.79375 -1.5 to 1.5   // 1/3
    // wind aspect 1.0    x -1 to 1     //  1/2
    
    //x*0.396875= 0.083333333
    //x = 0.083333333/0.396875
    // x= 0.209973753
    
    //unnormalise
    //x*0.5625 = 0.16666666
    //x = 0.16666666/0.5625
    // x = 0.296296284
    
    //x*0.79375= 0.333333333
    //x = 0.333333333/0.79375
    // x= 0.419947507
    
    // x*1.0 = 0.5
    // x = 0.5/1.0
    //x = 0.5
    
    //aspect 1 = 0.5
    //// let scaled_aspect = aspect*(aspect*0.5);
   // let x = ((screen_coords.x+3.0)*window_dim.x)*/*scaled_aspect;*/0.296296284*aspect;
    let x = ((screen_coords.x+1.0)*window_dim.x)*0.5;
    let y = ((screen_coords.y-1.0)*window_dim.y)*-0.5;
    //println!("aspect {}", aspect);
    Vector2::new(x,y)
  }
  
  pub fn mouse_to_world_ray(&self, mouse: Vector2<f32>, window_dim: Vector2<f32>) -> Vector3<f32> {
    let fov = 60.0;
    let aspect = window_dim.x / window_dim.y;
    let near = 0.1;
    let far = 256.0;
    // let f = (math::to_radians(fov) / 2.0).cot();
    let perspective = PerspectiveFov {
      fovy: Deg(fov).into(),
      aspect,
      near,
      far,
    };
    
    let perspective = perspective.to_perspective();
    
    let view = self.get_view_matrix();
    
    let invt_view = view.invert().unwrap();
    let invt_perspective = Matrix4::from(perspective).invert().unwrap();
    
    // normalise mouse coords
    let x = (2.0*mouse.x) / window_dim.x - 1.0;
    let y = -(2.0*mouse.y) / window_dim.y + 1.0;
    
    let clip_coords = Vector4::new(x, y, -1.0, 1.0);
    
    // clip to eye space
    let eye_matrix = invt_perspective * clip_coords;
    let eye_coords = Vector4::new(eye_matrix.x, eye_matrix.y, -1.0, 0.0);
    
    let world_matrix = invt_view * eye_coords;
    let mouse_ray = Vector3::new(world_matrix.x, world_matrix.y, world_matrix.z).normalize();
    
    mouse_ray
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
  
  fn create_view_matrix(eye: Vector3<f32>, center: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    let dir = center - eye;
    
    let f = (dir).normalize();
    let s = (f.cross(up)).normalize();
    let u = s.cross(f);
    
    let look_at_matrix = Matrix4::from_cols(Vector4::new(s.x,           u.x,        -f.x,         0.0), 
                                            Vector4::new(s.y,           u.y,        -f.y,         0.0), 
                                            Vector4::new(s.z,           u.z,        -f.z,         0.0), 
                                            Vector4::new(-dot(eye, s), -dot(eye, u), dot(eye, f), 1.0));
    
    look_at_matrix
  }
  
  fn get_perspective_matrix(&self, aspect: f32) -> Matrix4<f32> {
    let near = 0.1;
    let far = 256.0;
    let fov = 60.0;
    let f = 1.0 / (math::to_radians(fov) / 2.0).tan();
    
    let perspective = Matrix4::from_cols(
                      Vector4::new(f / aspect, 0.0,   0.0,                               0.0),
                      Vector4::new(0.0,        f,     0.0,                               0.0),
                      Vector4::new(0.0,        0.0,   (far + near) / (near - far),      -1.0),
                      Vector4::new(0.0,        0.0,   (2.0 * far * near) / (near - far), 0.0)
                    );
                
    perspective
  }
  
  pub fn get_position(&self) -> Vector3<f32> {
    self.position
  }
}
