#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 new_colour;
layout(location = 2) out vec2 textured_blackwhite;

layout(binding = 0) uniform Data {
  mat4 projection;
  mat4 scale;
} uniforms;

// 128 bytes, float 4 bytes
layout(push_constant) uniform PushConstants {
  mat4 model;
  vec4 colour;
  vec4 sprite_sheet; // block_x, block_y, num_of_rows
  vec4 has_texture_blackwhite;
} push_constants;

void main() {
  float num_rows = push_constants.sprite_sheet.z;
  float block_x = push_constants.sprite_sheet.x;
  float block_y = push_constants.sprite_sheet.y;
  
  vec2 texcoords = uv.xy;
  texcoords += vec2(block_x, block_y);
  texcoords /= num_rows;
  uvs = texcoords;
  
  new_colour = push_constants.colour;
  
  textured_blackwhite = push_constants.has_texture_blackwhite.xy;
  
  gl_Position = uniforms.projection * uniforms.scale * push_constants.model * vec4(position, 0.0, 1.0);
}
