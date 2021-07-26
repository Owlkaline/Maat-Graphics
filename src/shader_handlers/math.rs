use std::ops::*;

pub struct Math {}

pub trait VectorMath {
  fn dot(self, other: Self) -> f32;
  fn scale(self, other: f32) -> Self;
  fn set_magnitude(self, magnitude: f32) -> Self;
  fn mix(self, other: Self, a: f32) -> Self;
  fn normalise(self) -> Self;

  fn magnitude(&self) -> f32;
  fn squared_magnitude(&self) -> f32;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector2 {
  pub x: f32,
  pub y: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector3 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector4 {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}

impl Vector2 {
  pub fn new(x: f32, y: f32) -> Vector2 {
    Vector2 {
      x,
      y,
    }
  }

  pub fn from_f32(v: f32) -> Vector2 {
    Vector2 {
      x: v,
      y: v,
    }
  }

  pub fn from_array(a: [f32; 2]) -> Vector2 {
    Vector2 {
      x: a[0],
      y: a[1],
    }
  }
}

impl Add for Vector2 {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }
}

impl Add<f32> for Vector2 {
  type Output = Self;

  fn add(self, other: f32) -> Self {
    Self {
      x: self.x + other,
      y: self.y + other,
    }
  }
}

impl Add<[f32; 2]> for Vector2 {
  type Output = Self;

  fn add(self, other: [f32; 2]) -> Self {
    Self {
      x: self.x + other[0],
      y: self.y + other[1],
    }
  }
}

impl Sub for Vector2 {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    Self {
      x: self.x - other.x,
      y: self.y - other.y,
    }
  }
}

impl Sub<f32> for Vector2 {
  type Output = Self;

  fn sub(self, other: f32) -> Self {
    Self {
      x: self.x - other,
      y: self.y - other,
    }
  }
}

impl Sub<[f32; 2]> for Vector2 {
  type Output = Self;

  fn sub(self, other: [f32; 2]) -> Self {
    Self {
      x: self.x - other[0],
      y: self.y - other[1],
    }
  }
}

impl AddAssign for Vector2 {
  fn add_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x + other.x,
      y: self.y + other.y,
    };
  }
}

impl AddAssign<f32> for Vector2 {
  fn add_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x + other,
      y: self.y + other,
    };
  }
}

impl AddAssign<[f32; 2]> for Vector2 {
  fn add_assign(&mut self, other: [f32; 2]) {
    *self = Self {
      x: self.x + other[0],
      y: self.y + other[1],
    };
  }
}

impl SubAssign for Vector2 {
  fn sub_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x - other.x,
      y: self.y - other.y,
    };
  }
}

impl SubAssign<f32> for Vector2 {
  fn sub_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x - other,
      y: self.y - other,
    };
  }
}

impl SubAssign<[f32; 2]> for Vector2 {
  fn sub_assign(&mut self, other: [f32; 2]) {
    *self = Self {
      x: self.x - other[0],
      y: self.y - other[1],
    };
  }
}

impl Mul for Vector2 {
  type Output = Self;

  fn mul(self, other: Self) -> Self {
    Self {
      x: self.x * other.x,
      y: self.y * other.y,
    }
  }
}

impl Mul<f32> for Vector2 {
  type Output = Self;

  fn mul(self, other: f32) -> Self {
    Self {
      x: self.x * other,
      y: self.y * other,
    }
  }
}

impl Mul<[f32; 2]> for Vector2 {
  type Output = Self;

  fn mul(self, other: [f32; 2]) -> Self {
    Self {
      x: self.x * other[0],
      y: self.y * other[1],
    }
  }
}

impl Div for Vector2 {
  type Output = Self;

  fn div(self, other: Self) -> Self {
    Self {
      x: self.x / other.x,
      y: self.y / other.y,
    }
  }
}

impl Div<f32> for Vector2 {
  type Output = Self;

  fn div(self, other: f32) -> Self {
    Self {
      x: self.x / other,
      y: self.y / other,
    }
  }
}

impl Div<[f32; 2]> for Vector2 {
  type Output = Self;

  fn div(self, other: [f32; 2]) -> Self {
    Self {
      x: self.x / other[0],
      y: self.y / other[1],
    }
  }
}

impl MulAssign for Vector2 {
  fn mul_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x * other.x,
      y: self.y * other.y,
    };
  }
}

impl MulAssign<f32> for Vector2 {
  fn mul_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x * other,
      y: self.y * other,
    };
  }
}

impl MulAssign<[f32; 2]> for Vector2 {
  fn mul_assign(&mut self, other: [f32; 2]) {
    *self = Self {
      x: self.x * other[0],
      y: self.y * other[1],
    };
  }
}

impl DivAssign for Vector2 {
  fn div_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x / other.x,
      y: self.y / other.y,
    };
  }
}

impl DivAssign<f32> for Vector2 {
  fn div_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x / other,
      y: self.y / other,
    };
  }
}

impl DivAssign<[f32; 2]> for Vector2 {
  fn div_assign(&mut self, other: [f32; 2]) {
    *self = Self {
      x: self.x / other[0],
      y: self.y / other[1],
    };
  }
}

impl Neg for Vector2 {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self {
      x: -self.x,
      y: -self.y,
    }
  }
}

impl Into<[f32; 2]> for Vector2 {
  fn into(self) -> [f32; 2] {
    [self.x, self.y]
  }
}

impl Into<[f32; 2]> for &Vector2 {
  fn into(self) -> [f32; 2] {
    [self.x, self.y]
  }
}

impl Vector3 {
  pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3 { x, y, z }
  }

  pub fn from_f32(v: f32) -> Vector3 {
    Vector3 { x: v, y: v, z: v }
  }

  pub fn from_array(a: [f32; 3]) -> Vector3 {
    Vector3 {
      x: a[0],
      y: a[1],
      z: a[2],
    }
  }

  pub fn cross(self, other: Vector3) -> Vector3 {
    Vector3 {
      x: self.y * other.z - other.y * self.z,
      y: self.z * other.x - other.z * self.x,
      z: self.x * other.y - other.x * self.y,
    }
  }
}

impl VectorMath for Vector3 {
  fn dot(self, other: Self) -> f32 {
    let m = self * other;

    m.x + m.y + m.z
  }

  fn scale(self, other: f32) -> Self {
    self * other
  }

  fn set_magnitude(self, magnitude: f32) -> Self {
    let len = self.magnitude();

    Self {
      x: self.x * magnitude / len,
      y: self.y * magnitude / len,
      z: self.z * magnitude / len,
    }
  }

  fn mix(self, other: Self, a: f32) -> Self {
    Self {
      x: self.x * (1.0 - a) + other.x * a,
      y: self.y * (1.0 - a) + other.y * a,
      z: self.z * (1.0 - a) + other.z * a,
    }
  }

  fn normalise(self) -> Self {
    let mag = self.magnitude();

    self / mag
  }

  fn squared_magnitude(&self) -> f32 {
    self.x * self.x + self.y * self.y + self.z * self.z
  }

  fn magnitude(&self) -> f32 {
    self.squared_magnitude().sqrt()
  }
}

impl Add for Vector3 {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
      z: self.z + other.z,
    }
  }
}

impl Add<f32> for Vector3 {
  type Output = Self;

  fn add(self, other: f32) -> Self {
    Self {
      x: self.x + other,
      y: self.y + other,
      z: self.z + other,
    }
  }
}

impl Add<[f32; 3]> for Vector3 {
  type Output = Self;

  fn add(self, other: [f32; 3]) -> Self {
    Self {
      x: self.x + other[0],
      y: self.y + other[1],
      z: self.z + other[2],
    }
  }
}

impl Sub for Vector3 {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    Self {
      x: self.x - other.x,
      y: self.y - other.y,
      z: self.z - other.z,
    }
  }
}

impl Sub<f32> for Vector3 {
  type Output = Self;

  fn sub(self, other: f32) -> Self {
    Self {
      x: self.x - other,
      y: self.y - other,
      z: self.z - other,
    }
  }
}

impl Sub<[f32; 3]> for Vector3 {
  type Output = Self;

  fn sub(self, other: [f32; 3]) -> Self {
    Self {
      x: self.x - other[0],
      y: self.y - other[1],
      z: self.z - other[2],
    }
  }
}

impl AddAssign for Vector3 {
  fn add_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x + other.x,
      y: self.y + other.y,
      z: self.z + other.z,
    };
  }
}

impl AddAssign<f32> for Vector3 {
  fn add_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x + other,
      y: self.y + other,
      z: self.z + other,
    };
  }
}

impl AddAssign<[f32; 3]> for Vector3 {
  fn add_assign(&mut self, other: [f32; 3]) {
    *self = Self {
      x: self.x + other[0],
      y: self.y + other[1],
      z: self.z + other[2],
    };
  }
}

impl SubAssign for Vector3 {
  fn sub_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x - other.x,
      y: self.y - other.y,
      z: self.z - other.z,
    };
  }
}

impl SubAssign<f32> for Vector3 {
  fn sub_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x - other,
      y: self.y - other,
      z: self.z - other,
    };
  }
}

impl SubAssign<[f32; 3]> for Vector3 {
  fn sub_assign(&mut self, other: [f32; 3]) {
    *self = Self {
      x: self.x - other[0],
      y: self.y - other[1],
      z: self.z - other[2],
    };
  }
}

impl Mul for Vector3 {
  type Output = Self;

  fn mul(self, other: Self) -> Self {
    Self {
      x: self.x * other.x,
      y: self.y * other.y,
      z: self.z * other.z,
    }
  }
}

impl Mul<f32> for Vector3 {
  type Output = Self;

  fn mul(self, other: f32) -> Self {
    Self {
      x: self.x * other,
      y: self.y * other,
      z: self.z * other,
    }
  }
}

impl Mul<[f32; 3]> for Vector3 {
  type Output = Self;

  fn mul(self, other: [f32; 3]) -> Self {
    Self {
      x: self.x * other[0],
      y: self.y * other[1],
      z: self.z * other[2],
    }
  }
}

impl Div for Vector3 {
  type Output = Self;

  fn div(self, other: Self) -> Self {
    Self {
      x: self.x / other.x,
      y: self.y / other.y,
      z: self.z / other.z,
    }
  }
}

impl Div<f32> for Vector3 {
  type Output = Self;

  fn div(self, other: f32) -> Self {
    Self {
      x: self.x / other,
      y: self.y / other,
      z: self.z / other,
    }
  }
}

impl Div<[f32; 3]> for Vector3 {
  type Output = Self;

  fn div(self, other: [f32; 3]) -> Self {
    Self {
      x: self.x / other[0],
      y: self.y / other[1],
      z: self.z / other[2],
    }
  }
}

impl MulAssign for Vector3 {
  fn mul_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x * other.x,
      y: self.y * other.y,
      z: self.z * other.z,
    };
  }
}

impl MulAssign<f32> for Vector3 {
  fn mul_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x * other,
      y: self.y * other,
      z: self.z * other,
    };
  }
}

impl MulAssign<[f32; 3]> for Vector3 {
  fn mul_assign(&mut self, other: [f32; 3]) {
    *self = Self {
      x: self.x * other[0],
      y: self.y * other[1],
      z: self.z * other[2],
    };
  }
}

impl DivAssign for Vector3 {
  fn div_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x / other.x,
      y: self.y / other.y,
      z: self.z / other.z,
    };
  }
}

impl DivAssign<f32> for Vector3 {
  fn div_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x / other,
      y: self.y / other,
      z: self.z / other,
    };
  }
}

impl DivAssign<[f32; 3]> for Vector3 {
  fn div_assign(&mut self, other: [f32; 3]) {
    *self = Self {
      x: self.x / other[0],
      y: self.y / other[1],
      z: self.z / other[2],
    };
  }
}

impl Neg for Vector3 {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self {
      x: -self.x,
      y: -self.y,
      z: -self.z,
    }
  }
}

impl Into<[f32; 3]> for Vector3 {
  fn into(self) -> [f32; 3] {
    [self.x, self.y, self.z]
  }
}

impl Into<[f32; 3]> for &Vector3 {
  fn into(self) -> [f32; 3] {
    [self.x, self.y, self.z]
  }
}

impl Vector4 {
  pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
    Vector4 { x, y, z, w }
  }

  pub fn from_f32(v: f32) -> Vector4 {
    Vector4 {
      x: v,
      y: v,
      z: v,
      w: v,
    }
  }

  pub fn from_array(a: [f32; 4]) -> Vector4 {
    Vector4 {
      x: a[0],
      y: a[1],
      z: a[2],
      w: a[3],
    }
  }
}

impl VectorMath for Vector4 {
  fn dot(self, other: Self) -> f32 {
    let m = self * other;

    m.x + m.y + m.z + m.w
  }

  fn scale(self, other: f32) -> Self {
    self * other
  }

  fn set_magnitude(self, magnitude: f32) -> Self {
    let len = self.magnitude();

    Self {
      x: self.x * magnitude / len,
      y: self.y * magnitude / len,
      z: self.z * magnitude / len,
      w: self.w * magnitude / len,
    }
  }

  fn mix(self, other: Self, a: f32) -> Self {
    Self {
      x: self.x * (1.0 - a) + other.x * a,
      y: self.y * (1.0 - a) + other.y * a,
      z: self.z * (1.0 - a) + other.z * a,
      w: self.w * (1.0 - a) + other.w * a,
    }
  }

  fn normalise(self) -> Self {
    let mag = self.magnitude();

    self / mag
  }

  fn squared_magnitude(&self) -> f32 {
    self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
  }

  fn magnitude(&self) -> f32 {
    self.squared_magnitude().sqrt()
  }
}

impl Add for Vector4 {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
      z: self.z + other.z,
      w: self.w + other.w,
    }
  }
}

impl Add<f32> for Vector4 {
  type Output = Self;

  fn add(self, other: f32) -> Self {
    Self {
      x: self.x + other,
      y: self.y + other,
      z: self.z + other,
      w: self.w + other,
    }
  }
}

impl Add<[f32; 4]> for Vector4 {
  type Output = Self;

  fn add(self, other: [f32; 4]) -> Self {
    Self {
      x: self.x + other[0],
      y: self.y + other[1],
      z: self.z + other[2],
      w: self.w + other[3],
    }
  }
}

impl Sub for Vector4 {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    Self {
      x: self.x - other.x,
      y: self.y - other.y,
      z: self.z - other.z,
      w: self.w - other.w,
    }
  }
}

impl Sub<f32> for Vector4 {
  type Output = Self;

  fn sub(self, other: f32) -> Self {
    Self {
      x: self.x - other,
      y: self.y - other,
      z: self.z - other,
      w: self.w - other,
    }
  }
}

impl Sub<[f32; 4]> for Vector4 {
  type Output = Self;

  fn sub(self, other: [f32; 4]) -> Self {
    Self {
      x: self.x - other[0],
      y: self.y - other[1],
      z: self.z - other[2],
      w: self.w - other[3],
    }
  }
}

impl AddAssign for Vector4 {
  fn add_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x + other.x,
      y: self.y + other.y,
      z: self.z + other.z,
      w: self.w + other.w,
    };
  }
}

impl AddAssign<f32> for Vector4 {
  fn add_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x + other,
      y: self.y + other,
      z: self.z + other,
      w: self.w + other,
    };
  }
}

impl AddAssign<[f32; 4]> for Vector4 {
  fn add_assign(&mut self, other: [f32; 4]) {
    *self = Self {
      x: self.x + other[0],
      y: self.y + other[1],
      z: self.z + other[2],
      w: self.w + other[3],
    };
  }
}

impl SubAssign for Vector4 {
  fn sub_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x - other.x,
      y: self.y - other.y,
      z: self.z - other.z,
      w: self.w - other.w,
    };
  }
}

impl SubAssign<f32> for Vector4 {
  fn sub_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x - other,
      y: self.y - other,
      z: self.z - other,
      w: self.w - other,
    };
  }
}

impl SubAssign<[f32; 4]> for Vector4 {
  fn sub_assign(&mut self, other: [f32; 4]) {
    *self = Self {
      x: self.x - other[0],
      y: self.y - other[1],
      z: self.z - other[2],
      w: self.w - other[3],
    };
  }
}

impl Mul for Vector4 {
  type Output = Self;

  fn mul(self, other: Self) -> Self {
    Self {
      x: self.x * other.x,
      y: self.y * other.y,
      z: self.z * other.z,
      w: self.w * other.w,
    }
  }
}

impl Mul<f32> for Vector4 {
  type Output = Self;

  fn mul(self, other: f32) -> Self {
    Self {
      x: self.x * other,
      y: self.y * other,
      z: self.z * other,
      w: self.w * other,
    }
  }
}

impl Mul<[f32; 4]> for Vector4 {
  type Output = Self;

  fn mul(self, other: [f32; 4]) -> Self {
    Self {
      x: self.x * other[0],
      y: self.y * other[1],
      z: self.z * other[2],
      w: self.w * other[3],
    }
  }
}

impl Div for Vector4 {
  type Output = Self;

  fn div(self, other: Self) -> Self {
    Self {
      x: self.x / other.x,
      y: self.y / other.y,
      z: self.z / other.z,
      w: self.w / other.w,
    }
  }
}

impl Div<f32> for Vector4 {
  type Output = Self;

  fn div(self, other: f32) -> Self {
    Self {
      x: self.x / other,
      y: self.y / other,
      z: self.z / other,
      w: self.w / other,
    }
  }
}

impl Div<[f32; 4]> for Vector4 {
  type Output = Self;

  fn div(self, other: [f32; 4]) -> Self {
    Self {
      x: self.x / other[0],
      y: self.y / other[1],
      z: self.z / other[2],
      w: self.w / other[3],
    }
  }
}

impl MulAssign for Vector4 {
  fn mul_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x * other.x,
      y: self.y * other.y,
      z: self.z * other.z,
      w: self.w * other.w,
    };
  }
}

impl MulAssign<f32> for Vector4 {
  fn mul_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x * other,
      y: self.y * other,
      z: self.z * other,
      w: self.w * other,
    };
  }
}

impl MulAssign<[f32; 4]> for Vector4 {
  fn mul_assign(&mut self, other: [f32; 4]) {
    *self = Self {
      x: self.x * other[0],
      y: self.y * other[1],
      z: self.z * other[2],
      w: self.w * other[3],
    };
  }
}

impl DivAssign for Vector4 {
  fn div_assign(&mut self, other: Self) {
    *self = Self {
      x: self.x / other.x,
      y: self.y / other.y,
      z: self.z / other.z,
      w: self.w / other.w,
    };
  }
}

impl DivAssign<f32> for Vector4 {
  fn div_assign(&mut self, other: f32) {
    *self = Self {
      x: self.x / other,
      y: self.y / other,
      z: self.z / other,
      w: self.w / other,
    };
  }
}

impl DivAssign<[f32; 4]> for Vector4 {
  fn div_assign(&mut self, other: [f32; 4]) {
    *self = Self {
      x: self.x / other[0],
      y: self.y / other[1],
      z: self.z / other[2],
      w: self.w / other[3],
    };
  }
}

impl Neg for Vector4 {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self {
      x: -self.x,
      y: -self.y,
      z: -self.z,
      w: -self.w,
    }
  }
}

impl Into<[f32; 4]> for Vector4 {
  fn into(self) -> [f32; 4] {
    [self.x, self.y, self.z, self.w]
  }
}

impl Into<[f32; 4]> for &Vector4 {
  fn into(self) -> [f32; 4] {
    [self.x, self.y, self.z, self.w]
  }
}

impl Math {
  pub fn vec3_equals(p: [f32; 3], q: [f32; 3]) -> bool {
    p[0] == q[0] && p[1] == q[1] && p[2] == q[2]
  }

  pub fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
  }

  pub fn vec3_minus(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
  }

  pub fn vec3_mul(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
  }

  pub fn vec3_div(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] / b[0], a[1] / b[1], a[2] / b[2]]
  }

  pub fn vec3_normalise(a: [f32; 3]) -> [f32; 3] {
    let mag = Math::vec3_mag(a);

    [a[0] / mag, a[1] / mag, a[2] / mag]
  }

  pub fn vec3_mul_f32(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
  }

  pub fn vec3_div_f32(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] / s, v[1] / s, v[2] / s]
  }

  pub fn vec3_mix(x: [f32; 3], y: [f32; 3], a: f32) -> [f32; 3] {
    let mut vec = [0.0; 3];

    for i in 0..3 {
      vec[i] = x[i] * (1.0 - a) + y[i] * a;
    }

    vec
  }

  pub fn vec3_from_f32(x: f32) -> [f32; 3] {
    [x, x, x]
  }

  pub fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    let m = Math::vec3_mul(a, b);

    m[0] + m[1] + m[2]
  }

  pub fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
      a[1] * b[2] - b[1] * a[2],
      a[2] * b[0] - b[2] * a[0],
      a[0] * b[1] - b[0] * a[1],
    ]
  }

  pub fn vec3_squared_mag(a: [f32; 3]) -> f32 {
    a[0] * a[0] + a[1] * a[1] + a[2] * a[2]
  }

  pub fn vec3_mag(a: [f32; 3]) -> f32 {
    (Math::vec3_squared_mag(a)).sqrt()
  }

  pub fn vec3_set_mag(a: [f32; 3], l: f32) -> [f32; 3] {
    let mut a = a;

    let len = Math::vec3_mag(a);
    a[0] *= l / len;
    a[1] *= l / len;
    a[2] *= l / len;

    a
  }

  pub fn vec4_equals(p: [f32; 4], q: [f32; 4]) -> bool {
    p[0] == q[0] && p[1] == q[1] && p[2] == q[2] && p[3] == q[3]
  }

  pub fn vec4_add(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
  }

  pub fn vec4_minus(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
  }

  pub fn vec4_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
  }

  pub fn vec4_div(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] / b[0], a[1] / b[1], a[2] / b[2], a[3] / b[3]]
  }

  pub fn vec4_normalise(a: [f32; 4]) -> [f32; 4] {
    let mag = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2] + a[3] * a[3]).sqrt();

    [a[0] / mag, a[1] / mag, a[2] / mag, a[3] / mag]
  }

  pub fn vec4_mul_f32(v: [f32; 4], s: f32) -> [f32; 4] {
    [v[0] * s, v[1] * s, v[2] * s, v[3] * s]
  }

  pub fn vec4_div_f32(v: [f32; 4], s: f32) -> [f32; 4] {
    [v[0] / s, v[1] / s, v[2] / s, v[3] / s]
  }

  pub fn vec4_mix(x: [f32; 4], y: [f32; 4], a: f32) -> [f32; 4] {
    let mut vec = [0.0; 4];

    for i in 0..4 {
      vec[i] = x[i] * (1.0 - a) + y[i] * a;
    }

    vec
  }

  pub fn vec4_from_f32(x: f32) -> [f32; 4] {
    [x, x, x, x]
  }

  pub fn vec4_dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    let m = Math::vec4_mul(a, b);

    m[0] + m[1] + m[2] + m[3]
  }

  pub fn mat3_identity() -> [f32; 9] {
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
  }

  pub fn mat3_from_3_vec3(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 9] {
    let mut m = Math::mat3_identity();

    let r = 3;
    m[r * 0 + 1] = v0[0];
    m[r * 0 + 2] = v0[1];
    m[r * 0 + 3] = v0[2];
    m[r * 1 + 1] = v1[0];
    m[r * 1 + 2] = v1[1];
    m[r * 1 + 3] = v1[2];
    m[r * 2 + 1] = v2[0];
    m[r * 2 + 2] = v2[1];
    m[r * 2 + 3] = v2[2];

    m
  }

  pub fn mat3_into_3_vec3(m: [f32; 9]) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let m0 = [m[0], m[1], m[2]];
    let m1 = [m[3], m[4], m[5]];
    let m2 = [m[6], m[7], m[8]];

    (m0, m1, m2)
  }

  pub fn mat3_add(m: [f32; 9], n: [f32; 9]) -> [f32; 9] {
    let (m0, m1, m2) = Math::mat3_into_3_vec3(m);
    let (n0, n1, n2) = Math::mat3_into_3_vec3(n);

    let mn0 = Math::vec3_add(m0, n0);
    let mn1 = Math::vec3_add(m1, n1);
    let mn2 = Math::vec3_add(m2, n2);

    Math::mat3_from_3_vec3(mn0, mn1, mn2)
  }

  pub fn mat3_minus(m: [f32; 9], n: [f32; 9]) -> [f32; 9] {
    let (m0, m1, m2) = Math::mat3_into_3_vec3(m);
    let (n0, n1, n2) = Math::mat3_into_3_vec3(n);

    let mn0 = Math::vec3_minus(m0, n0);
    let mn1 = Math::vec3_minus(m1, n1);
    let mn2 = Math::vec3_minus(m2, n2);

    Math::mat3_from_3_vec3(mn0, mn1, mn2)
  }

  pub fn mat3_mul(m: [f32; 9], n: [f32; 9]) -> [f32; 9] {
    let mut result = Math::mat3_identity();

    let r = 3;
    result[r * 0 + 0] =
      m[r * 0 + 0] * n[r * 0 + 0] + m[r * 1 + 0] * n[r * 0 + 1] + m[r * 2 + 0] * n[r * 0 + 2];
    result[r * 0 + 1] =
      m[r * 0 + 1] * n[r * 0 + 0] + m[r * 1 + 1] * n[r * 0 + 1] + m[r * 2 + 1] * n[r * 0 + 2];
    result[r * 0 + 2] =
      m[r * 0 + 2] * n[r * 0 + 0] + m[r * 1 + 2] * n[r * 0 + 1] + m[r * 2 + 2] * n[r * 0 + 2];
    result[r * 1 + 0] =
      m[r * 0 + 0] * n[r * 1 + 0] + m[r * 1 + 0] * n[r * 1 + 1] + m[r * 2 + 0] * n[r * 1 + 2];
    result[r * 1 + 1] =
      m[r * 0 + 1] * n[r * 1 + 0] + m[r * 1 + 1] * n[r * 1 + 1] + m[r * 2 + 1] * n[r * 1 + 2];
    result[r * 1 + 2] =
      m[r * 0 + 2] * n[r * 1 + 0] + m[r * 1 + 2] * n[r * 1 + 1] + m[r * 2 + 2] * n[r * 1 + 2];
    result[r * 2 + 0] =
      m[r * 0 + 0] * n[r * 2 + 0] + m[r * 1 + 0] * n[r * 2 + 1] + m[r * 2 + 0] * n[r * 2 + 2];
    result[r * 2 + 1] =
      m[r * 0 + 1] * n[r * 2 + 0] + m[r * 1 + 1] * n[r * 2 + 1] + m[r * 2 + 1] * n[r * 2 + 2];
    result[r * 2 + 2] =
      m[r * 0 + 2] * n[r * 2 + 0] + m[r * 1 + 2] * n[r * 2 + 1] + m[r * 2 + 2] * n[r * 2 + 2];

    result
  }

  pub fn mat3_mul_f32(m: [f32; 9], s: f32) -> [f32; 9] {
    let (v0, v1, v2) = Math::mat3_into_3_vec3(m);

    let m0 = Math::vec3_mul_f32(v0, s);
    let m1 = Math::vec3_mul_f32(v1, s);
    let m2 = Math::vec3_mul_f32(v2, s);

    Math::mat3_from_3_vec3(m0, m1, m2)
  }

  pub fn mat3_div_f32(m: [f32; 9], s: f32) -> [f32; 9] {
    let (v0, v1, v2) = Math::mat3_into_3_vec3(m);

    let m0 = Math::vec3_div_f32(v0, s);
    let m1 = Math::vec3_div_f32(v1, s);
    let m2 = Math::vec3_div_f32(v2, s);

    Math::mat3_from_3_vec3(m0, m1, m2)
  }

  pub fn mat3_transpose(m: [f32; 9]) -> [f32; 9] {
    let mut n = Math::mat3_identity();

    let r = 3;
    n[r * 0 + 0] = m[r * 0 + 0];
    n[r * 0 + 1] = m[r * 1 + 0];
    n[r * 0 + 2] = m[r * 2 + 0];

    n[r * 1 + 0] = m[r * 0 + 1];
    n[r * 1 + 1] = m[r * 1 + 1];
    n[r * 1 + 2] = m[r * 2 + 1];

    n[r * 1 + 0] = m[r * 0 + 2];
    n[r * 1 + 1] = m[r * 1 + 2];
    n[r * 1 + 2] = m[r * 2 + 2];

    n
  }

  pub fn mat3_determinant(m: [f32; 9]) -> f32 {
    let r = 3;

    m[r * 0 + 0] * (m[r * 1 + 1] * m[r * 2 + 2] - m[r * 2 + 1] * m[r * 1 + 2]) -
      m[r * 1 + 0] * (m[r * 0 + 1] * m[r * 2 + 2] - m[r * 2 + 1] * m[r * 0 + 2]) +
      m[r * 2 + 0] * (m[r * 0 + 1] * m[r * 1 + 2] - m[r * 1 + 1] * m[r * 0 + 2])
  }

  pub fn mat3_inverse(m: [f32; 9]) -> [f32; 9] {
    let mut inverse = Math::mat3_identity();

    let one_over_det = 1.0 / Math::mat3_determinant(m);

    let r = 3;
    inverse[r * 0 + 0] = (m[r * 1 + 1] * m[r * 2 + 2] - m[r * 2 + 1] * m[r * 1 + 2]) * one_over_det;
    inverse[r * 1 + 0] = (m[r * 1 + 0] * m[r * 2 + 2] - m[r * 2 + 0] * m[r * 1 + 2]) * one_over_det;
    inverse[r * 2 + 0] = (m[r * 1 + 0] * m[r * 2 + 1] - m[r * 2 + 0] * m[r * 1 + 1]) * one_over_det;
    inverse[r * 0 + 1] = (m[r * 0 + 1] * m[r * 2 + 2] - m[r * 2 + 1] * m[r * 0 + 2]) * one_over_det;
    inverse[r * 1 + 1] = (m[r * 0 + 0] * m[r * 2 + 2] - m[r * 2 + 0] * m[r * 0 + 2]) * one_over_det;
    inverse[r * 2 + 1] = (m[r * 0 + 0] * m[r * 2 + 1] - m[r * 2 + 0] * m[r * 0 + 1]) * one_over_det;
    inverse[r * 0 + 2] = (m[r * 0 + 1] * m[r * 1 + 2] - m[r * 1 + 1] * m[r * 0 + 2]) * one_over_det;
    inverse[r * 1 + 2] = (m[r * 0 + 0] * m[r * 1 + 2] - m[r * 1 + 0] * m[r * 0 + 2]) * one_over_det;
    inverse[r * 2 + 2] = (m[r * 0 + 0] * m[r * 1 + 1] - m[r * 1 + 0] * m[r * 0 + 1]) * one_over_det;

    inverse
  }

  pub fn mat4_identity() -> [f32; 16] {
    [
      1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
  }

  pub fn mat4_scale(mat4: [f32; 16], s: [f32; 3]) -> [f32; 16] {
    let mut scale_matrix = mat4;

    let r = 4;

    scale_matrix[r * 0 + 0] *= s[0];
    scale_matrix[r * 1 + 1] *= s[1];
    scale_matrix[r * 2 + 2] *= s[2];

    scale_matrix
  }

  pub fn mat4_from_mat2(m2: [f32; 4]) -> [f32; 16] {
    let mut m = Math::mat4_identity();

    m[0] = m2[0];
    m[1] = m2[1];
    m[4] = m2[2];
    m[5] = m2[3];

    m
  }

  pub fn mat4_from_mat3(m3: [f32; 9]) -> [f32; 16] {
    let mut m = Math::mat4_identity();

    m[0] = m3[0];
    m[1] = m3[1];
    m[2] = m3[2];

    m[4] = m3[3];
    m[5] = m3[4];
    m[6] = m3[5];

    m[8] = m3[6];
    m[9] = m3[7];
    m[10] = m3[8];

    m
  }

  pub fn mat4_add(m: [f32; 16], n: [f32; 16]) -> [f32; 16] {
    let mut matrix = m;
    for i in 0..16 {
      matrix[i] += n[i];
    }

    matrix
  }

  pub fn mat4_minus(m: [f32; 16], n: [f32; 16]) -> [f32; 16] {
    let mut matrix = m;
    for i in 0..16 {
      matrix[i] -= n[i];
    }

    matrix
  }

  pub fn mat4_add_f32(m: [f32; 16], s: f32) -> [f32; 16] {
    let mut matrix = m;
    for i in 0..16 {
      matrix[i] += s;
    }
    matrix
  }

  pub fn mat4_mul_f32(m: [f32; 16], x: f32) -> [f32; 16] {
    let mut matrix = Math::mat4_identity();

    for i in 0..16 {
      matrix[i] = m[i] * x;
    }

    matrix
  }

  pub fn mat4_mul_vec3(m: [f32; 16], x: [f32; 3]) -> [f32; 16] {
    let mut matrix = m;

    let row0 = Math::vec4_mul_f32([m[0], m[1], m[2], m[3]], x[0]);
    let row1 = Math::vec4_mul_f32([m[4], m[5], m[6], m[7]], x[1]);
    let row2 = Math::vec4_mul_f32([m[8], m[9], m[10], m[11]], x[2]);

    matrix[0] = row0[0];
    matrix[1] = row0[1];
    matrix[2] = row0[2];
    matrix[3] = row0[3];

    matrix[4] = row1[0];
    matrix[5] = row1[1];
    matrix[6] = row1[2];
    matrix[7] = row1[3];

    matrix[8] = row2[0];
    matrix[9] = row2[1];
    matrix[10] = row2[2];
    matrix[11] = row2[3];

    matrix
  }

  pub fn mat4_from_vec4(v: [f32; 4]) -> [f32; 16] {
    let mut m = [0.0; 16];

    m[0] = v[0];
    m[5] = v[1];
    m[10] = v[2];
    m[15] = v[3];

    m
  }

  pub fn mat4_from_vec3(v: [f32; 3]) -> [f32; 16] {
    let mut m = [0.0; 16];

    m[0] = v[0];
    m[5] = v[1];
    m[10] = v[2];
    m[15] = 1.0;

    m
  }

  pub fn mat4_mul_vec4(m: [f32; 16], x: [f32; 4]) -> [f32; 16] {
    let mut matrix = m;

    let row0 = Math::vec4_mul_f32([m[0], m[1], m[2], m[3]], x[0]);
    let row1 = Math::vec4_mul_f32([m[4], m[5], m[6], m[7]], x[1]);
    let row2 = Math::vec4_mul_f32([m[8], m[9], m[10], m[11]], x[2]);
    let row3 = Math::vec4_mul_f32([m[8], m[9], m[10], m[11]], x[3]);

    matrix[0] = row0[0];
    matrix[1] = row0[1];
    matrix[2] = row0[2];
    matrix[3] = row0[3];

    matrix[4] = row1[0];
    matrix[5] = row1[1];
    matrix[6] = row1[2];
    matrix[7] = row1[3];

    matrix[8] = row2[0];
    matrix[9] = row2[1];
    matrix[10] = row2[2];
    matrix[11] = row2[3];

    matrix[12] = row3[0];
    matrix[13] = row3[1];
    matrix[14] = row3[2];
    matrix[15] = row3[3];

    matrix
  }

  pub fn mat4_div_f32(m: [f32; 16], x: f32) -> [f32; 16] {
    let mut matrix = Math::mat4_identity();

    for i in 0..16 {
      matrix[i] = m[i] / x;
    }

    matrix
  }

  pub fn mat4_transpose(m: [f32; 16]) -> [f32; 16] {
    let mut matrix = [0.0; 16];
    let r = 4;

    matrix[r * 0 + 0] = m[r * 0 + 0];
    matrix[r * 0 + 1] = m[r * 1 + 0];
    matrix[r * 0 + 2] = m[r * 2 + 0];
    matrix[r * 0 + 3] = m[r * 3 + 0];

    matrix[r * 1 + 0] = m[r * 0 + 1];
    matrix[r * 1 + 1] = m[r * 1 + 1];
    matrix[r * 1 + 2] = m[r * 2 + 1];
    matrix[r * 1 + 3] = m[r * 3 + 1];

    matrix[r * 2 + 0] = m[r * 0 + 2];
    matrix[r * 2 + 1] = m[r * 1 + 2];
    matrix[r * 2 + 2] = m[r * 2 + 2];
    matrix[r * 2 + 3] = m[r * 3 + 2];

    matrix[r * 3 + 0] = m[r * 0 + 3];
    matrix[r * 3 + 1] = m[r * 1 + 3];
    matrix[r * 3 + 2] = m[r * 2 + 3];
    matrix[r * 3 + 3] = m[r * 3 + 3];

    matrix
  }

  pub fn mat4_determinant(m: [f32; 16]) -> f32 {
    let r = 4;

    let sub_factor0 = m[r * 2 + 2] * m[r * 3 + 3] - m[r * 3 + 2] * m[r * 2 + 3];
    let sub_factor1 = m[r * 2 + 1] * m[r * 3 + 3] - m[r * 3 + 1] * m[r * 2 + 3];
    let sub_factor2 = m[r * 2 + 1] * m[r * 3 + 2] - m[r * 3 + 1] * m[r * 2 + 2];
    let sub_factor3 = m[r * 2 + 0] * m[r * 3 + 3] - m[r * 3 + 0] * m[r * 2 + 3];
    let sub_factor4 = m[r * 2 + 0] * m[r * 3 + 2] - m[r * 3 + 0] * m[r * 2 + 2];
    let sub_factor5 = m[r * 2 + 0] * m[r * 3 + 1] - m[r * 3 + 0] * m[r * 2 + 1];

    let def_cof = [
      (m[r * 1 + 1] * sub_factor0 - m[r * 1 + 2] * sub_factor1 + m[r * 1 + 3] * sub_factor2),
      -(m[r * 1 + 0] * sub_factor0 - m[r * 1 + 2] * sub_factor3 + m[r * 1 + 3] * sub_factor4),
      (m[r * 1 + 0] * sub_factor1 - m[r * 1 + 1] * sub_factor3 + m[r * 1 + 3] * sub_factor5),
      -(m[r * 1 + 0] * sub_factor2 - m[r * 1 + 1] * sub_factor4 + m[r * 1 + 2] * sub_factor5),
    ];

    m[r * 0 + 0] * def_cof[0] +
      m[r * 0 + 1] * def_cof[1] +
      m[r * 0 + 2] * def_cof[2] +
      m[r * 0 + 3] * def_cof[3]
  }

  pub fn mat4_translate_vec3(mut m: [f32; 16], v: [f32; 3]) -> [f32; 16] {
    let r = 4;

    let m_0 = Math::vec4_mul_f32(
      [m[r * 0 + 0], m[r * 0 + 1], m[r * 0 + 2], m[r * 0 + 3]],
      v[0],
    );
    let m_1 = Math::vec4_mul_f32(
      [m[r * 1 + 0], m[r * 1 + 1], m[r * 1 + 2], m[r * 1 + 3]],
      v[1],
    );
    let m_2 = Math::vec4_mul_f32(
      [m[r * 2 + 0], m[r * 2 + 1], m[r * 2 + 2], m[r * 2 + 3]],
      v[2],
    );
    let m_3 = [m[r * 3 + 0], m[r * 3 + 1], m[r * 3 + 2], m[r * 3 + 3]];

    let row = [
      m_0[0] + m_1[0] + m_2[0] + m_3[0],
      m_0[1] + m_1[1] + m_2[1] + m_3[1],
      m_0[2] + m_1[2] + m_2[2] + m_3[2],
      m_0[3] + m_1[3] + m_2[3] + m_3[3],
    ];
    m[r * 3 + 0] = row[0];
    m[r * 3 + 1] = row[1];
    m[r * 3 + 2] = row[2];
    m[r * 3 + 3] = row[3];

    m
    /*
    let mut translation = Math::mat4_identity();
    translation[r*0 + 3] = v[0];
    translation[r*1 + 3] = v[1];
    translation[r*2 + 3] = v[2];

    Math::mat4_mul(m, translation)*/
  }

  pub fn mat4_mul(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
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
      a11 * b11 + a12 * b21 + a13 * b31 + a14 * b41,
      a11 * b12 + a12 * b22 + a13 * b32 + a14 * b42,
      a11 * b13 + a12 * b23 + a13 * b33 + a14 * b43,
      a11 * b14 + a12 * b24 + a13 * b34 + a14 * b44,
      a21 * b11 + a22 * b21 + a23 * b31 + a24 * b41,
      a21 * b12 + a22 * b22 + a23 * b32 + a24 * b42,
      a21 * b13 + a22 * b23 + a23 * b33 + a24 * b43,
      a21 * b14 + a22 * b24 + a23 * b34 + a24 * b44,
      a31 * b11 + a32 * b21 + a33 * b31 + a34 * b41,
      a31 * b12 + a32 * b22 + a33 * b32 + a34 * b42,
      a31 * b13 + a32 * b23 + a33 * b33 + a34 * b43,
      a31 * b14 + a32 * b24 + a33 * b34 + a34 * b44,
      a41 * b11 + a42 * b21 + a43 * b31 + a44 * b41,
      a41 * b12 + a42 * b22 + a43 * b32 + a44 * b42,
      a41 * b13 + a42 * b23 + a43 * b33 + a44 * b43,
      a41 * b14 + a42 * b24 + a43 * b34 + a44 * b44,
    ]
  }

  pub fn mat4_flatten(m: [f32; 16]) -> Vec<f32> {
    let mut matrix = Vec::new();

    for i in 0..16 {
      matrix.push(m[i]);
    }

    matrix
  }

  pub fn mat4_inverse(m: [f32; 16]) -> [f32; 16] {
    let mut inv = [0.0; 16];
    let mut det;

    inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15] +
      m[9] * m[7] * m[14] +
      m[13] * m[6] * m[11] -
      m[13] * m[7] * m[10];

    inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15] -
      m[8] * m[7] * m[14] -
      m[12] * m[6] * m[11] +
      m[12] * m[7] * m[10];

    inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15] +
      m[8] * m[7] * m[13] +
      m[12] * m[5] * m[11] -
      m[12] * m[7] * m[9];

    inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14] -
      m[8] * m[6] * m[13] -
      m[12] * m[5] * m[10] +
      m[12] * m[6] * m[9];

    inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15] -
      m[9] * m[3] * m[14] -
      m[13] * m[2] * m[11] +
      m[13] * m[3] * m[10];

    inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15] +
      m[8] * m[3] * m[14] +
      m[12] * m[2] * m[11] -
      m[12] * m[3] * m[10];

    inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15] -
      m[8] * m[3] * m[13] -
      m[12] * m[1] * m[11] +
      m[12] * m[3] * m[9];

    inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14] +
      m[8] * m[2] * m[13] +
      m[12] * m[1] * m[10] -
      m[12] * m[2] * m[9];

    inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15] +
      m[5] * m[3] * m[14] +
      m[13] * m[2] * m[7] -
      m[13] * m[3] * m[6];

    inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15] -
      m[4] * m[3] * m[14] -
      m[12] * m[2] * m[7] +
      m[12] * m[3] * m[6];

    inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15] +
      m[4] * m[3] * m[13] +
      m[12] * m[1] * m[7] -
      m[12] * m[3] * m[5];

    inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14] -
      m[4] * m[2] * m[13] -
      m[12] * m[1] * m[6] +
      m[12] * m[2] * m[5];

    inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11] -
      m[5] * m[3] * m[10] -
      m[9] * m[2] * m[7] +
      m[9] * m[3] * m[6];

    inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11] +
      m[4] * m[3] * m[10] +
      m[8] * m[2] * m[7] -
      m[8] * m[3] * m[6];

    inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11] -
      m[4] * m[3] * m[9] -
      m[8] * m[1] * m[7] +
      m[8] * m[3] * m[5];

    inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10] +
      m[4] * m[2] * m[9] +
      m[8] * m[1] * m[6] -
      m[8] * m[2] * m[5];

    det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];

    //if (det == 0)
    //    return false;

    det = 1.0 / det;

    let mut inverse = [0.0; 16];
    for i in 0..16 {
      inverse[i] = inv[i] * det;
    }

    inverse
  }

  pub fn mat4_rotate_eular_axis(mat4: [f32; 16], angle: f32, axis: [f32; 3]) -> [f32; 16] {
    let a = angle;
    let c = a.cos();
    let s = a.sin();

    let axis = axis;
    let temp = Math::vec3_mul_f32(axis, 1.0 - c);

    let mut rotate = [0.0; 16];

    let r = 4;

    rotate[r * 0 + 0] = c + temp[0] * axis[0];
    rotate[r * 0 + 1] = 0.0 + temp[0] * axis[1] + s * axis[2];
    rotate[r * 0 + 2] = 0.0 + temp[0] * axis[2] - s * axis[1];

    rotate[r * 1 + 0] = 0.0 + temp[1] * axis[0] - s * axis[2];
    rotate[r * 1 + 1] = c + temp[1] * axis[1];
    rotate[r * 1 + 2] = 0.0 + temp[1] * axis[2] + s * axis[0];

    rotate[r * 2 + 0] = 0.0 + temp[2] * axis[0] + s * axis[1];
    rotate[r * 2 + 1] = 0.0 + temp[2] * axis[1] - s * axis[0];
    rotate[r * 2 + 2] = c + temp[2] * axis[2];

    let mut result = [0.0; 16];

    result[r * 0 + 0] = mat4[r * 0 + 0] * rotate[r * 0 + 0] +
      mat4[r * 1 + 0] * rotate[r * 0 + 1] +
      mat4[r * 2 + 0] * rotate[r * 0 + 2];
    result[r * 0 + 1] = mat4[r * 0 + 1] * rotate[r * 0 + 0] +
      mat4[r * 1 + 1] * rotate[r * 0 + 1] +
      mat4[r * 2 + 1] * rotate[r * 0 + 2];
    result[r * 0 + 2] = mat4[r * 0 + 2] * rotate[r * 0 + 0] +
      mat4[r * 1 + 2] * rotate[r * 0 + 1] +
      mat4[r * 2 + 2] * rotate[r * 0 + 2];
    result[r * 0 + 3] = mat4[r * 0 + 3] * rotate[r * 0 + 0] +
      mat4[r * 1 + 3] * rotate[r * 0 + 1] +
      mat4[r * 2 + 3] * rotate[r * 0 + 2];

    result[r * 1 + 0] = mat4[r * 0 + 0] * rotate[r * 1 + 0] +
      mat4[r * 1 + 0] * rotate[r * 1 + 1] +
      mat4[r * 2 + 0] * rotate[r * 1 + 2];
    result[r * 1 + 1] = mat4[r * 0 + 1] * rotate[r * 1 + 0] +
      mat4[r * 1 + 1] * rotate[r * 1 + 1] +
      mat4[r * 2 + 1] * rotate[r * 1 + 2];
    result[r * 1 + 2] = mat4[r * 0 + 2] * rotate[r * 1 + 0] +
      mat4[r * 1 + 2] * rotate[r * 1 + 1] +
      mat4[r * 2 + 2] * rotate[r * 1 + 2];
    result[r * 1 + 3] = mat4[r * 0 + 3] * rotate[r * 1 + 0] +
      mat4[r * 1 + 3] * rotate[r * 1 + 1] +
      mat4[r * 2 + 3] * rotate[r * 1 + 2];

    result[r * 2 + 0] = mat4[r * 0 + 0] * rotate[r * 2 + 0] +
      mat4[r * 1 + 0] * rotate[r * 2 + 1] +
      mat4[r * 2 + 0] * rotate[r * 2 + 2];
    result[r * 2 + 1] = mat4[r * 0 + 1] * rotate[r * 2 + 0] +
      mat4[r * 1 + 1] * rotate[r * 2 + 1] +
      mat4[r * 2 + 1] * rotate[r * 2 + 2];
    result[r * 2 + 2] = mat4[r * 0 + 2] * rotate[r * 2 + 0] +
      mat4[r * 1 + 2] * rotate[r * 2 + 1] +
      mat4[r * 2 + 2] * rotate[r * 2 + 2];
    result[r * 2 + 3] = mat4[r * 0 + 3] * rotate[r * 2 + 0] +
      mat4[r * 1 + 3] * rotate[r * 2 + 1] +
      mat4[r * 2 + 3] * rotate[r * 2 + 2];

    result[r * 3 + 0] = mat4[r * 3 + 0];
    result[r * 3 + 1] = mat4[r * 3 + 1];
    result[r * 3 + 2] = mat4[r * 3 + 2];
    result[r * 3 + 3] = mat4[r * 3 + 3];

    result
  }

  pub fn mat4_axis_rotate(mat4: [f32; 16], angle: f32, axis: [f32; 3]) -> [f32; 16] {
    let a = angle;
    let c = a.cos();
    let s = a.sin();

    let axis = axis;
    let temp = Math::vec3_mul_f32(axis, 1.0 - c);

    let mut rotate = [0.0; 16];

    let r = 4;

    rotate[r * 0 + 0] = c + temp[0] * axis[0];
    rotate[r * 0 + 1] = 0.0 + temp[0] * axis[1] + s * axis[2];
    rotate[r * 0 + 2] = 0.0 + temp[0] * axis[2] - s * axis[1];

    rotate[r * 1 + 0] = 0.0 + temp[1] * axis[0] - s * axis[2];
    rotate[r * 1 + 1] = c + temp[1] * axis[1];
    rotate[r * 1 + 2] = 0.0 + temp[1] * axis[2] + s * axis[0];

    rotate[r * 2 + 0] = 0.0 + temp[2] * axis[0] + s * axis[1];
    rotate[r * 2 + 1] = 0.0 + temp[2] * axis[1] - s * axis[0];
    rotate[r * 2 + 2] = c + temp[2] * axis[2];

    let mut result = [0.0; 16];

    result[r * 0 + 0] = mat4[r * 0 + 0] * rotate[r * 0 + 0] +
      mat4[r * 1 + 0] * rotate[r * 0 + 1] +
      mat4[r * 2 + 0] * rotate[r * 0 + 2];
    result[r * 0 + 1] = mat4[r * 0 + 1] * rotate[r * 0 + 0] +
      mat4[r * 1 + 1] * rotate[r * 0 + 1] +
      mat4[r * 2 + 1] * rotate[r * 0 + 2];
    result[r * 0 + 2] = mat4[r * 0 + 2] * rotate[r * 0 + 0] +
      mat4[r * 1 + 2] * rotate[r * 0 + 1] +
      mat4[r * 2 + 2] * rotate[r * 0 + 2];
    result[r * 0 + 3] = mat4[r * 0 + 3] * rotate[r * 0 + 0] +
      mat4[r * 1 + 3] * rotate[r * 0 + 1] +
      mat4[r * 2 + 3] * rotate[r * 0 + 2];

    result[r * 1 + 0] = mat4[r * 0 + 0] * rotate[r * 1 + 0] +
      mat4[r * 1 + 0] * rotate[r * 1 + 1] +
      mat4[r * 2 + 0] * rotate[r * 1 + 2];
    result[r * 1 + 1] = mat4[r * 0 + 1] * rotate[r * 1 + 0] +
      mat4[r * 1 + 1] * rotate[r * 1 + 1] +
      mat4[r * 2 + 1] * rotate[r * 1 + 2];
    result[r * 1 + 2] = mat4[r * 0 + 2] * rotate[r * 1 + 0] +
      mat4[r * 1 + 2] * rotate[r * 1 + 1] +
      mat4[r * 2 + 2] * rotate[r * 1 + 2];
    result[r * 1 + 3] = mat4[r * 0 + 3] * rotate[r * 1 + 0] +
      mat4[r * 1 + 3] * rotate[r * 1 + 1] +
      mat4[r * 2 + 3] * rotate[r * 1 + 2];

    result[r * 2 + 0] = mat4[r * 0 + 0] * rotate[r * 2 + 0] +
      mat4[r * 1 + 0] * rotate[r * 2 + 1] +
      mat4[r * 2 + 0] * rotate[r * 2 + 2];
    result[r * 2 + 1] = mat4[r * 0 + 1] * rotate[r * 2 + 0] +
      mat4[r * 1 + 1] * rotate[r * 2 + 1] +
      mat4[r * 2 + 1] * rotate[r * 2 + 2];
    result[r * 2 + 2] = mat4[r * 0 + 2] * rotate[r * 2 + 0] +
      mat4[r * 1 + 2] * rotate[r * 2 + 1] +
      mat4[r * 2 + 2] * rotate[r * 2 + 2];
    result[r * 2 + 3] = mat4[r * 0 + 3] * rotate[r * 2 + 0] +
      mat4[r * 1 + 3] * rotate[r * 2 + 1] +
      mat4[r * 2 + 3] * rotate[r * 2 + 2];

    result[r * 3 + 0] = mat4[r * 3 + 0];
    result[r * 3 + 1] = mat4[r * 3 + 1];
    result[r * 3 + 2] = mat4[r * 3 + 2];
    result[r * 3 + 3] = mat4[r * 3 + 3];

    result
  }

  pub fn quat_identity() -> [f32; 4] {
    [1.0, 0.0, 0.0, 0.0]
  }

  pub fn quat_equals(p: [f32; 4], q: [f32; 4]) -> bool {
    Math::vec4_equals(p, q)
  }

  pub fn quat_add(p: [f32; 4], q: [f32; 4]) -> [f32; 4] {
    let mut r: [f32; 4] = [0.0; 4];
    for i in 0..4 {
      r[i] = p[i] + q[i];
    }

    r
  }

  pub fn quat_minus(p: [f32; 4], q: [f32; 4]) -> [f32; 4] {
    let mut r: [f32; 4] = [0.0; 4];
    for i in 0..4 {
      r[i] = p[i] + q[i];
    }

    r
  }

  pub fn quat_mul(p: [f32; 4], q: [f32; 4]) -> [f32; 4] {
    let mut r = [0.0; 4];

    r[3] = p[3] * q[3] - p[0] * q[0] - p[1] * q[1] - p[2] * q[2];
    r[0] = p[3] * q[0] + p[0] * q[3] + p[1] * q[2] - p[2] * q[1];
    r[1] = p[3] * q[1] + p[1] * q[3] + p[2] * q[0] - p[0] * q[2];
    r[2] = p[3] * q[2] + p[2] * q[3] + p[0] * q[1] - p[1] * q[0];

    r
  }

  pub fn quat_mul_f32(p: [f32; 4], s: f32) -> [f32; 4] {
    Math::vec4_mul_f32(p, s)
  }

  pub fn quat_mul_vec3(p: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let quat_vec3 = [p[0], p[1], p[2]];
    let uv = Math::vec3_cross(quat_vec3, v);
    let uuv = Math::vec3_cross(quat_vec3, uv);

    Math::vec3_add(
      v,
      Math::vec3_mul_f32(Math::vec3_add(Math::vec3_mul_f32(uv, p[3]), uuv), 2.0),
    )
  }

  pub fn quat_mul_vec4(p: [f32; 4], q: [f32; 4]) -> [f32; 4] {
    Math::vec4_mul(p, q)
  }

  pub fn quat_div_f32(p: [f32; 4], s: f32) -> [f32; 4] {
    Math::vec4_div_f32(p, s)
  }

  pub fn quat_cross_vec3(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    Math::quat_mul_vec3(q, v)
  }

  pub fn quat_rotate_vec3(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    Math::quat_mul_vec3(q, v)
  }

  pub fn quat_rotate_vec4(q: [f32; 4], v: [f32; 4]) -> [f32; 4] {
    Math::quat_mul_vec4(q, v)
  }

  // Quaternion interpolation using the rotation short path
  pub fn quat_short_mix(x: [f32; 4], y: [f32; 4], a: f32) -> [f32; 4] {
    if a <= 0.0 {
      return x
    }
    if a >= 1.0 {
      return y
    }

    let mut f_cos = Math::vec4_dot(x, y);
    let mut y2 = y;

    if f_cos < 0.0 {
      y2 = [-y[0], -y[1], -y[2], -y[3]];
      f_cos = -f_cos;
    }

    let k0;
    let k1;
    if f_cos > 1.0 - f32::EPSILON {
      k0 = 1.0 - a;
      k1 = 0.0 + a;
    } else {
      let f_sin = (1.0f32 - f_cos as f32 * f_cos as f32).sqrt();
      let f_angle = (f_sin).atan2(f_cos);
      let f_one_over_sin = 1.0 / f_sin;
      k0 = ((1.0 - a) * f_angle).sin() * f_one_over_sin;
      k1 = ((0.0 + a) * f_angle).sin() * f_one_over_sin;
    }

    [
      k0 * x[0] + k1 * y2[0],
      k0 * x[1] + k1 * y2[1],
      k0 * x[2] + k1 * y2[2],
      k0 * x[3] + k1 * y2[3],
    ]
  }

  //  Quaternion normalized linear interpolation.
  pub fn quat_mix(x: [f32; 4], y: [f32; 4], a: f32) -> [f32; 4] {
    Math::vec4_normalise(Math::quat_add(
      Math::quat_mul_f32(x, 1.0 - a),
      Math::quat_mul_f32(y, a),
    ))
  }

  pub fn quat_length_sqrd(q: [f32; 4]) -> f32 {
    q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]
  }

  pub fn quat_slerp(qa: [f32; 4], qb: [f32; 4], t: f32) -> [f32; 4] {
    let mut qm = [0.0; 4];

    let cos_half_theta = qa[3] * qb[3] + qa[0] * qb[0] + qa[1] * qb[1] + qa[2] * qb[2];

    if (cos_half_theta).abs() >= 1.0 {
      qm = qa;
      return qm
    }

    let half_theta = (cos_half_theta).acos();
    let sin_half_theta = (1.0 - cos_half_theta * cos_half_theta).sqrt();

    if sin_half_theta.abs() < 0.001 {
      qm[0] = qa[0] * 0.5 + qb[0] * 0.5;
      qm[1] = qa[1] * 0.5 + qb[1] * 0.5;
      qm[2] = qa[2] * 0.5 + qb[2] * 0.5;
      qm[3] = qa[3] * 0.5 + qb[3] * 0.5;

      return qm
    }

    let ratio_a = ((1.0 - t) * half_theta).sin() / sin_half_theta;
    let ratio_b = (t * half_theta).sin() / sin_half_theta;

    qm[0] = qa[0] * ratio_a + qb[0] * ratio_b;
    qm[1] = qa[1] * ratio_a + qb[1] * ratio_b;
    qm[2] = qa[2] * ratio_a + qb[2] * ratio_b;
    qm[3] = qa[3] * ratio_a + qb[3] * ratio_b;

    qm
  }

  pub fn quat_to_mat4(quat: [f32; 4]) -> [f32; 16] {
    let mut matrix = Math::mat4_identity();

    let x = quat[0];
    let y = quat[1];
    let z = quat[2];
    let w = quat[3];

    let r = 4;
    matrix[r * 0 + 0] = 1.0 - 2.0 * y * y - 2.0 * z * z;
    matrix[r * 0 + 1] = 2.0 * x * y - 2.0 * z * w;
    matrix[r * 0 + 2] = 2.0 * x * z + 2.0 * y * w;
    matrix[r * 0 + 3] = 0.0;

    matrix[r * 1 + 0] = 2.0 * x * y + 2.0 * z * w;
    matrix[r * 1 + 1] = 1.0 - 2.0 * x * x - 2.0 * z * z;
    matrix[r * 1 + 2] = 2.0 * y * z - 2.0 * x * w;
    matrix[r * 1 + 3] = 0.0;

    matrix[r * 2 + 0] = 2.0 * x * z - 2.0 * y * w;
    matrix[r * 2 + 1] = 2.0 * y * z + 2.0 * x * w;
    matrix[r * 2 + 2] = 1.0 - 2.0 * x * x - 2.0 * y * y;
    matrix[r * 2 + 3] = 0.0;

    matrix[r * 3 + 0] = 0.0;
    matrix[r * 3 + 1] = 0.0;
    matrix[r * 3 + 2] = 0.0;
    matrix[r * 3 + 3] = 1.0;

    matrix
  }

  pub fn quat_from_p_y_r() {}
  //quat_from_roatation(
  //	valType const& pitch,
  //	valType const& yaw,
  //	valType const& roll
  //)
  //{
  //	vec<3, valType> eulerAngle(pitch * valType(0.5), yaw * valType(0.5), roll * valType(0.5));
  //	vec<3, valType> c = glm::cos(eulerAngle * valType(0.5));
  //	vec<3, valType> s = glm::sin(eulerAngle * valType(0.5));
  //
  //	this->w = c.x * c.y * c.z + s.x * s.y * s.z;
  //	this->x = s.x * c.y * c.z - c.x * s.y * s.z;
  //	this->y = c.x * s.y * c.z + s.x * c.y * s.z;
  //	this->z = c.x * c.y * s.z - s.x * s.y * c.z;
  //}

  pub fn perspective(fovy: f32, aspect: f32, znear: f32, zfar: f32, flip_y: bool) -> [f32; 16] {
    let rad = fovy.to_radians();
    let tan_half_fovy = (rad / 2.0).tan();

    let r = 4;

    let mut matrix = [0.0; 16];

    //let s = 1.0 / tan_half_fovy;
    matrix[r * 0 + 0] = 1.0 / (aspect * tan_half_fovy);
    matrix[r * 1 + 1] = -1.0 / (tan_half_fovy);
    matrix[r * 2 + 2] = -(zfar - znear) / (zfar - znear);
    matrix[r * 2 + 3] = -1.0;
    matrix[r * 3 + 2] = -(2.0 * zfar * znear) / (zfar - znear);

    if flip_y {
      matrix[r * 1 + 1] *= -1.0;
    }

    matrix
  }
}
