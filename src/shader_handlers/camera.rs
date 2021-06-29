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
    Camera {
      fov: 60.0,
      znear: 0.1,
      zfar: 256.0,
      
      rotation: [0.0, -135.0, 0.0],
      position: [0.0, 0.5, -1.0],
      view_pos: [0.0; 4],
      
      rotation_speed: 1.0,
      movement_speed: 1.0,
      
      perspective: Camera::perspective(60.0, 1280.0/720.0, 0.1, 256.0, true),
      view: Camera::view([1.0, 0.0, 4.0], [0.0, 150.0, 0.0], CameraType::LookAt, true),
      
      camera_type: CameraType::FirstPerson,
      
      flip_y: true,
      
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
    
    self.position = Camera::vec3_add(self.position, Camera::vec3_sacle(camera_front, ms));
    
    self.update_view_matrix();
  }
  
  pub fn backward(&mut self, delta_time: f32) {
    let camera_front = Camera::camera_front(self.rotation);
    
    let ms = self.movement_speed * delta_time;
    
    self.position = Camera::vec3_minus(self.position, Camera::vec3_sacle(camera_front, ms));
    
    self.update_view_matrix();
  }
  
  pub fn left(&mut self, delta_time: f32) {
    let camera_front = Camera::camera_front(self.rotation);
    
    let ms = self.movement_speed * delta_time;
    
    self.position = Camera::vec3_minus(self.position, Camera::vec3_sacle(Camera::vec3_normalise(Camera::vec3_cross(camera_front, [0.0, 1.0, 0.0])), ms));
    
    self.update_view_matrix();
  }
  
  pub fn right(&mut self, delta_time: f32) {
    let camera_front = Camera::camera_front(self.rotation);
    
    let ms = self.movement_speed * delta_time;
    
    self.position = Camera::vec3_add(self.position, Camera::vec3_sacle(Camera::vec3_normalise(Camera::vec3_cross(camera_front, [0.0, 1.0, 0.0])), ms));
    
    self.update_view_matrix();
  }
  
  pub fn update_view_matrix(&mut self) {
    self.view = Camera::view(self.position, self.rotation, CameraType::FirstPerson, self.flip_y);
    self.view_pos = [-self.position[0], self.position[1], -self.position[2], 1.0];
    
    self.updated = true;
  }
  
  // camera.translate(glm::vec3(0.0f, 0.0f, (float)wheelDelta * 0.005f));
  pub fn update_translate(&mut self, delta: [f32; 3]) {
    self.position = Camera::vec3_add(self.position, delta);
    self.update_view_matrix();
  }
  
  pub fn update_rotate(&mut self, delta: [f32; 3]) {
    self.rotation = Camera::vec3_add(self.rotation, delta);
    self.update_view_matrix();
  }
  
  pub fn update_aspect_ratio(&mut self, aspect: f32) {
    self.perspective = Camera::perspective(self.fov, aspect, self.znear, self.zfar, self.flip_y);
  }
  
  pub fn perspective(fovy: f32, aspect: f32, znear: f32, zfar: f32, flip_y: bool) -> [f32; 16] {
    let rad = fovy.to_radians();
    let tan_half_fovy = (rad / 2.0).tan();
    
    let r = 4;
    
    let mut matrix = [0.0; 16];
    matrix[r*0 + 0] = 1.0 / (aspect * tan_half_fovy);
    matrix[r*1 + 1] = -1.0 / (tan_half_fovy);
    matrix[r*2 + 2] = -(zfar - znear) / (zfar - znear);
    matrix[r*2 + 3] = -1.0;
    matrix[r*3 + 2] = -(2.0 * zfar * znear) / (zfar - znear);
    
    if flip_y {
      matrix[r*1 + 1] *= -1.0;
    }
    
    matrix
  }
  
  pub fn view(position: [f32; 3], rotation: [f32; 3], camera_type: CameraType, flip_y: bool) -> [f32; 16] {
    let mut rot_m = Camera::mat4_identity();
    
    rot_m = Camera::rotate(rot_m, (rotation[0] * if flip_y { -1.0 } else { 1.0 }).to_radians(), [1.0, 0.0, 0.0]);
    rot_m = Camera::rotate(rot_m, (rotation[1]).to_radians(), [0.0, 1.0, 0.0]);
    rot_m = Camera::rotate(rot_m, (rotation[2]).to_radians(), [0.0, 0.0, 1.0]);
    
    let mut translation = position;
    if flip_y {
      translation[1] *= -1.0;
    }
    let trans_m = Camera::translate(Camera::mat4_identity(), translation);
    
    match camera_type {
      CameraType::FirstPerson => {
        // rot_m * trans_m
        Camera::mat4_mul(trans_m, rot_m)
      },
      CameraType::LookAt => {
        // trans_m * rot_m
        Camera::mat4_mul(rot_m, trans_m)
      }
    }
  }
  
  pub fn rotate(mat4: [f32; 16], angle: f32, axis: [f32; 3]) -> [f32; 16] {
    let a = angle;
    let c = a.cos();
    let s = a.sin();
    
    let axis = axis;
    let temp = Camera::vec3_sacle(axis, 1.0 - c);
    
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
  
  pub fn dot_product(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
  }
  
  pub fn translate(mut m: [f32; 16], v: [f32; 3]) -> [f32; 16] {
    let r = 4;
    
    let m_0 = Camera::vec4_sacle([m[r*0 + 0], m[r*0 + 1], m[r*0 + 2], m[r*0 + 3]], v[0]);
    let m_1 = Camera::vec4_sacle([m[r*1 + 0], m[r*1 + 1], m[r*1 + 2], m[r*1 + 3]], v[1]);
    let m_2 = Camera::vec4_sacle([m[r*2 + 0], m[r*2 + 1], m[r*2 + 2], m[r*2 + 3]], v[2]);
    let m_3 = [m[r*3 + 0], m[r*3 + 1], m[r*3 + 2], m[r*3 + 3]];
    
    let row = [m_0[0] + m_1[0] + m_2[0] + m_3[0],
               m_0[1] + m_1[1] + m_2[1] + m_3[1],
               m_0[2] + m_1[2] + m_2[2] + m_3[2],
               m_0[3] + m_1[3] + m_2[3] + m_3[3],
              ];
    m[r*3 + 0] = row[0];
    m[r*3 + 1] = row[1];
    m[r*3 + 2] = row[2];
    m[r*3 + 3] = row[3];
    
    m
  }
  
  pub fn mat4_identity() -> [f32; 16] {
    [1.0, 0.0, 0.0, 0.0,
     0.0, 1.0, 0.0, 0.0,
     0.0, 0.0, 1.0, 0.0,
     0.0, 0.0, 0.0, 1.0]
  }
  
  pub fn mat4_mul(a: [f32; 16], b: [f32 ; 16]) -> [f32; 16] {
    let a11 = a[0];
    let a12 = a[1];
    let a13 = a[2];
    let a14 = a[3];
    
    let a21 = a[4];
    let a22 = a[5];
    let a23 = a[6];
    let a24 = a[7];
    
    let a31 = a[8];
    let a32 = a[9];
    let a33 = a[10];
    let a34 = a[11];
    
    let a41 = a[12];
    let a42 = a[13];
    let a43 = a[14];
    let a44 = a[15];
    
    let b11 = b[0];
    let b12 = b[1];
    let b13 = b[2];
    let b14 = b[3];
    
    let b21 = b[4];
    let b22 = b[5];
    let b23 = b[6];
    let b24 = b[7];
    
    let b31 = b[8];
    let b32 = b[9];
    let b33 = b[10];
    let b34 = b[11];
    
    let b41 = b[12];
    let b42 = b[13];
    let b43 = b[14];
    let b44 = b[15];
    
    [
      a11*b11 + a12*b21 + a13*b31 + a14*b41, a11*b12 + a12*b22 + a13*b32 + a14*b42, a11*b13 + a12*b23 + a13*b33 + a14*b43, a11*b14 + a12*b24 + a13*b34 + a14*b44,
      a21*b11 + a22*b21 + a23*b31 + a24*b41, a21*b12 + a22*b22 + a23*b32 + a24*b42, a21*b13 + a22*b23 + a23*b33 + a24*b43, a21*b14 + a22*b24 + a23*b34 + a24*b44,
      a31*b11 + a32*b21 + a33*b31 + a34*b41, a31*b12 + a32*b22 + a33*b32 + a34*b42, a31*b13 + a32*b23 + a33*b33 + a34*b43, a31*b14 + a32*b24 + a33*b34 + a34*b44,
      a41*b11 + a42*b21 + a43*b31 + a44*b41, a41*b12 + a42*b22 + a43*b32 + a44*b42, a41*b13 + a42*b23 + a43*b33 + a44*b43, a41*b14 + a42*b24 + a43*b34 + a44*b44
    ]
  }
  
  pub fn vec3_sacle(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
  }
  
  pub fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
  }
  
  pub fn vec3_minus(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
  }
  
  pub fn vec3_normalise(a: [f32; 3]) -> [f32; 3] {
    let mag = (a[0]*a[0] + a[1]*a[1] + a[2]*a[2]).sqrt();
    
    [a[0] / mag, a[1] / mag, a[2] / mag]
  }
  
  pub fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
     a[1]*b[2] - b[1]*a[2],
     a[2]*b[0] - b[2]*a[0],
     a[0]*b[1] - b[0]*a[1]
    ]
  }
  
  pub fn vec4_sacle(v: [f32; 4], s: f32) -> [f32; 4] {
    [v[0] * s, v[1] * s, v[2] * s, v[3] * s]
  }
  
  pub fn mat4_scale(mat4: [f32; 16], s: [f32; 3]) -> [f32; 16] {
    let mut scale_matrix = Camera::mat4_identity();
    scale_matrix[0] = s[0];
    scale_matrix[5] = s[1];
    scale_matrix[10] = s[2];
    
    Camera::mat4_mul(scale_matrix, mat4)
  }
  
  pub fn quat_to_mat4(quat: [f32; 4]) -> [f32; 16] {
    let mut matrix = Camera::mat4_identity();
    
    let x = quat[0];
    let y = quat[0];
    let z = quat[0];
    let w = quat[0];
    
    let r = 4;
    matrix[r*0 + 0] = 1.0 - 2.0*y*y - 2.0*z*z;
    matrix[r*0 + 1] = 2.0*x*y + 2.0*w*z;
    matrix[r*0 + 2] = 2.0*x*z - 2.0*w*y;
    matrix[r*0 + 3] = 0.0;
    
    matrix[r*1 + 0] = 2.0*x*y - 2.0*w*z;
    matrix[r*1 + 1] = 1.0 - 2.0*x*x - 2.0*z*z;
    matrix[r*1 + 2] = 2.0*y*z + 2.0*w*x;
    matrix[r*1 + 3] = 0.0;
    
    matrix[r*2 + 0] = 2.0*x*z - 2.0*w*y;
    matrix[r*2 + 1] = 2.0*y*z + 2.0*w*x;
    matrix[r*2 + 2] = 1.0 - 2.0*x*x - 2.0*y*y;
    matrix[r*2 + 3] = 0.0;
    
    matrix[r*3 + 0] = 0.0;
    matrix[r*3 + 1] = 0.0;
    matrix[r*3 + 2] = 0.0;
    matrix[r*3 + 3] = 1.0;
    
    matrix
  }
  
  fn camera_front(rotation: [f32; 3]) -> [f32; 3] {
    let mut cam_front = [0.0, 0.0, 0.0];
    cam_front[0] = -(rotation[0].to_radians()).cos() * (rotation[1].to_radians()).sin();
    cam_front[1] = (rotation[0].to_radians()).sin();
    cam_front[2] = (rotation[0].to_radians()).cos() * (rotation[1].to_radians()).cos();
    
    Camera::vec3_normalise(cam_front)
  }
}










