#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec4 colour;
layout(location = 4) in vec4 tangent;

// Instanced Data
layout(location = 5) in vec4 model; // x, y, z, x_scale
layout(location = 6) in vec4 rotation; // x_rot, y_rot, z_rot, y_scale
layout(location = 7) in vec4 overwrite_colour; // r, g, b, a
layout(location = 8) in vec4 hologram_scanline; // hologram_enabled, scanline, _, z_scale

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 v_colour;
layout(location = 2) out vec4 v_base_colour_factor;
layout(location = 3) out vec4 v_alpha_cutoff;
layout(location = 4) out vec3 v_normal;
layout(location = 5) out vec3 v_to_light[2];
layout(location = 7) out vec3 v_scanline;
layout(location = 8) out vec4 v_use_textures;
layout(location = 9) out vec4 v_overwrite_colour;

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
  vec4 c_up;       // x, y, z, 
} push_constants;

const float M_PI = 3.141592653589793;

const vec3 sun_pos = vec3(-40.0, 20.0, -40.0);
const vec3 light2 = vec3(40.0, 20.0, 40.0);

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
  
  mat4 rotation_matrix = rot_y*rot_x*rot_z;
  
  return rotation_matrix;
}

mat4 create_translation_matrix(vec3 pos) {
  mat4 translate_matrix = mat4(vec4(1.0, 0.0,     0.0,     0.0), 
                               vec4(0.0,     1.0, 0.0,     0.0), 
                               vec4(0.0,     0.0,     1.0, 0.0), 
                               vec4(pos,                       1.0));
  
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
  vec3 model_scale = vec3(model.w, rotation.w, hologram_scanline.w);
  
  mat4 projection = create_perspective_matrix(push_constants.c_position.w, push_constants.c_center.w, 0.1, 1024.0);
  mat4 view = create_view_matrix(push_constants.c_position.xyz, push_constants.c_center.xyz, push_constants.c_up.xyz);
  mat4 model = create_translation_matrix(model.xyz);
  mat4 scale = create_scale_matrix(model_scale);
  mat4 rotation = create_rotation_matrix(rotation.xyz);
  
  vec3 local_pos = vec3(model * rotation * scale * vec4(position, 1.0));
  
  vec4 rotated_normal = /*model */ rotation * vec4(-normal.x, normal.y, normal.z, 1.0);
  
  uvs = uv;
  v_colour = colour;
  v_alpha_cutoff = vec4(uniforms.emissive_alpha.z, uniforms.emissive_alpha.w, 0.0, 0.0);
  v_base_colour_factor = uniforms.base_colour_factor;
  v_normal = rotated_normal.xyz;
  v_to_light[0] = sun_pos - local_pos;
  v_to_light[1] = light2 - local_pos;
  
  v_use_textures = uniforms.use_textures;
  v_scanline = vec3(hologram_scanline.y, local_pos.y, hologram_scanline.x);
  v_overwrite_colour = overwrite_colour;
  
  gl_Position = projection * view * vec4(local_pos, 1.0);
}
