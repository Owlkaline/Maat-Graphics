use crate::shader_handlers::Math;

#[derive(Copy, Clone)]
pub enum CameraType {
  FirstPerson,
  LookAt,
}

pub struct Camera {
  fov: f32,
  znear: f32,
  zfar: f32,
  
  rotation: [f32; 3],
  position: [f32; 3],
  view_pos: [f32; 4],
  
  rotation_speed: f32,
  movement_speed: f32,
  
  perspective: [f32; 16],
  view: [f32; 16],
  
  camera_type: CameraType,
  
  flip_y: bool,
  
  updated: bool,
}

impl Camera {
  pub fn new() -> Camera {
    let mut flip_y = true;
    
    Camera {
      fov: 71.0,
      znear: 0.1,
      zfar: 256.0,
      
      rotation: [0.0, -135.0, 0.0],
      position: [0.0, 0.5, -1.0],
      view_pos: [0.0; 4],
      
      rotation_speed: 1.0,
      movement_speed: 1.0,
      
      perspective:  Math::perspective(71.0, 1280.0/720.0, 0.1, 256.0, flip_y),
      view:  Camera::view([1.0, 0.0, 4.0], [0.0, 150.0, 0.0], CameraType::LookAt, flip_y),
      
      camera_type: CameraType::FirstPerson,
      
      flip_y,
      
      updated: false,
    }
  }
  
  pub fn set_movement_speed(&mut self, speed: f32) {
    self.movement_speed = speed;
  }
  
  pub fn perspective_matrix(&self) -> [f32; 16] {
    self.perspective
  }
  
  pub fn view_matrix(&self) -> [f32; 16] {
    self.view
  }
  
  pub fn is_updated(&self) -> bool {
    self.updated
  }
  
  pub fn forward(&mut self, delta_time: f32) {
    let camera_front = Camera::camera_front(self.rotation);
    
    let ms = self.movement_speed * delta_time;
    
    self.position =  Math::vec3_add(self.position,  Math::vec3_mul_f32(camera_front, ms));
    
    self.update_view_matrix();
  }
  
  pub fn backward(&mut self, delta_time: f32) {
    let camera_front = Camera::camera_front(self.rotation);
    
    let ms = self.movement_speed * delta_time;
    
    self.position =  Math::vec3_minus(self.position,  Math::vec3_mul_f32(camera_front, ms));
    
    self.update_view_matrix();
  }
  
  pub fn left(&mut self, delta_time: f32) {
    let camera_front = Camera::camera_front(self.rotation);
    
    let ms = self.movement_speed * delta_time;
    
    self.position =  Math::vec3_add(self.position,  Math::vec3_mul_f32( Math::vec3_normalise( Math::vec3_cross(camera_front, [0.0, 1.0, 0.0])), ms));
    
    self.update_view_matrix();
  }
  
  pub fn right(&mut self, delta_time: f32) {
    let camera_front = Camera::camera_front(self.rotation);
    
    let ms = self.movement_speed * delta_time;
    
    self.position = Math::vec3_minus(self.position, Math::vec3_mul_f32(Math::vec3_normalise(Math::vec3_cross(camera_front, [0.0, 1.0, 0.0])), ms));
    
    self.update_view_matrix();
  }
  
  pub fn update_view_matrix(&mut self) {
    self.view = Camera::view(self.position, self.rotation, CameraType::FirstPerson, self.flip_y);
    self.view_pos = [-self.position[0], self.position[1], -self.position[2], 1.0];
    
    self.updated = true;
  }
  
  // camera.translate(glm::vec3(0.0f, 0.0f, (float)wheelDelta * 0.005f));
  pub fn update_translate(&mut self, delta: [f32; 3]) {
    self.position = Math::vec3_add(self.position, delta);
    self.update_view_matrix();
  }
  
  pub fn update_rotate(&mut self, delta: [f32; 3]) {
    self.rotation = Math::vec3_add(self.rotation, delta);
    
    let angle_limit = 85.0;
    
    if self.rotation[0] > 180.0+angle_limit {
      self.rotation[0] = 180.0+angle_limit;
    }
    if self.rotation[0] < 180.0-angle_limit {
      self.rotation[0] = 180.0-angle_limit;
    }
    self.update_view_matrix();
  }
  
  pub fn update_aspect_ratio(&mut self, aspect: f32) {
    self.perspective = Math::perspective(self.fov, aspect, self.znear, self.zfar, self.flip_y);
  }
  
  pub fn view(position: [f32; 3], rotation: [f32; 3], camera_type: CameraType, flip_y: bool) -> [f32; 16] {
    let mut rot_m =  Math::mat4_identity();
    
    rot_m = Camera::rotate(rot_m, (rotation[0] * if flip_y { -1.0 } else { 1.0 }).to_radians(), [1.0, 0.0, 0.0]);
    rot_m = Camera::rotate(rot_m, (rotation[1]).to_radians(), [0.0, 1.0, 0.0]);
    rot_m = Camera::rotate(rot_m, (rotation[2]).to_radians(), [0.0, 0.0, 1.0]);
    
    let mut translation = position;
    if flip_y {
      translation[1] *= -1.0;
    }
    let trans_m = Math::mat4_translate_vec3(Math::mat4_identity(), translation);
    
    match camera_type {
      CameraType::FirstPerson => {
        // rot_m * trans_m
         Math::mat4_mul(trans_m, rot_m)
      },
      CameraType::LookAt => {
        // trans_m * rot_m
         Math::mat4_mul(rot_m, trans_m)
      }
    }
  }
  
  pub fn rotate(mat4: [f32; 16], angle: f32, axis: [f32; 3]) -> [f32; 16] {
    let a = angle;
    let c = a.cos();
    let s = a.sin();
    
    let axis = axis;
    let temp =  Math::vec3_mul_f32(axis, 1.0 - c);
    
    let mut rotate = [0.0; 16];
    
    let r = 4;
    
    rotate[r*0 + 0] = c + temp[0] * axis[0];
    rotate[r*0 + 1] = 0.0 + temp[0] * axis[1] + s * axis[2];
    rotate[r*0 + 2] = 0.0 + temp[0] * axis[2] - s * axis[1];
    
    rotate[r*1 + 0] = 0.0 + temp[1] * axis[0] - s * axis[2];
    rotate[r*1 + 1] = c + temp[1] * axis[1];
    rotate[r*1 + 2] = 0.0 + temp[1] * axis[2] + s * axis[0];
    
    rotate[r*2 + 0] = 0.0 + temp[2] * axis[0] + s * axis[1];
    rotate[r*2 + 1] = 0.0 + temp[2] * axis[1] - s * axis[0];
    rotate[r*2 + 2] = c + temp[2] * axis[2];
    
    let mut result = [0.0; 16];
    
    result[r*0 + 0] = mat4[r*0 + 0] * rotate[r*0 + 0] + mat4[r*1 + 0] * rotate[r*0 + 1] + mat4[r*2 + 0] * rotate[r*0 + 2];
    result[r*0 + 1] = mat4[r*0 + 1] * rotate[r*0 + 0] + mat4[r*1 + 1] * rotate[r*0 + 1] + mat4[r*2 + 1] * rotate[r*0 + 2];
    result[r*0 + 2] = mat4[r*0 + 2] * rotate[r*0 + 0] + mat4[r*1 + 2] * rotate[r*0 + 1] + mat4[r*2 + 2] * rotate[r*0 + 2];
    result[r*0 + 3] = mat4[r*0 + 3] * rotate[r*0 + 0] + mat4[r*1 + 3] * rotate[r*0 + 1] + mat4[r*2 + 3] * rotate[r*0 + 2];
    
    result[r*1 + 0] = mat4[r*0 + 0] * rotate[r*1 + 0] + mat4[r*1 + 0] * rotate[r*1 + 1] + mat4[r*2 + 0] * rotate[r*1 + 2];
    result[r*1 + 1] = mat4[r*0 + 1] * rotate[r*1 + 0] + mat4[r*1 + 1] * rotate[r*1 + 1] + mat4[r*2 + 1] * rotate[r*1 + 2];
    result[r*1 + 2] = mat4[r*0 + 2] * rotate[r*1 + 0] + mat4[r*1 + 2] * rotate[r*1 + 1] + mat4[r*2 + 2] * rotate[r*1 + 2];
    result[r*1 + 3] = mat4[r*0 + 3] * rotate[r*1 + 0] + mat4[r*1 + 3] * rotate[r*1 + 1] + mat4[r*2 + 3] * rotate[r*1 + 2];
    
    result[r*2 + 0] = mat4[r*0 + 0] * rotate[r*2 + 0] + mat4[r*1 + 0] * rotate[r*2 + 1] + mat4[r*2 + 0] * rotate[r*2 + 2];
    result[r*2 + 1] = mat4[r*0 + 1] * rotate[r*2 + 0] + mat4[r*1 + 1] * rotate[r*2 + 1] + mat4[r*2 + 1] * rotate[r*2 + 2];
    result[r*2 + 2] = mat4[r*0 + 2] * rotate[r*2 + 0] + mat4[r*1 + 2] * rotate[r*2 + 1] + mat4[r*2 + 2] * rotate[r*2 + 2];
    result[r*2 + 3] = mat4[r*0 + 3] * rotate[r*2 + 0] + mat4[r*1 + 3] * rotate[r*2 + 1] + mat4[r*2 + 3] * rotate[r*2 + 2];
    
    result[r*3 + 0] = mat4[r*3 + 0];
    result[r*3 + 1] = mat4[r*3 + 1];
    result[r*3 + 2] = mat4[r*3 + 2];
    result[r*3 + 3] = mat4[r*3 + 3];
    
    result
  }
  
  fn camera_front(rotation: [f32; 3]) -> [f32; 3] {
    let mut cam_front = [0.0, 0.0, 0.0];
    cam_front[0] = -(rotation[0].to_radians()).cos() * (rotation[1].to_radians()).sin();
    cam_front[1] = (rotation[0].to_radians()).sin();
    cam_front[2] = (rotation[0].to_radians()).cos() * (rotation[1].to_radians()).cos();
    
     Math::vec3_normalise(cam_front)
  }
}










