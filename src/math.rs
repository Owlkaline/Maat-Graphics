use cgmath::Deg;
use cgmath::Vector2;
use cgmath::Vector3;
use cgmath::Vector4;
use cgmath::Matrix4;
use cgmath::InnerSpace;

use std::f64::consts::PI;

pub fn calculate_texture_model(translation: Vector3<f32>, size: Vector2<f32>, rotation: f32) -> Matrix4<f32> {
  let axis_z = Vector3::new(0.0, 0.0, 1.0).normalize();
  let rotation: Matrix4<f32> = Matrix4::from_axis_angle(axis_z, Deg(450.0-rotation));
  
  let mut model = Matrix4::from_translation(translation)*rotation;
  model = model * Matrix4::from_nonuniform_scale(size.x, size.y, 1.0);
  model
}

pub fn calculate_y_rotation(y_rotation: f32) -> (f32, f32) {
  let x_rot;
  let z_rot;
  
  let q1 = 90.0;
  let q2 = 180.0;
  let q3 = 270.0;
  
  let mut angle_y = y_rotation;
  
  if angle_y < 0.0 {
    angle_y = 360.0 + angle_y;
  }
  
  if angle_y > 360.0 {
    angle_y = angle_y - 360.0;
  }
  
  if angle_y < q1 {
    z_rot = 1.0 - (angle_y/90.0);
    x_rot = angle_y/90.0;
  } else if angle_y < q2 {
    angle_y -= q1;
    z_rot = -(angle_y/90.0);
    x_rot = 1.0-(angle_y/90.0);
  } else if angle_y < q3 {
    angle_y -= q2;
    z_rot = (angle_y/90.0) - 1.0;
    x_rot = -(angle_y/90.0);
  } else {
    angle_y -= q3;
    z_rot = angle_y/90.0;
    x_rot = angle_y/90.0 - 1.0;
  }
  
  (x_rot, z_rot)
}

pub fn array2_to_vec2<T: Clone>(array: [T; 2]) -> Vector2<T> {
  Vector2::new(array[0].clone(), array[1].clone())
}

pub fn array3_to_vec3<T: Clone>(array: [T; 3]) -> Vector3<T> {
  Vector3::new(array[0].clone(), array[1].clone(), array[2].clone())
}

pub fn array4_to_vec4<T: Clone>(array: [T; 4]) -> Vector4<T> {
  Vector4::new(array[0].clone(), array[1].clone(), array[2].clone(), array[3].clone())
}

pub fn to_radians(degree: f32) -> f32 {
  degree * (PI as f32/180.0)
}

pub fn to_degrees(radian: f32) -> f32 {
  radian *  180.0 / PI as f32
}

pub fn squared_distance(origin: Vector2<f32>, point: Vector2<f32>) -> f32 {
  unsquared_distance(origin, point).sqrt()
}

pub fn unsquared_distance(origin: Vector2<f32>, point: Vector2<f32>) -> f32 {
  (origin.x-point.x)*(origin.x-point.x) + (origin.y-point.y)*(origin.y-point.y)
}

pub fn aabb_circle_collision(circle: Vector3<f32>, square: Vector4<f32>, inner_radius: f32, outer_radius: f32) -> bool {
  let dist_between_centers = squared_distance(Vector2::new(square.x, square.y), Vector2::new(circle.x, circle.y));
  if dist_between_centers > (outer_radius+circle.z)*(outer_radius+circle.z) {
    return false;
  }
  if dist_between_centers > (inner_radius+circle.z)*(inner_radius+circle.z) {
    return true;
  }
  
  let dist = squared_distance(Vector2::new(circle.x, circle.y), Vector2::new(square.x, square.y));
  let c1c2vec = Vector2::new((circle.x-square.x)/dist,  (circle.y-square.y)/dist);
  box_collision(Vector4::new(c1c2vec.x, c1c2vec.y, 1.0, 1.0), square)
}

pub fn intersection(center: Vector2<f32>, radius: f32, p1: Vector2<f32>, p2: Vector2<f32>) -> ((f32, f32), (f32, f32)) {
  let mut dx = p2.x - p1.x;
  let mut dy = p2.y - p1.y;
  let mut radius = radius;
  if dx < 1.0 && dx > -1.0 {
    if dx <= 0.0 {
      dx = -1.0;
    } else {
      dx = 1.0;
    }
  }
  
  if dy < 1.0 && dy > -1.0 {
    if dy <= 0.0 {
      dy = -1.0;
    } else {
      dy = 1.0;
    }
  }
  
  let a = dx*dx + dy*dy;
  let b = 2.0* (dx * (p1.x - center.x) + dy * (p1.y - center.y));
  let mut c = (p1.x - center.x)*(p1.x - center.x) + (p1.y - center.y)*(p1.y - center.y) - radius*radius;
  
  let mut discriminit = b*b - 4.0*a*c;
  if discriminit < 0.0 {
    radius *= 2.0;
    c = (p1.x - center.x)*(p1.x - center.x) + (p1.y - center.y)*(p1.y - center.y) - radius*radius;
    discriminit = b*b - 4.0*a*c;
  }
  
  let t1 = (-b + discriminit.sqrt()) / (2.0 * a);
  let t2 = (-b - discriminit.sqrt()) / (2.0 * a);
  
  ((dx * t1 + p1.x, dy * t1 + p1.y), (dx * t2 + p1.x, dy* t2 + p1.y))
}

/// Simple collision between two cicles given
/// a Vector3(center_x, center_y, raidus)
///
/// # Examples
/// 
/// Simple example with circles that do collide.
///
/// ```
/// # extern crate maat_graphics;
/// # extern crate cgmath;
/// let a = cgmath::Vector3::new(1.0, 1.0, 5.0);
/// let b = cgmath::Vector3::new(-1.0, -1.0, 4.0);
/// assert!(maat_graphics::math::circle_collision(a, b));
/// ```
///
/// Simple eample with circle that dont collide.
/// 
/// ```
/// # extern crate maat_graphics;
/// # extern crate cgmath;
/// let a = cgmath::Vector3::new(10.0, 10.0, 5.0);
/// let b = cgmath::Vector3::new(-10.0, -10.0, 4.0);
/// assert!(!maat_graphics::math::circle_collision(a, b));
/// ```
/// 
pub fn circle_collision(a: Vector3<f32>, b: Vector3<f32>) -> bool {
  let dist = a.z + b.z;
  let dx = b.x - a.x;
  let dy = b.y - a.y;
  
  if dx*dx + dy*dy < dist*dist {
    return true
  }
  
  false
}

/// Simple collision between two box's given
/// a Vector4(center_x, center_y, width, height)
///
/// # Examples
/// 
/// Simple example with box's that do collide.
///
/// ```
/// # extern crate maat_graphics;
/// # extern crate cgmath;
/// let a = cgmath::Vector4::new(1.0, 1.0, 5.0, 5.0);
/// let b = cgmath::Vector4::new(-1.0, -1.0, 4.0, 4.0);
/// assert!(maat_graphics::math::box_collision(a, b));
/// ```
///
/// Simple eample with circle that dont collide.
/// 
/// ```
/// # extern crate maat_graphics;
/// # extern crate cgmath;
/// let a = cgmath::Vector4::new(10.0, 10.0, 5.0, 5.0);
/// let b = cgmath::Vector4::new(-10.0, -10.0, 4.0, 4.0);
/// assert!(!maat_graphics::math::box_collision(a, b));
/// ```
/// 
pub fn box_collision(a: Vector4<f32>, b: Vector4<f32>) -> bool {
  if a.x+a.z*0.5 < b.x-b.z*0.5 || a.x-a.z*0.5 > b.x+b.z*0.5 {
    return false
  }
  if a.y+a.w*0.5 < b.y-b.w*0.5 || a.y-a.w*0.5 > b.y+b.w*0.5 {
    return false
  }
  true
}

pub fn min(a: f32, b: f32) -> f32 {
  a.min(b)
}

pub fn max(a: f32, b: f32) -> f32 {
  a.max(b)
}
