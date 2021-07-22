use crate::shader_handlers::Math;

#[derive(Copy, Clone)]
pub enum CameraType {
  Fly,
  FirstPerson,
  ThirdPerson,
  LookAt,
}

pub struct Camera {
  fov: f32,
  znear: f32,
  zfar: f32,

  // First and fly camera variables
  rotation: [f32; 3],
  position: [f32; 3],
  view_pos: [f32; 4],

  // Third person camera variables
  target: [f32; 3],
  offset: [f32; 3],

  movement_speed: f32,
  rotation_speed: f32,

  perspective: [f32; 16],
  view: [f32; 16],

  camera_type: CameraType,

  flip_y: bool,

  updated: bool, // Indicator if uniform buffers should be updated
}

impl Camera {
  pub fn new() -> Camera {
    let flip_y = false;

    let camera_type = CameraType::ThirdPerson;

    let (position, rotation) = match camera_type {
      CameraType::FirstPerson | CameraType::Fly | CameraType::ThirdPerson => {
        ([0.4351558, -6.641949, 3.27347], [121.0, 0.0, 0.0])
      }
      CameraType::LookAt => (
        [-0.21398444, 0.36948895, -7.2325215],
        [122.20079, 91.60079, 0.0],
      ),
    };

    let target = [0.0; 3];

    let mut cam = Camera {
      fov: 71.0,
      znear: 0.1,
      zfar: 256.0,

      rotation,
      position,
      view_pos: [0.0; 4],

      target,
      offset: [0.0, -8.0, 5.0],

      movement_speed: 1.0,
      rotation_speed: 90.0, // degrees per second

      perspective: Math::perspective(71.0, 1280.0 / 720.0, 0.1, 256.0, flip_y),
      view: Camera::view(position, rotation, camera_type, flip_y),

      camera_type,

      flip_y,

      updated: false,
    };

    cam.update_view_matrix();

    cam
  }

  pub fn set_movement_speed(&mut self, speed: f32) {
    self.movement_speed = speed;
  }

  pub fn set_first_person(&mut self) {
    self.camera_type = CameraType::FirstPerson;
  }

  pub fn set_look_at(&mut self) {
    self.camera_type = CameraType::LookAt;
  }

  pub fn set_rotation(&mut self, rot: [f32; 3]) {
    self.rotation = rot;
  }

  pub fn follow_target(&mut self, target: [f32; 3]) {
    if !Math::vec3_equals(self.target, target) {
      self.target = [-target[0], -target[1], -target[2]];
      self.position = Math::vec3_add(self.target, self.offset);

      self.update_view_matrix();
    }
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
          Camera::camera_front([180.0, self.rotation[1], self.rotation[2]])
        }
        CameraType::ThirdPerson => {
          let length = Math::vec3_mag(self.offset);

          self.rotation[0] -= self.rotation_speed * delta_time;
          if self.rotation[0] <= 89.0 {
            self.rotation[0] = 89.0;
          }

          let new_camera_front = Camera::camera_front(self.rotation);
          let new_offset = Math::vec3_set_mag(new_camera_front, -length);

          self.offset = new_offset;

          [0.0, 0.0, 0.0]
        }
        _ => [0.0, 0.0, 0.0],
      }
    };

    let ms = self.movement_speed * delta_time;

    self.position = Math::vec3_add(self.position, Math::vec3_mul_f32(camera_front, ms));

    self.update_view_matrix();
  }

  pub fn backward(&mut self, delta_time: f32) {
    let camera_front = {
      match self.camera_type {
        CameraType::Fly => Camera::camera_front(self.rotation),
        CameraType::FirstPerson => {
          Camera::camera_front([-180.0, self.rotation[1], self.rotation[2]])
        }
        CameraType::ThirdPerson => {
          let length = Math::vec3_mag(self.offset);

          self.rotation[0] += self.rotation_speed * delta_time;
          if self.rotation[0] >= 189.0 {
            self.rotation[0] = 189.0;
          }

          let new_camera_front = Camera::camera_front(self.rotation);
          let new_offset = Math::vec3_set_mag(new_camera_front, -length);

          self.offset = new_offset;

          [0.0; 3]
        }
        _ => [0.0, 0.0, 0.0],
      }
    };

    let ms = self.movement_speed * delta_time;

    self.position = Math::vec3_minus(self.position, Math::vec3_mul_f32(camera_front, ms));

    self.update_view_matrix();
  }

  pub fn left(&mut self, delta_time: f32) {
    match self.camera_type {
      CameraType::Fly | CameraType::FirstPerson => {
        let camera_front = Camera::camera_front(self.rotation);

        let ms = self.movement_speed * delta_time;

        self.position = Math::vec3_add(
          self.position,
          Math::vec3_mul_f32(
            Math::vec3_normalise(Math::vec3_cross(camera_front, [0.0, 1.0, 0.0])),
            ms,
          ),
        );
      }
      CameraType::ThirdPerson => {
        let length = Math::vec3_mag(self.offset);
        self.rotation[1] -= self.rotation_speed * delta_time;
        let new_camera_front = Camera::camera_front(self.rotation);
        let new_offset = Math::vec3_set_mag(new_camera_front, -length);
        self.offset = new_offset;
      }
      _ => {}
    }

    self.update_view_matrix();
  }

  pub fn right(&mut self, delta_time: f32) {
    match self.camera_type {
      CameraType::Fly | CameraType::FirstPerson => {
        let camera_front = Camera::camera_front(self.rotation);

        let ms = self.movement_speed * delta_time;

        self.position = Math::vec3_minus(
          self.position,
          Math::vec3_mul_f32(
            Math::vec3_normalise(Math::vec3_cross(camera_front, [0.0, 1.0, 0.0])),
            ms,
          ),
        );
      }
      CameraType::ThirdPerson => {
        let length = Math::vec3_mag(self.offset);
        self.rotation[1] += self.rotation_speed * delta_time;
        let new_camera_front = Camera::camera_front(self.rotation);
        let new_offset = Math::vec3_set_mag(new_camera_front, -length);
        self.offset = new_offset;
      }
      _ => {}
    }
    self.update_view_matrix();
  }

  pub fn update_view_matrix(&mut self) {
    self.view = Camera::view(self.position, self.rotation, self.camera_type, self.flip_y);
    self.view_pos = Math::vec4_mul(
      [self.position[0], self.position[1], self.position[2], 0.0],
      [-1.0, 1.0, -1.0, 1.0],
    );

    self.updated = true;
  }

  pub fn zoom(&mut self, offset: f32) {
    match self.camera_type {
      CameraType::ThirdPerson => {
        let front = Camera::camera_front(self.rotation);
        let zoom_speed = -offset;

        self.offset = Math::vec3_add(self.offset, Math::vec3_mul(front, [zoom_speed; 3]));

        self.update_view_matrix();
      }
      _ => {}
    }
  }

  // Rotate camera by degrees along the (x, y, z) axis
  pub fn rotate_by_degrees(&mut self, delta: [f32; 3]) {
    match self.camera_type {
      CameraType::Fly | CameraType::FirstPerson => {
        self.rotation = Math::vec3_add(self.rotation, delta);

        let angle_limit = 85.0;

        if self.rotation[0] > 180.0 + angle_limit {
          self.rotation[0] = 180.0 + angle_limit;
        }
        if self.rotation[0] < 180.0 - angle_limit {
          self.rotation[0] = 180.0 - angle_limit;
        }
        self.update_view_matrix();
      }
      _ => {}
    }
  }

  pub fn update_aspect_ratio(&mut self, aspect: f32) {
    self.perspective = Math::perspective(self.fov, aspect, self.znear, self.zfar, self.flip_y);
    self.update_view_matrix();
  }

  pub fn view(
    position: [f32; 3],
    rotation: [f32; 3],
    camera_type: CameraType,
    flip_y: bool,
  ) -> [f32; 16] {
    let mut rot_m = Math::mat4_identity();

    rot_m = Math::mat4_axis_rotate(
      rot_m,
      (rotation[0] * if flip_y { -1.0 } else { 1.0 }).to_radians(),
      [1.0, 0.0, 0.0],
    );
    rot_m = Math::mat4_axis_rotate(rot_m, (rotation[1]).to_radians(), [0.0, 1.0, 0.0]);
    rot_m = Math::mat4_axis_rotate(rot_m, (rotation[2]).to_radians(), [0.0, 0.0, 1.0]);

    let mut translation = position;
    if flip_y {
      translation[1] *= -1.0;
    }

    let trans_m = Math::mat4_translate_vec3(Math::mat4_identity(), translation);

    match camera_type {
      CameraType::FirstPerson | CameraType::Fly | CameraType::ThirdPerson => {
        // rot_m * trans_m
        Math::mat4_mul(trans_m, rot_m)
      }
      CameraType::LookAt => {
        // trans_m * rot_m
        Math::mat4_mul(rot_m, trans_m)
      }
    }
  }

  fn camera_front(rotation: [f32; 3]) -> [f32; 3] {
    let mut cam_front = [0.0, 0.0, 0.0];
    cam_front[0] = -(rotation[0].to_radians()).cos() * (rotation[1].to_radians()).sin();
    cam_front[1] = (rotation[0].to_radians()).sin();
    cam_front[2] = (rotation[0].to_radians()).cos() * (rotation[1].to_radians()).cos();

    Math::vec3_normalise(cam_front)
  }
}
