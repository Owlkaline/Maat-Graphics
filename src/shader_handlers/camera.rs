use crate::{
  extra::Math,
  glam::{Vec3, Vec4},
};

const TP_X_ROT_MIN: f32 = 89.0;
const TP_X_ROT_MAX: f32 = 189.0;

const FP_X_ROT_MIN: f32 = 91.0;
const FP_X_ROT_MAX: f32 = 269.0;

const FOV: f32 = 71.0;
const ZNEAR: f32 = 0.9;
const ZFAR: f32 = 100.0;

#[derive(Copy, Clone)]
pub enum CameraType {
  Fly,
  FirstPerson,
  ThirdPerson,
}

#[derive(Clone)]
pub struct Camera {
  fov: f32,
  znear: f32,
  zfar: f32,
  aspect: f32,

  // First and fly camera variables
  rotation: Vec3,
  position: Vec3,
  view_pos: Vec4,

  // Third person camera variables
  target: Vec3,
  offset: Vec3,

  movement_speed: f32,
  rotation_speed: f32,

  invert_x_rotation: f32,
  invert_y_rotation: f32,

  min_x_rotation: Option<f32>,
  max_x_rotation: Option<f32>,

  perspective: [f32; 16],
  view: [f32; 16],

  camera_type: CameraType,

  flip_y: bool,

  updated: bool, // Indicator if uniform buffers should be updated
}

impl Camera {
  pub fn new() -> Camera {
    let flip_y = false;

    let camera_type = CameraType::Fly;

    let (position, rotation) = (
      Vec3::new(0.4351558, -6.641949, 3.27347),
      Vec3::new(121.0, 0.0, 0.0),
    );

    let target = Vec3::splat(0.0);

    let aspect = 1280.0 / 720.0;

    let mut cam = Camera {
      fov: FOV,
      znear: ZNEAR,
      zfar: ZFAR,
      aspect,

      rotation,
      position,
      view_pos: Vec4::splat(0.0),

      target,
      offset: Vec3::new(0.0, -8.0, 5.0),

      movement_speed: 1.0,
      rotation_speed: 90.0, // degrees per second

      invert_x_rotation: 1.0,
      invert_y_rotation: 1.0,

      min_x_rotation: None,
      max_x_rotation: None,

      perspective: Math::perspective(FOV, aspect, ZNEAR, ZFAR, flip_y),
      view: Camera::view(position, rotation, camera_type, flip_y),

      camera_type,

      flip_y,

      updated: false,
    };

    cam.update_view_matrix();

    cam
  }

  pub fn set_fovy(&mut self, fovy: f32) {
    self.fov = fovy;
    self.update_view_matrix();
  }

  pub fn rotation(&self) -> Vec3 {
    self.rotation
  }

  pub fn min_x_rotation(&self) -> Option<f32> {
    self.min_x_rotation
  }

  pub fn max_x_rotation(&self) -> Option<f32> {
    self.max_x_rotation
  }

  pub fn set_min_x_rotation(&mut self, min_rot: f32) {
    self.min_x_rotation = Some(min_rot);
  }

  pub fn set_max_x_rotation(&mut self, max_rot: f32) {
    self.max_x_rotation = Some(max_rot);
  }

  pub fn set_movement_speed(&mut self, speed: f32) {
    self.movement_speed = speed;
  }

  pub fn set_fly_mode(&mut self) {
    self.camera_type = CameraType::Fly;
    self.min_x_rotation = None;
    self.max_x_rotation = None;
  }

  pub fn set_first_person_mode(&mut self) {
    self.camera_type = CameraType::FirstPerson;
    self.min_x_rotation = Some(FP_X_ROT_MIN);
    self.max_x_rotation = Some(FP_X_ROT_MAX);
  }

  pub fn set_third_person_mode(&mut self) {
    self.camera_type = CameraType::ThirdPerson;
    self.min_x_rotation = Some(TP_X_ROT_MIN);
    self.max_x_rotation = Some(TP_X_ROT_MAX);
  }

  pub fn invert_up_down(&mut self) {
    self.invert_x_rotation = -self.invert_x_rotation;
  }

  pub fn invert_left_right(&mut self) {
    self.invert_y_rotation = -self.invert_y_rotation;
  }

  pub fn set_up_down_inverted(&mut self, inverted: bool) {
    self.invert_x_rotation = {
      if inverted {
        -1.0
      } else {
        1.0
      }
    };
  }

  pub fn set_left_right_inverted(&mut self, inverted: bool) {
    self.invert_y_rotation = {
      if inverted {
        -1.0
      } else {
        1.0
      }
    };
  }

  pub fn set_rotation(&mut self, rot: [f32; 3]) {
    self.rotation = Vec3::from(rot);
  }

  pub fn follow_target(&mut self, target: [f32; 3]) {
    let target = Vec3::from(target);
    if self.target != target {
      self.target = Vec3::new(-target.x, -target.y, -target.z);
      self.position = self.target + self.offset;

      self.update_view_matrix();
    }
  }

  pub fn follow_target_lerp(&mut self, target: [f32; 3], percentage: f32) {
    let target = Vec3::from(target);
    //if self.target != target {
    //self.target = Vec3::new(-target.x, -target.y, -target.z);
    self.target = self.target.lerp(-target, percentage);
    self.position = self.target + self.offset;

    self.update_view_matrix();
    //}
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
    let camera_front = {
      match self.camera_type {
        CameraType::Fly => Camera::camera_front(self.rotation),
        CameraType::FirstPerson => {
          Camera::camera_front(Vec3::new(180.0, self.rotation.y, self.rotation.z))
        }
        CameraType::ThirdPerson => {
          let length = self.offset.length();

          self.rotation.x -= self.rotation_speed * delta_time;

          if let Some(min_x_rotation) = self.min_x_rotation {
            if self.rotation.x <= min_x_rotation {
              self.rotation.x = min_x_rotation;
            }
          }

          let new_camera_front = Camera::camera_front(self.rotation);
          let new_offset = new_camera_front.normalize_or_zero() * -length;

          self.offset = new_offset;

          Vec3::splat(0.0)
        }
      }
    };

    let ms = self.movement_speed * delta_time;

    self.position += camera_front * ms;

    self.update_view_matrix();
  }

  pub fn backward(&mut self, delta_time: f32) {
    let camera_front = {
      match self.camera_type {
        CameraType::Fly => Camera::camera_front(self.rotation),
        CameraType::FirstPerson => {
          Camera::camera_front(Vec3::new(-180.0, self.rotation.y, self.rotation.z))
        }
        CameraType::ThirdPerson => {
          let length = self.offset.length();

          self.rotation.x += self.rotation_speed * delta_time;

          if let Some(max_x_rotation) = self.max_x_rotation {
            if self.rotation.x >= max_x_rotation {
              self.rotation.x = max_x_rotation;
            }
          }

          let new_camera_front = Camera::camera_front(self.rotation);
          let new_offset = new_camera_front.normalize_or_zero() * -length;

          self.offset = new_offset;

          Vec3::splat(0.0)
        }
      }
    };

    let ms = self.movement_speed * delta_time;

    self.position -= camera_front * ms;

    self.update_view_matrix();
  }

  pub fn left(&mut self, delta_time: f32) {
    match self.camera_type {
      CameraType::Fly | CameraType::FirstPerson => {
        let camera_front = Camera::camera_front(self.rotation);

        let ms = self.movement_speed * delta_time;

        self.position += camera_front
          .cross(Vec3::new(0.0, 1.0, 0.0))
          .normalize_or_zero() *
          ms;
      }
      CameraType::ThirdPerson => {
        let length = self.offset.length();
        self.rotation.y -= self.rotation_speed * delta_time;
        let new_camera_front = Camera::camera_front(self.rotation);
        let new_offset = new_camera_front.normalize_or_zero() * (-length);
        self.offset = new_offset;
      }
    }

    self.update_view_matrix();
  }

  pub fn right(&mut self, delta_time: f32) {
    match self.camera_type {
      CameraType::Fly | CameraType::FirstPerson => {
        let camera_front = Camera::camera_front(self.rotation);

        let ms = self.movement_speed * delta_time;

        self.position -= camera_front
          .cross(Vec3::new(0.0, 1.0, 0.0))
          .normalize_or_zero() *
          ms
      }
      CameraType::ThirdPerson => {
        let length = self.offset.length();
        self.rotation.y += self.rotation_speed * delta_time;
        let new_camera_front = Camera::camera_front(self.rotation);
        let new_offset = new_camera_front.normalize_or_zero() * (-length);
        self.offset = new_offset;
      }
    }
    self.update_view_matrix();
  }

  pub fn update_view_matrix(&mut self) {
    self.view = Camera::view(self.position, self.rotation, self.camera_type, self.flip_y);
    self.view_pos = Vec4::new(self.position.x, self.position.y, self.position.z, 0.0) *
      Vec4::new(-1.0, 1.0, -1.0, 1.0);

    self.updated = true;
  }

  pub fn zoom_lerp(&mut self, offset: f32, percentage: f32) {
    match self.camera_type {
      CameraType::ThirdPerson => {
        let front = Camera::camera_front(self.rotation);
        let zoom_speed = -offset;

        let goal_offset = front * Vec3::splat(zoom_speed);

        self.offset = self.offset.lerp(goal_offset, percentage);

        self.update_view_matrix();
      }
      _ => {}
    }
  }

  pub fn zoom(&mut self, offset: f32) {
    match self.camera_type {
      CameraType::ThirdPerson => {
        let front = Camera::camera_front(self.rotation);
        let zoom_speed = -offset;

        self.offset += front * Vec3::splat(zoom_speed);

        self.update_view_matrix();
      }
      _ => {}
    }
  }

  // Rotate camera by degrees along the (x, y, z) axis
  pub fn rotate_by_degrees(&mut self, delta: [f32; 3]) {
    let delta = Vec3::from([
      delta[0] * self.invert_y_rotation,
      delta[1] * self.invert_x_rotation,
      delta[2],
    ]);

    match self.camera_type {
      CameraType::Fly | CameraType::FirstPerson => {
        self.rotation = self.rotation + delta;

        if let Some(max_x_rotation) = self.max_x_rotation {
          if self.rotation.x > max_x_rotation {
            self.rotation.x = max_x_rotation;
          }
        }

        if let Some(min_x_rotation) = self.min_x_rotation {
          if self.rotation.x < min_x_rotation {
            self.rotation.x = min_x_rotation;
          }
        }
      }
      CameraType::ThirdPerson => {
        let length = self.offset.length();
        self.rotation = self.rotation + delta;

        if let Some(max_x_rotation) = self.max_x_rotation {
          if self.rotation.x > max_x_rotation {
            self.rotation.x = max_x_rotation;
          }
        }

        if let Some(min_x_rotation) = self.min_x_rotation {
          if self.rotation.x < min_x_rotation {
            self.rotation.x = min_x_rotation;
          }
        }

        let new_camera_front = Camera::camera_front(self.rotation);
        let new_offset = new_camera_front.normalize_or_zero() * -length;
        self.offset = new_offset;
      }
    }
    self.update_view_matrix();
  }

  pub fn update_fovy(&mut self, fovy: f32) {
    self.perspective = Math::perspective(fovy, self.aspect, self.znear, self.zfar, self.flip_y);
    self.fov = fovy;
    self.update_view_matrix();
  }

  pub fn update_aspect_ratio(&mut self, aspect: f32) {
    self.perspective = Math::perspective(self.fov, aspect, self.znear, self.zfar, self.flip_y);
    self.aspect = aspect;
    self.update_view_matrix();
  }

  pub fn view(position: Vec3, rotation: Vec3, camera_type: CameraType, flip_y: bool) -> [f32; 16] {
    let mut rot_m = Math::mat4_identity();

    rot_m = Math::mat4_axis_rotate(
      rot_m,
      (rotation.x * if flip_y { -1.0 } else { 1.0 }).to_radians(),
      [1.0, 0.0, 0.0],
    );
    rot_m = Math::mat4_axis_rotate(rot_m, (rotation.y).to_radians(), [0.0, 1.0, 0.0]);
    rot_m = Math::mat4_axis_rotate(rot_m, (rotation.z).to_radians(), [0.0, 0.0, 1.0]);

    let mut translation = position;
    if flip_y {
      translation.y *= -1.0;
    }

    let trans_m = Math::mat4_translate_vec3(Math::mat4_identity(), translation.into());

    match camera_type {
      CameraType::FirstPerson | CameraType::Fly | CameraType::ThirdPerson => {
        // rot_m * trans_m
        Math::mat4_mul(trans_m, rot_m)
      } //CameraType::LookAt => {
        //  // trans_m * rot_m
        //  Math::mat4_mul(rot_m, trans_m)
        //}
    }
  }

  pub fn camera_front(rotation: Vec3) -> Vec3 {
    let mut cam_front = Vec3::splat(0.0);
    cam_front.x = -(rotation.x.to_radians()).cos() * (rotation.y.to_radians()).sin();
    cam_front.y = (rotation.x.to_radians()).sin();
    cam_front.z = (rotation.x.to_radians()).cos() * (rotation.y.to_radians()).cos();

    cam_front.normalize_or_zero()
  }
}
