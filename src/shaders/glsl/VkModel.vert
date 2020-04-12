#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec4 colour;
layout(location = 4) in vec4 tangent;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 v_colour;
layout(location = 2) out vec4 v_base_colour_factor;
layout(location = 3) out vec4 v_alpha_cutoff; // alpha, cutoff, _, use emissive
layout(location = 4) out vec3 v_normal;
layout(location = 5) out vec3 v_world_pos;
layout(location = 6) out vec3 v_camera_pos;
layout(location = 10) out vec3 v_scanline;
layout(location = 11) out vec4 v_use_textures;
layout(location = 12) out vec2 v_mr;

layout(set = 0, binding = 0) uniform UniformBuffer {
  vec4 use_textures; //base, metallic_roughness, normal, occlusion
  vec4 emissive_alpha; //use_emissive, normal_scale, alpha_cutoff, alpha_mask
  vec4 base_colour_factor; // r, g, b, a
  vec4 mro_factors; // metallic_factor, roughness_factor, occlusion_string, _
  vec4 emissive_factor; // r, g, b, _
} uniforms;

layout(push_constant) uniform PushConstants {
  vec4 c_position; // x, y, z, fov
  vec4 c_center;   // x, y, z, aspect
  vec4 c_up;       // x, y, z, x_scale
  vec4 model;      // x, y, z, y_scale
  vec4 rotation;   // x_rot, y_rot, z_rot, z_scale
  vec4 hologram_scanline; // hologram_enabled, scanline, _, _
} push_constants;

const float M_PI = 3.141592653589793;

float cot(float value) {
  return 1.0 / tan(value);
}

float to_radians(float degree) {
  return degree * (M_PI/180.0);
}

vec3 to_radians(vec3 degrees) {
  return vec3(to_radians(degrees.x), to_radians(degrees.y), to_radians(degrees.z));
}

mat4 create_perspective_matrix(float fov, float aspect, float near, float far) {
  float f = cot(to_radians(fov) / 2.0);
  
  mat4 perspective = mat4(
                      vec4(f / aspect, 0.0,   0.0,                               0.0),
                      vec4(0.0,        f,     0.0,                               0.0),
                      vec4(0.0,        0.0,   (far + near) / (near - far),      -1.0),
                      vec4(0.0,        0.0,   (2.0 * far * near) / (near - far), 0.0)
                    );
                
  return perspective;
}

// center is a point not a direction
mat4 create_view_matrix(vec3 eye, vec3 center, vec3 up) {
  vec3 dir = center - eye;
  
  vec3 f = normalize(dir);
  vec3 s = normalize(cross(f, up));
  vec3 u = cross(s,f);
  
  mat4 look_at_matrix = mat4(vec4(s.x,           u.x,        -f.x,         0.0), 
                             vec4(s.y,           u.y,        -f.y,         0.0), 
                             vec4(s.z,           u.z,        -f.z,         0.0), 
                             vec4(-dot(eye, s), -dot(eye, u), dot(eye, f), 1.0));
  
  return look_at_matrix;
}

vec4 quaternion_normalise(vec4 q) {
  float n = sqrt(q.x *q.x + q.y*q.y + q.z*q.z + q.w*q.w);
  
  return vec4(q.x/n, q.y/n, q.z/n, q.w/n);
}

vec4 quaternion_from_rotation(vec3 deg_rotation) {
  /*vec4 q = vec4(0.0);
  
  float h_angle = (deg_rotation*0.5) * M_PI / 180.0;
  q.x = axis.x * sin(h_angle);
  q.y = axis.y * sin(h_angle);
  q.z = axis.z * sin(h_angle);
  q.w = cos(h_angle);
  
  return q;*/
  //deg_rotation.x += 180.0;
  //deg_rotation.z += 270.0;
  //deg_rotation = vec3(0.0, 0.0, 0.0);
  float h = 0.5;
  float s_x = sin(to_radians(deg_rotation.x) * h); // 0
  float c_x = cos(to_radians(deg_rotation.x) * h); // 1
  
  float s_y = sin(to_radians(deg_rotation.z) * h); // 0
  float c_y = cos(to_radians(deg_rotation.z) * h); // 1
   
  float s_z = sin(to_radians(deg_rotation.y) * h); // 0
  float c_z = cos(to_radians(deg_rotation.y) * h); // 1
  
  vec4 q = vec4(0.0);
  q.w = -s_x*s_y*s_z + c_x*c_y*c_z; //1
  q.x =  s_x*c_y*c_z + s_y*s_z*c_x; //0
  q.z = -s_x*s_z*c_y + s_y*c_x*c_z; //0
  q.y = s_z*s_y*c_z + s_z*c_x*c_y;  // 
  
  return q;
  //return vec4(0.0, 0.707, 0.0, 1.0);
}

vec4 quaternion_conj(vec4 q) {
  return vec4(-q.xyz, q.w);
}

vec4 quaternion_from_position(vec3 pos) {
  return vec4(pos, 0.0);
}

vec4 quaternion_inverse(vec4 q) {
  vec4 q_1 = vec4(0.0);
  
  float n = sqrt(q.x *q.x + q.y*q.y + q.z*q.z + q.w*q.w);
  q_1 = vec4(-q.xyz, q.w) / n;
  //q_1 = vec4(-q.x/n, -q.y/n, -q.z/n, w/n);.
  return q_1;
}

vec4 quaternion_mul(vec4 q1, vec4 q2) {
  vec4 qr;
  qr.x = (q1.w * q2.x) + (q1.x * q2.w) + (q1.y * q2.z) - (q1.z * q2.y);
  qr.y = (q1.w * q2.y) - (q1.x * q2.z) + (q1.y * q2.w) + (q1.z * q2.x);
  qr.z = (q1.w * q2.z) + (q1.x * q2.y) - (q1.y * q2.x) + (q1.z * q2.w);
  qr.w = (q1.w * q2.w) - (q1.x * q2.x) - (q1.y * q2.y) - (q1.z * q2.z);
  return qr;
}

vec3 rotate_vertex_position(vec3 position, vec3 angle) { 
  vec4 q = quaternion_normalise(quaternion_from_rotation(angle));
  vec3 v = position.xyz;
  return v + 2.0 * cross(q.xyz, cross(q.xyz, v) + q.w * v);
}


mat4 create_rotation_matrix_from_quaternion(vec3 deg_rotation) {
  vec4 q = quaternion_from_rotation(deg_rotation);
  /*float s = sin(to_radians(deg_rotation.x) * 0.5) + sin(to_radians(deg_rotation.y) * 0.5) + sin(to_radians(deg_rotation.z) * 0.5);
  mat4 rotation_matrix = mat4(
          vec4(1.0 - 2.0*s*(q.y*q.y + q.z*q.z),       2.0*s*(q.x*q.y - q.z*q.w),       2.0*s*(q.x*q.z + q.y*q.w), 0.0),
          vec4(      2.0*s*(q.x*q.y + q.z*q.w), 1.0 - 2.0*s*(q.x*q.x + q.z*q.z),       2.0*s*(q.y*q.z - q.x*q.w), 0.0),
          vec4(      2.0*s*(q.x*q.z - q.y*q.w),       2.0*s*(q.y*q.z + q.x*q.w), 1.0 - 2.0*s*(q.x*q.x + q.y*q.y), 0.0),
          vec4(0.0, 0.0, 0.0, 1.0));
  */
  return rotation_matrix;
}

vec3 rotate_vector_by_angle(vec3 pos, vec3 rotation) {
  vec3 rotated_pos = rotate_vertex_position(pos, rotation);
  
  return rotated_pos;
}
/*
mat4 create_entire_rotation_matrix(vec3 rotations) {
  mat4 rot_x = create_rotation_matrix_from_quaternion(vec3(1.0, 0.0, 0.0), rotations.x);
  mat4 rot_y = create_rotation_matrix_from_quaternion(vec3(0.0, 1.0, 0.0), rotations.y);
  mat4 rot_z = create_rotation_matrix_from_quaternion(vec3(0.0, 0.0, 1.0), rotations.z);
  
  return rot_y*rot_z*rot_x;
}*/
/*
mat4 create_rotation_matrix(vec3 deg_rotation) {
  vec3 rotation = to_radians(deg_rotation);
  
  float s = sin(rotation.x);
  float c = cos(rotation.x);

  mat4 rot_x = mat4(vec4(1.0,  0.0, 0.0, 0.0), 
                    vec4(0.0,  c,   s,   0.0), 
                    vec4(0.0, -s,   c,   0.0), 
                    vec4(0.0,  0.0, 0.0, 1.0));
  
  s = sin(rotation.y);
  c = cos(rotation.y);
  
  mat4 rot_y = mat4(vec4(c,   0.0, -s,   0.0), 
                    vec4(0.0, 1.0,  0.0, 0.0), 
                    vec4(s,   0.0,  c,   0.0), 
                    vec4(0.0, 0.0,  0.0, 1.0));
  
  s = sin(rotation.z);
  c = cos(rotation.z);
  
  mat4 rot_z = mat4(vec4(c,   s,   0.0, 0.0), 
                    vec4(-s,  c,   0.0, 0.0), 
                    vec4(0.0, 0.0, 1.0, 0.0), 
                    vec4(0.0, 0.0, 0.0, 1.0));
  
  mat4 rotation_matrix = rot_y*rot_z*rot_x;
  
  return rotation_matrix;
}*/

mat4 create_translation_matrix(vec3 pos) {
  mat4 translate_matrix = mat4(vec4(1.0, 0.0, 0.0, 0.0), 
                               vec4(0.0, 1.0, 0.0, 0.0), 
                               vec4(0.0, 0.0, 1.0, 0.0), 
                               vec4(pos,           1.0));
  
  return translate_matrix;
}

mat4 create_scale_matrix(vec3 scale) {
  mat4 scale_matrix = mat4(vec4(scale.x, 0.0,     0.0,     0.0), 
                               vec4(0.0,     scale.y, 0.0,     0.0), 
                               vec4(0.0,     0.0,     scale.z, 0.0), 
                               vec4(0.0,     0.0,     0.0,     1.0));
  
  return scale_matrix;
}

void main() {
  vec3 model_scale = vec3(push_constants.c_up.w, push_constants.model.w, push_constants.rotation.w);
  
  mat4 projection = create_perspective_matrix(push_constants.c_position.w, push_constants.c_center.w, 0.1, 1080.0);
  mat4 view = create_view_matrix(push_constants.c_position.xyz, push_constants.c_center.xyz, push_constants.c_up.xyz);
  mat4 model = create_translation_matrix(push_constants.model.xyz);
  mat4 scale = create_scale_matrix(model_scale);
  vec3 rotation = push_constants.rotation.xyz;
  
  vec3 local_pos = vec3(model * scale * vec4(rotate_vector_by_angle(position, rotation), 1.0));
  
  vec4 rotated_normal = vec4(rotate_vector_by_angle(vec3(-normal.x, normal.y, normal.z), rotation), 1.0);
  
  uvs = uv;
  v_colour = colour;
  v_alpha_cutoff = vec4(uniforms.emissive_alpha.z, uniforms.emissive_alpha.w, 0.0, uniforms.emissive_alpha.x);
  v_base_colour_factor = uniforms.base_colour_factor;
  v_world_pos = local_pos;
  v_normal = rotated_normal.xyz;
  
  v_camera_pos = push_constants.c_position.rgb;
  
  v_use_textures = uniforms.use_textures;
  v_scanline = vec3(push_constants.hologram_scanline.y, local_pos.y, push_constants.hologram_scanline.x);
  v_mr = vec2(uniforms.mro_factors.x, uniforms.mro_factors.y);
  
  gl_Position = projection * view * vec4(local_pos, 1.0);
}
