use cgmath::{Deg, Rad};
use cgmath::{Vector2, Vector3, Vector4, Matrix4,
             InnerSpace, Quaternion, Angle, Zero, Euler};

use std::f64::consts::PI;

pub fn is_point_inside_AABB(point: Vector3<f32>, box_location: Vector3<f32>, box_size: Vector3<f32>) -> bool {
  let min_x = box_location.x - box_size.x*0.5;
  let max_x = box_location.x + box_size.x*0.5;
  let min_y = box_location.y - box_size.y*0.5;
  let max_y = box_location.y + box_size.y*0.5;
  let min_z = box_location.z - box_size.z*0.5;
  let max_z = box_location.z + box_size.z*0.5;
  
  (point.x >= min_x && point.x <= max_x) && 
  (point.y >= min_y && point.y <= max_y) && 
  (point.z >= min_z && point.z <= max_z)
}

pub fn intersect_AABB(box_a_location: Vector3<f32>, box_a_size: Vector3<f32>, box_b_location: Vector3<f32>, box_b_size: Vector3<f32>) -> bool {
  let a_min_x = box_a_location.x - box_a_size.x*0.5;
  let a_max_x = box_a_location.x + box_a_size.x*0.5;
  let a_min_y = box_a_location.y - box_a_size.y*0.5;
  let a_max_y = box_a_location.y + box_a_size.y*0.5;
  let a_min_z = box_a_location.z - box_a_size.z*0.5;
  let a_max_z = box_a_location.z + box_a_size.z*0.5;
  
  let b_min_x = box_b_location.x - box_b_size.x*0.5;
  let b_max_x = box_b_location.x + box_b_size.x*0.5;
  let b_min_y = box_b_location.y - box_b_size.y*0.5;
  let b_max_y = box_b_location.y + box_b_size.y*0.5;
  let b_min_z = box_b_location.z - box_b_size.z*0.5;
  let b_max_z = box_b_location.z + box_b_size.z*0.5;
  
  (a_min_x <= b_max_x && a_max_x >= b_min_x) &&
  (a_min_y <= b_max_y && a_max_y >= b_min_y) &&
  (a_min_z <= b_max_z && a_max_z >= b_min_z)
}

pub fn is_point_inside_sphere(point: Vector3<f32>, sphere: Vector4<f32>) -> bool {
  let dist = ((point.x - sphere.x) * (point.x - sphere.x) +
              (point.y - sphere.y) * (point.y - sphere.y) +
              (point.z - sphere.z) * (point.z - sphere.z)).sqrt();
  
  dist < sphere.w // dist < radius
}

pub fn intersect_sphere(sphere_a: Vector4<f32>, sphere_b: Vector4<f32>) -> bool {
  let dist = ((sphere_a.x - sphere_b.x) * (sphere_a.x - sphere_b.x) +
              (sphere_a.y - sphere_b.y) * (sphere_a.y - sphere_b.y) +
              (sphere_a.z - sphere_b.z) * (sphere_a.z - sphere_b.z)).sqrt();
  
  dist < sphere_a.w + sphere_b.w // dist < a_radius+b_radius
}

pub fn sphere_intersect_AABB(sphere: Vector4<f32>, box_location: Vector3<f32>, box_size: Vector3<f32>) -> bool {
  let min_x = box_location.x - box_size.x*0.5;
  let max_x = box_location.x + box_size.x*0.5;
  let min_y = box_location.y - box_size.y*0.5;
  let max_y = box_location.y + box_size.y*0.5;
  let min_z = box_location.z - box_size.z*0.5;
  let max_z = box_location.z + box_size.z*0.5;
  
  let x = ((sphere.x).min(max_x)).max(min_x);
  let y = ((sphere.y).min(max_y)).max(min_y);
  let z = ((sphere.z).min(max_z)).max(min_z);
  
  let dist = ((x - sphere.x) * (x - sphere.x) +
              (y - sphere.y) * (y - sphere.y) +
              (z - sphere.z) * (z - sphere.z)).sqrt();
  
  dist < sphere.w
}

pub fn is_point_inside_circle(point: Vector2<f32>, sphere: Vector2<f32>, radius: f32) -> bool {
  let dist = ((point.x - sphere.x) * (point.x - sphere.x) +
              (point.y - sphere.y) * (point.y - sphere.y)).sqrt();
  
  dist < radius // dist < radius
}

pub fn intersect_circle(sphere_a: Vector2<f32>, radius_a: f32, sphere_b: Vector2<f32>, radius_b: f32) -> bool {
  let dist = ((sphere_a.x - sphere_b.x) * (sphere_a.x - sphere_b.x) +
              (sphere_a.y - sphere_b.y) * (sphere_a.y - sphere_b.y)).sqrt();
  
  dist < radius_a + radius_b // dist < a_radius+b_radius
}

pub fn intersect_square(box_a_location: Vector2<f32>, box_a_size: Vector2<f32>, box_b_location: Vector2<f32>, box_b_size: Vector2<f32>) -> bool {
  let a_min_x = box_a_location.x - box_a_size.x*0.5;
  let a_max_x = box_a_location.x + box_a_size.x*0.5;
  let a_min_y = box_a_location.y - box_a_size.y*0.5;
  let a_max_y = box_a_location.y + box_a_size.y*0.5;
  
  let b_min_x = box_b_location.x - box_b_size.x*0.5;
  let b_max_x = box_b_location.x + box_b_size.x*0.5;
  let b_min_y = box_b_location.y - box_b_size.y*0.5;
  let b_max_y = box_b_location.y + box_b_size.y*0.5;
  
  (a_min_x <= b_max_x && a_max_x >= b_min_x) &&
  (a_min_y <= b_max_y && a_max_y >= b_min_y)
}

pub fn circle_intersect_square(sphere: Vector2<f32>, radius: f32, box_location: Vector2<f32>, box_size: Vector2<f32>) -> bool {
  let min_x = box_location.x - box_size.x*0.5;
  let max_x = box_location.x + box_size.x*0.5;
  let min_y = box_location.y - box_size.y*0.5;
  let max_y = box_location.y + box_size.y*0.5;
  
  let x = ((sphere.x).min(max_x)).max(min_x);
  let y = ((sphere.y).min(max_y)).max(min_y);
  
  let dist = ((x - sphere.x) * (x - sphere.x) +
              (y - sphere.y) * (y - sphere.y)).sqrt();
  
  dist < radius
}

pub fn line_intersect_square(point_a: Vector2<f32>, point_b: Vector2<f32>, box_location: Vector2<f32>, box_size: Vector2<f32>) -> bool {
  let min_x = box_location.x - box_size.x*0.5;
  let max_x = box_location.x + box_size.x*0.5;
  let min_y = box_location.y - box_size.y*0.5;
  let max_y = box_location.y + box_size.y*0.5;
  
  let line_min_x = point_a.x.min(point_b.x);
  let line_max_x = point_a.x.max(point_b.x);
  let line_min_y = point_a.y.min(point_b.y);
  let line_max_y = point_a.y.max(point_b.y);
  
  if min_x > line_max_x || max_x < line_min_x {
    return false;
  }
  
  if max_y < line_min_y || min_y > line_max_y {
    return false;
  }
  
  // if box not axis aligned
  // let y_at_rect_min_x = line.calculate_y_for_x(box.left)
  // let y_at_rect_max_x = line.calculate_y_for_x(box.right)
  //if min_x > yatrectleft || max_x < yatrectright {
  //  return false;
  //}
  
  //if max_y < yatrectleft || min_y > yatrectright {
  //  return false;
  //}
  
  true
}

pub trait Vector2Math<T> {
  fn abs(&self) -> Vector2<T>;
  fn floor(&self) -> Vector2<T>;
  fn multiply(&self, other: Vector2<T>) -> Vector2<T>;
}

impl Vector2Math<f32> for Vector2<f32> {
  fn abs(&self) -> Vector2<f32> {
    Vector2::new(self.x.abs(), self.y.abs())
  }
  
  fn floor(&self) -> Vector2<f32> {
    Vector2::new(self.x.floor(), self.y.floor())
  }
  
  fn multiply(&self, other: Vector2<f32>) -> Vector2<f32> {
    Vector2::new(self.x*other.x, self.y*other.y)
  }
}

pub trait Vector3Math<T> {
  fn abs(&self) -> Vector3<T>;
  fn floor(&self) -> Vector3<T>;
  fn multiply(&self, other: Vector3<T>) -> Vector3<T>;
}

impl Vector3Math<f32> for Vector3<f32> {
  fn abs(&self) -> Vector3<f32> {
    Vector3::new(self.x.abs(), self.y.abs(), self.z.abs())
  }
  
  fn floor(&self) -> Vector3<f32> {
    Vector3::new(self.x.floor(), self.y.floor(), self.z.floor())
  }
  
  fn multiply(&self, other: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(self.x*other.x, self.y*other.y, self.z*other.z)
  }
}

pub fn calculate_texture_model(translation: Vector3<f32>, size: Vector2<f32>, rotation: f32) -> Matrix4<f32> {
  let axis_z = Vector3::new(0.0, 0.0, 1.0).normalize();
  let rotation: Matrix4<f32> = Matrix4::from_axis_angle(axis_z, Deg(450.0-rotation));
  
  let mut model = Matrix4::from_translation(translation)*rotation;
  model = model * Matrix4::from_nonuniform_scale(size.x, size.y, 1.0);
  model
}

pub fn barryCentric(p1: Vector3<f32>, p2: Vector3<f32>, p3: Vector3<f32>, pos: Vector2<f32>) -> f32 {
  let det = (p2.z - p3.z) * (p1.x - p3.x) + (p3.x - p2.x) * (p1.z - p3.z);
  let l1 = ((p2.z - p3.z) * (pos.x - p3.x) + (p3.x - p2.x) * (pos.y - p3.z)) / det;
  let l2 = ((p3.z - p1.z) * (pos.x - p3.x) + (p1.x - p3.x) * (pos.y - p3.z)) / det;
  let l3 = 1.0 - l1 - l2;
  
  l1 * p1.y + l2 * p2.y + l3 * p3.y
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

pub fn vec3_lerp(current: Vector3<f32>, goal: Vector3<f32>, t: f32) -> Vector3<f32> {
  
  let x = lerp(current.x, goal.x, t);
  let y = lerp(current.y, goal.y, t);
  let z = lerp(current.z, goal.z, t);
  
  Vector3::new(x, y, z)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + t * (b-a)
}

// Cause fucking cgmath gives nan at 0.0, 0.0 situation
pub fn normalise_vector2(mut some_vec: Vector2<f32>) -> Vector2<f32> {
  if some_vec == Vector2::new(0.0, 0.0) || some_vec.x.is_nan() || some_vec.y.is_nan() || some_vec.x.is_infinite() || some_vec.y.is_infinite() {
    return Vector2::new(0.0,0.0);
  }
  
  some_vec = some_vec.normalize();
  some_vec
}

pub fn normalise_vector3(mut some_vec: Vector3<f32>) -> Vector3<f32> {
  if some_vec == Vector3::new(0.0, 0.0, 0.0) || some_vec.x.is_nan() || some_vec.y.is_nan() || some_vec.z.is_nan() || some_vec.x.is_infinite() || some_vec.y.is_infinite() || some_vec.z.is_infinite() {
    return Vector3::new(0.0, 0.0, 0.0);
  }
  
  some_vec = some_vec.normalize();
  some_vec
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
  radian * 180.0 / PI as f32
}

pub fn squared_distance(origin: Vector2<f32>, point: Vector2<f32>) -> f32 {
  unsquared_distance(origin, point).sqrt()
}

pub fn unsquared_distance(origin: Vector2<f32>, point: Vector2<f32>) -> f32 {
  (origin.x-point.x)*(origin.x-point.x) + (origin.y-point.y)*(origin.y-point.y)
}

pub fn rotate_vector2(direction: Vector2<f32>, angle: f32) -> Vector2<f32> {
  let radians = to_radians(angle);
  
  let cos = radians.cos();
  let sin = radians.sin();
  
  Vector2::new(direction.x*cos - direction.y*sin, direction.x*sin + direction.y*cos)
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

// line Vector4(x, y, x2, y2), rect Vector4(center_x, center_y, width, height)
pub fn line_rect_collision(line: Vector4<f32>, rect: Vector4<f32>) -> bool {
  let x = rect.x-rect.z*0.5;
  let y = rect.y-rect.w*0.5;
  let width = rect.z;
  let height = rect.w;
  
  let left = line_line_collision(line, Vector4::new(x, y, x, y+height));
  let right = line_line_collision(line, Vector4::new(x+width, y, x+width, y+height));
  let bottom = line_line_collision(line, Vector4::new(x, y, x+width, y));
  let top = line_line_collision(line, Vector4::new(x, y+height, x+width, y+height));
  
  if left.is_some() || right.is_some() || bottom.is_some() || top.is_some() {
    return true
  }
  
  false
}

pub fn line_line_collision(line_1: Vector4<f32>, line_2: Vector4<f32>) -> Option<Vector2<f32>> {
  let x1 = line_1.x;
  let y1 = line_1.y;
  let x2 = line_1.z;
  let y2 = line_1.w;
  let x3 = line_2.x;
  let y3 = line_2.y;
  let x4 = line_2.z;
  let y4 = line_2.w;
  
  // direction of lines
  let u_a = ((x4-x3)*(y1-y3) - (y4-y3)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));
  let u_b = ((x2-x1)*(y1-y3) - (y2-y1)*(x1-x3)) / ((y4-y3)*(x2-x1) - (x4-x3)*(y2-y1));
  
  if u_a >= 0.0 && u_a <= 1.0 && u_b >= 0.0 && u_b <= 1.0 {
    let intersection_x = x1 + (u_a * (x2-x1));
    let intersection_y = y1 + (u_a * (y2-y1));
    
    return Some(Vector2::new(intersection_x, intersection_y))
  }
  
  None
}


pub fn quaternion_from_rotation(axis: Vector3<f32>, angle: f32) -> Vector4<f32> {
  let mut q = Vector4::zero();
  
  let h_angle = (angle*0.5) * PI as f32 / 180.0;
  q.x = axis.x * h_angle.sin();
  q.y = axis.y * h_angle.sin();
  q.z = axis.z * h_angle.sin();
  q.w = h_angle.cos();
  
  q
}

pub fn rotate_vertex_with_quaternion(vertex: Vector3<f32>, axis: Vector3<f32>, angle: f32) -> Vector3<f32> {
  let q = quaternion_from_rotation(axis, angle);
  let v = vertex.xyz();
  vertex + 2.0 * (vertex * q.w + vertex.cross(q.xyz())).cross(q.xyz())
}

pub fn rotate_vertex_by_angle(vertex: Vector3<f32>, angle: Vector3<f32>) -> Vector3<f32> {
  // rotation should be in degrees
  
  let mut rotated_vertex = rotate_vertex_with_quaternion(vertex, Vector3::new(1.0, 0.0, 0.0), angle.x);
  rotated_vertex = rotate_vertex_with_quaternion(rotated_vertex, Vector3::new(0.0, 1.0, 0.0), angle.y);
  rotated_vertex = rotate_vertex_with_quaternion(rotated_vertex, Vector3::new(0.0, 0.0, 1.0), angle.z);
  
  rotated_vertex
}

// scale 0.866025404
// rot_x 0.577350269
// rot y 
// rot z 
// angle 120

pub fn quaternion_to_axis_angle(q: Vector4<f32>) -> Vector3<f32> {
  let sig = 0.499;
  
  let unit = q.x + q.z + q.y + q.w;
  let test = q.x*q.z + q.y*q.w;
  
  let scale = (q.x*q.x + q.y*q.y + q.z*q.z).sqrt();
    
  if test > sig * unit {
    Vector3::new(0.0, 90.0, q.x.atan2(q.w) *2.0)
  } else if test < -sig * unit {
    Vector3::new(0.0, -90.0, -q.x.atan2(q.w) *2.0)
  } else {
    Vector3::new(
       (2.0 * (-q.y * q.z - q.x * q.w) as f32).atan2(1.0 - 2.0 * (q.x*q.x + q.y*q.y)) *0.5,
       (2.0 * (q.x * q.z + q.y * q.w)).asin() * 0.5,
       (2.0 * (-q.x * q.y + q.z * q.w) as f32).atan2(1.0 - 2.0 * (q.y*q.y + q.z*q.z)) *0.5,
    )
  }
  
 /* let scale = (q.x*q.x + q.y*q.y + q.z*q.z).sqrt();
  
  let angle = q.w.acos() * 2.0;
  let rotation = Vector3::new(q.x / scale, q.y /scale, q.z / scale);
  
  Vector3::new(angle*rotation.x, angle*rotation.y, angle*rotation.z)*/
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
