#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in mat4 model;
layout(location = 6) in vec4 colour;
layout(location = 7) in vec4 sprite_sheet; // block_x, block_y, num_of_rows
layout(location = 8) in vec4 has_texture_blackwhite;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 new_colour;
layout(location = 2) out vec2 textured_blackwhite;

layout(set = 0, binding = 1) uniform Data {
  mat4 projection;
  mat4 scale;
} uniforms;

void main() {
  float num_rows = sprite_sheet.z;
  float block_x = sprite_sheet.x;
  float block_y = sprite_sheet.y;
  
  vec2 texcoords = uv.xy;
  texcoords += vec2(block_x, block_y);
  texcoords /= num_rows;
  uvs = texcoords;
  
  new_colour = colour;
  
  textured_blackwhite = has_texture_blackwhite.xy;
  
  gl_Position = uniforms.projection * uniforms.scale * model * vec4(position, 0.0, 1.0);
}
