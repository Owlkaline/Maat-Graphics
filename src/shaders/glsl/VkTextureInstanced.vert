#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

// Instanced.
layout(location = 2) in vec4 model;//x y scaley scaley
layout(location = 4) in vec4 sprite_sheet; // block_x, block_y, num_of_rows, rotation

layout(location = 0) out vec2 uvs;

// 128 bytes, float 4 bytes
layout(push_constant) uniform PushConstants {
  vec4 projection; // 
  vec4 matrix_zoom;
} push_constants;

const float M_PI = 3.141592653589793;

float to_radians(float degree) {
  return degree * (M_PI/180.0);
}

mat4 create_scale_matrix(float scale) {
  mat4 matrix_scale = mat4(vec4(scale,  0.0,   0.0, 0.0), 
                    vec4(0.0, scale, 0.0, 0.0), 
                    vec4(0.0, 0.0,   1.0, 0.0), 
                    vec4(0.0, 0.0,   0.0, 1.0));
  
  return matrix_scale;
}

mat4 create_rotation_matrix(float rotation) {
  float s = sin(to_radians(rotation));
  float c = cos(to_radians(rotation));
  
  mat4 rot_z = mat4(vec4(c,   s,   0.0, 0.0), 
                    vec4(-s,  c,   0.0, 0.0), 
                    vec4(0.0, 0.0, 1.0, 0.0), 
                    vec4(0.0, 0.0, 0.0, 1.0));
  return rot_z;
}

mat4 create_translation_matrix(vec2 pos, vec2 scale) {
  mat4 translate_matrix = mat4(vec4(scale.x, 0.0,   0.0, 0.0), 
                               vec4(0.0,   scale.y, 0.0, 0.0), 
                               vec4(0.0,   0.0,   1.0, 0.0), 
                               vec4(pos,          0.0, 1.0));
  
  return translate_matrix;
}

mat4 create_ortho_projection(float near, float far, float right, float bottom, vec2 offset) {
  float left = offset.x;
  float top = offset.y;
  right += left;
  bottom += top;
  
  mat4 ortho = mat4(vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
                    vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
                    vec4(0.0, 0.0, -2.0 / (near / far), 0.0),
                    vec4(-(right + left) / (right - left), -(top+bottom)/(top-bottom), 0.0, 1.0));
  
  return ortho;
}

void main() {
  float num_rows = sprite_sheet.z;
  float block_x = sprite_sheet.x;
  float block_y = sprite_sheet.y;
  float matrix_zoom = push_constants.matrix_zoom.x;
  
  vec2 texture_pos = model.xy; 
  vec2 texture_scale = model.zw;
  float rotation = sprite_sheet.w;
  
  if (num_rows < 0.0) {
    num_rows *= -1;
  }
  
  vec2 texcoords = uv.xy;
  texcoords += vec2(block_x, block_y);
  texcoords /= num_rows;
  uvs = texcoords;
  
  float x_offset = push_constants.projection.x;
  float y_offset = push_constants.projection.y;
  float near = 1.0;
  float far = -1.0;
  float right = push_constants.projection.z;
  float bottom = push_constants.projection.w;
  mat4 projection = create_ortho_projection(near, far, right, bottom, vec2(x_offset, y_offset));
  
  mat4 rot_z = create_rotation_matrix(rotation);
  
  mat4 model_matrix = create_translation_matrix(texture_pos, texture_scale);
                              
  mat4 scale_matrix = create_scale_matrix(matrix_zoom);
  
  gl_Position = projection * model_matrix * rot_z * scale_matrix * vec4(position, 0.0, 1.0);
}
