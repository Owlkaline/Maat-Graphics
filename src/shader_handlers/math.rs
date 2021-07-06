
pub struct Math {}

impl Math {
  pub fn vec3_equals(p: [f32; 3], q: [f32; 3]) -> bool {
    p[0] == q[0] && p[1] == q[1] && p[2] == q[2]
  }
  
  pub fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]+b[0], a[1]+b[1], a[2]+b[2]]
  }
  
  pub fn vec3_minus(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
  }
  
  pub fn vec3_mul(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]*b[0], a[1]*b[1], a[2]*b[2]]
  }
  
  pub fn vec3_div(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]/b[0], a[1]/b[1], a[2]/b[2]]
  }
  
  pub fn vec3_normalise(a: [f32; 3]) -> [f32; 3] {
    let mag = (a[0]*a[0] + a[1]*a[1] + a[2]*a[2]).sqrt();
    
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
     a[1]*b[2] - b[1]*a[2],
     a[2]*b[0] - b[2]*a[0],
     a[0]*b[1] - b[0]*a[1]
    ]
  }
  
  pub fn vec4_equals(p: [f32; 4], q: [f32; 4]) -> bool {
    p[0] == q[0] && p[1] == q[1] && p[2] == q[2] && p[3] == q[3]
  }
  
  pub fn vec4_add(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0]+b[0], a[1]+b[1], a[2]+b[2], a[3]+b[3]]
  }
  
  pub fn vec4_minus(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2], a[3]-b[3]]
  }
  
  pub fn vec4_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0]*b[0], a[1]*b[1], a[2]*b[2], a[3]*b[3]]
  }
  
  pub fn vec4_div(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0]/b[0], a[1]/b[1], a[2]/b[2], a[3]/b[3]]
  }
  
  pub fn vec4_normalise(a: [f32; 4]) -> [f32; 4] {
    let mag = (a[0]*a[0] + a[1]*a[1] + a[2]*a[2] + a[3]*a[3]).sqrt();
    
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
  
  pub fn mat4_identity() -> [f32; 16] {
    [1.0, 0.0, 0.0, 0.0,
     0.0, 1.0, 0.0, 0.0,
     0.0, 0.0, 1.0, 0.0,
     0.0, 0.0, 0.0, 1.0]
  }
  
  pub fn mat4_scale_vec3(mat4: [f32; 16], s: [f32; 3]) -> [f32; 16] {
    let mut scale_matrix =  mat4;
    /*let row_0 = Math::vec4_mul_f32([mat4[0], mat4[1], mat4[2], mat4[3]], s[0]);
    let row_1 = Math::vec4_mul_f32([mat4[4], mat4[5], mat4[6], mat4[7]], s[1]);
    let row_2 = Math::vec4_mul_f32([mat4[8], mat4[9], mat4[10], mat4[11]], s[2]);
    let row_3 = [mat4[12], mat4[13], mat4[14], mat4[15]];
    
    let r = 4;
    
    for i in 0..4 {
      scale_matrix[r*0 + i] = row_0[i];
      scale_matrix[r*1 + i] = row_1[i];
      scale_matrix[r*2 + i] = row_2[i];
      scale_matrix[r*3 + i] = row_3[i];
    }*/
    
    let r = 4;
    
    scale_matrix[r*0 + 0] *= s[0];
    scale_matrix[r*1 + 1] *= s[1];
    scale_matrix[r*2 + 2] *= s[2];
    
    scale_matrix
  }
  /*
  pub fn mat4_scale_vec4(mat4: [f32; 16], s: [f32; 4]) -> [f32; 16] {
    let mut scale_matrix =  Math::mat4_identity();
    scale_matrix[0] *= s[0];
    scale_matrix[5] *= s[1];
    scale_matrix[10] *= s[2];
    scale_matrix[15] *= s[3];
    
     Math::mat4_mul(scale_matrix, mat4)
  }*/
  
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
    let mut matrix =  Math::mat4_identity();
    
    for i in 0..16 {
      matrix[i] = m[i]*x;
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
    let mut matrix =  Math::mat4_identity();
    
    for i in 0..16 {
      matrix[i] = m[i]/x;
    }
    
    matrix
  }
  
  pub fn mat4_transpose(m: [f32; 16]) -> [f32; 16] {
    let mut matrix = [0.0; 16];
    let r = 4;
    
    matrix[r*0 + 0] = m[r*0 + 0];
    matrix[r*0 + 1] = m[r*1 + 0];
    matrix[r*0 + 2] = m[r*2 + 0];
    matrix[r*0 + 3] = m[r*3 + 0];
    
    matrix[r*1 + 0] = m[r*0 + 1];
    matrix[r*1 + 1] = m[r*1 + 1];
    matrix[r*1 + 2] = m[r*2 + 1];
    matrix[r*1 + 3] = m[r*3 + 1];
    
    matrix[r*2 + 0] = m[r*0 + 2];
    matrix[r*2 + 1] = m[r*1 + 2];
    matrix[r*2 + 2] = m[r*2 + 2];
    matrix[r*2 + 3] = m[r*3 + 2];
    
    matrix[r*3 + 0] = m[r*0 + 3];
    matrix[r*3 + 1] = m[r*1 + 3];
    matrix[r*3 + 2] = m[r*2 + 3];
    matrix[r*3 + 3] = m[r*3 + 3];
    
    matrix
  }
  
  pub fn mat4_determinant(m: [f32; 16]) -> f32 {
    let r = 4;
    
    let sub_factor0 = m[r*2 + 2] * m[r*3 + 3] - m[r*3 + 2] * m[r*2 + 3];
    let sub_factor1 = m[r*2 + 1] * m[r*3 + 3] - m[r*3 + 1] * m[r*2 + 3];
    let sub_factor2 = m[r*2 + 1] * m[r*3 + 2] - m[r*3 + 1] * m[r*2 + 2];
    let sub_factor3 = m[r*2 + 0] * m[r*3 + 3] - m[r*3 + 0] * m[r*2 + 3];
    let sub_factor4 = m[r*2 + 0] * m[r*3 + 2] - m[r*3 + 0] * m[r*2 + 2];
    let sub_factor5 = m[r*2 + 0] * m[r*3 + 1] - m[r*3 + 0] * m[r*2 + 1];
    
    let def_cof = [
       (m[r*1 + 1] * sub_factor0 - m[r*1 + 2] * sub_factor1 + m[r*1 + 3] * sub_factor2),
      - (m[r*1 + 0] * sub_factor0 - m[r*1 + 2] * sub_factor3 + m[r*1 + 3] * sub_factor4),
       (m[r*1 + 0] * sub_factor1 - m[r*1 + 1] * sub_factor3 + m[r*1 + 3] * sub_factor5),
      - (m[r*1 + 0] * sub_factor2 - m[r*1 + 1] * sub_factor4 + m[r*1 + 2] * sub_factor5),
    ];
    
    m[r*0 + 0] * def_cof[0] + m[r*0 + 1] * def_cof[1] +
    m[r*0 + 2] * def_cof[2] + m[r*0 + 3] * def_cof[3]
  }
  
  pub fn mat4_translate_vec3(mut m: [f32; 16], v: [f32; 3]) -> [f32; 16] {
    let r = 4;
    
    let m_0 =  Math::vec4_mul_f32([m[r*0 + 0], m[r*0 + 1], m[r*0 + 2], m[r*0 + 3]], v[0]);
    let m_1 =  Math::vec4_mul_f32([m[r*1 + 0], m[r*1 + 1], m[r*1 + 2], m[r*1 + 3]], v[1]);
    let m_2 =  Math::vec4_mul_f32([m[r*2 + 0], m[r*2 + 1], m[r*2 + 2], m[r*2 + 3]], v[2]);
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
    /*
    let mut translation = Math::mat4_identity();
    translation[r*0 + 3] = v[0];
    translation[r*1 + 3] = v[1];
    translation[r*2 + 3] = v[2];
    
    Math::mat4_mul(m, translation)*/
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
  
  pub fn mat4_flatten(m: [f32; 16]) -> Vec<f32> {
    let mut matrix = Vec::new();
    
    for i in 0..16 {
      matrix.push(m[i]);
    }
    
    matrix
  }
  
  pub fn mat4_inverse(m: [f32; 16]) -> [f32; 16] {
    let mut inv = [0.0; 16];
    let mut det = 1.0;
    let mut i = 0;

    inv[0] = m[5]  * m[10] * m[15] - 
             m[5]  * m[11] * m[14] - 
             m[9]  * m[6]  * m[15] + 
             m[9]  * m[7]  * m[14] +
             m[13] * m[6]  * m[11] - 
             m[13] * m[7]  * m[10];

    inv[4] = -m[4]  * m[10] * m[15] + 
              m[4]  * m[11] * m[14] + 
              m[8]  * m[6]  * m[15] - 
              m[8]  * m[7]  * m[14] - 
              m[12] * m[6]  * m[11] + 
              m[12] * m[7]  * m[10];

    inv[8] = m[4]  * m[9] * m[15] - 
             m[4]  * m[11] * m[13] - 
             m[8]  * m[5] * m[15] + 
             m[8]  * m[7] * m[13] + 
             m[12] * m[5] * m[11] - 
             m[12] * m[7] * m[9];

    inv[12] = -m[4]  * m[9] * m[14] + 
               m[4]  * m[10] * m[13] +
               m[8]  * m[5] * m[14] - 
               m[8]  * m[6] * m[13] - 
               m[12] * m[5] * m[10] + 
               m[12] * m[6] * m[9];

    inv[1] = -m[1]  * m[10] * m[15] + 
              m[1]  * m[11] * m[14] + 
              m[9]  * m[2] * m[15] - 
              m[9]  * m[3] * m[14] - 
              m[13] * m[2] * m[11] + 
              m[13] * m[3] * m[10];

    inv[5] = m[0]  * m[10] * m[15] - 
             m[0]  * m[11] * m[14] - 
             m[8]  * m[2] * m[15] + 
             m[8]  * m[3] * m[14] + 
             m[12] * m[2] * m[11] - 
             m[12] * m[3] * m[10];

    inv[9] = -m[0]  * m[9] * m[15] + 
              m[0]  * m[11] * m[13] + 
              m[8]  * m[1] * m[15] - 
              m[8]  * m[3] * m[13] - 
              m[12] * m[1] * m[11] + 
              m[12] * m[3] * m[9];

    inv[13] = m[0]  * m[9] * m[14] - 
              m[0]  * m[10] * m[13] - 
              m[8]  * m[1] * m[14] + 
              m[8]  * m[2] * m[13] + 
              m[12] * m[1] * m[10] - 
              m[12] * m[2] * m[9];

    inv[2] = m[1]  * m[6] * m[15] - 
             m[1]  * m[7] * m[14] - 
             m[5]  * m[2] * m[15] + 
             m[5]  * m[3] * m[14] + 
             m[13] * m[2] * m[7] - 
             m[13] * m[3] * m[6];

    inv[6] = -m[0]  * m[6] * m[15] + 
              m[0]  * m[7] * m[14] + 
              m[4]  * m[2] * m[15] - 
              m[4]  * m[3] * m[14] - 
              m[12] * m[2] * m[7] + 
              m[12] * m[3] * m[6];

    inv[10] = m[0]  * m[5] * m[15] - 
              m[0]  * m[7] * m[13] - 
              m[4]  * m[1] * m[15] + 
              m[4]  * m[3] * m[13] + 
              m[12] * m[1] * m[7] - 
              m[12] * m[3] * m[5];

    inv[14] = -m[0]  * m[5] * m[14] + 
               m[0]  * m[6] * m[13] + 
               m[4]  * m[1] * m[14] - 
               m[4]  * m[2] * m[13] - 
               m[12] * m[1] * m[6] + 
               m[12] * m[2] * m[5];

    inv[3] = -m[1] * m[6] * m[11] + 
              m[1] * m[7] * m[10] + 
              m[5] * m[2] * m[11] - 
              m[5] * m[3] * m[10] - 
              m[9] * m[2] * m[7] + 
              m[9] * m[3] * m[6];

    inv[7] = m[0] * m[6] * m[11] - 
             m[0] * m[7] * m[10] - 
             m[4] * m[2] * m[11] + 
             m[4] * m[3] * m[10] + 
             m[8] * m[2] * m[7] - 
             m[8] * m[3] * m[6];

    inv[11] = -m[0] * m[5] * m[11] + 
               m[0] * m[7] * m[9] + 
               m[4] * m[1] * m[11] - 
               m[4] * m[3] * m[9] - 
               m[8] * m[1] * m[7] + 
               m[8] * m[3] * m[5];

    inv[15] = m[0] * m[5] * m[10] - 
              m[0] * m[6] * m[9] - 
              m[4] * m[1] * m[10] + 
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
    
    Math::vec3_add(v, Math::vec3_mul_f32(Math::vec3_add(Math::vec3_mul_f32(uv, p[3]), uuv), 2.0))
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
      return x;
    }
    if a >= 1.0 {
      return y;
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
      let f_sin = (1.0f32 - f_cos as f32*f_cos as f32).sqrt();
      let f_angle = (f_sin).atan2(f_cos);
      let  f_one_over_sin = 1.0 / f_sin;
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
    Math::vec4_normalise(Math::quat_add(Math::quat_mul_f32(x, (1.0 - a)), Math::quat_mul_f32(y, a)))
  }
  
  pub fn quat_length_sqrd(q: [f32; 4]) -> f32 {
    q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]
  }
  
  pub fn quat_slerp(qa: [f32; 4], qb: [f32; 4], t: f32) -> [f32; 4] {
    let mut qm = [0.0; 4];
    
    let cos_half_theta = qa[3] * qb[3] + qa[0] * qb[0] + qa[1] * qb[1] + qa[2] * qb[2];
    
    if (cos_half_theta).abs() >= 1.0 {
      qm = qa;
      return qm;
    }
    
    let half_theta = (cos_half_theta).acos();
    let sin_half_theta = (1.0 - cos_half_theta*cos_half_theta).sqrt();
    
    if sin_half_theta.abs() < 0.001 {
      qm[0] = qa[0] * 0.5 + qb[0] * 0.5;
      qm[1] = qa[1] * 0.5 + qb[1] * 0.5;
      qm[2] = qa[2] * 0.5 + qb[2] * 0.5;
      qm[3] = qa[3] * 0.5 + qb[3] * 0.5;
      
      return qm;
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
    let mut matrix =  Math::mat4_identity();
    
    let x = quat[0];
    let y = quat[1];
    let z = quat[2];
    let w = quat[3];
    
    let r = 4;
    matrix[r*0 + 0] = 1.0 - 2.0*y*y - 2.0*z*z;
    matrix[r*0 + 1] = 2.0*x*y - 2.0*z*w;
    matrix[r*0 + 2] = 2.0*x*z + 2.0*y*w;
    matrix[r*0 + 3] = 0.0;
    
    matrix[r*1 + 0] = 2.0*x*y + 2.0*z*w;
    matrix[r*1 + 1] = 1.0 - 2.0*x*x - 2.0*z*z;
    matrix[r*1 + 2] = 2.0*y*z - 2.0*x*w;
    matrix[r*1 + 3] = 0.0;
    
    matrix[r*2 + 0] = 2.0*x*z - 2.0*y*w;
    matrix[r*2 + 1] = 2.0*y*z + 2.0*x*w;
    matrix[r*2 + 2] = 1.0 - 2.0*x*x - 2.0*y*y;
    matrix[r*2 + 3] = 0.0;
    
    matrix[r*3 + 0] = 0.0;
    matrix[r*3 + 1] = 0.0;
    matrix[r*3 + 2] = 0.0;
    matrix[r*3 + 3] = 1.0;
    
    matrix
  }
  
  pub fn quat_from_p_y_r() {
    
  }
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
    
    let s = 1.0 / tan_half_fovy;
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
}
