#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 new_colour;
layout(location = 2) out vec2 textured_blackwhite;

// 128 bytes, float 4 bytes
layout(push_constant) uniform PushConstants {
  mat4 model;
  vec4 colour;
  vec4 sprite_sheet; // block_x, block_y, num_of_rows, image_scale
  vec4 has_texture_blackwhite_width_height;
} push_constants;

void main() {
  float num_rows = push_constants.sprite_sheet.z;
  float block_x = push_constants.sprite_sheet.x;
  float block_y = push_constants.sprite_sheet.y;
  float scale = push_constants.sprite_sheet.w;
  
  vec2 texcoords = uv.xy;
  texcoords += vec2(block_x, block_y);
  texcoords /= num_rows;
  uvs = texcoords;
  
  new_colour = push_constants.colour;
  
  textured_blackwhite = push_constants.has_texture_blackwhite_width_height.xy;
  
  mat4 scale_matrix = mat4(vec4(scale, 0.0, 0.0, 0.0), 
                           vec4(0.0, scale, 0.0, 0.0), 
                           vec4(0.0, 0.0, scale, 0.0), 
                           vec4(0.0, 0.0, 0.0, 1.0));
  
  float near = 1.0;
  float far = -1.0;
  float right = push_constants.has_texture_blackwhite_width_height.z;
  float left = 0.0;
  float bottom = push_constants.has_texture_blackwhite_width_height.w;
  float top = 0.0;
  mat4 projection = mat4(vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
                          vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
                          vec4(0.0, 0.0, -2.0 / (near / far), 0.0),
                          vec4(-(right + left) / (right - left), -(top+bottom)/(top-bottom), 0.0, 1.0));
  
  gl_Position = projection * scale_matrix * push_constants.model * vec4(position, 0.0, 1.0);
}
