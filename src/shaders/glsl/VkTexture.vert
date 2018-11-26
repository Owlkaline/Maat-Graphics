#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 new_colour;
layout(location = 2) out vec2 textured_blackwhite;

layout(set = 0, binding = 1) uniform Data {
  mat4 projection;
  mat4 scale;
} uniforms;

layout(set = 0, binding = 2) uniform DrawData {
  mat4 model;
  vec4 colour;
  vec4 sprite_sheet; // block_x, block_y, num_of_rows
  vec4 has_texture_blackwhite;
} draw_uniforms;

void main() {
  float num_rows = draw_uniforms.sprite_sheet.z;
  float block_x = draw_uniforms.sprite_sheet.x;
  float block_y = draw_uniforms.sprite_sheet.y;
  
  vec2 texcoords = uv.xy;
  texcoords += vec2(block_x, block_y);
  texcoords /= num_rows;
  uvs = texcoords;
  
  new_colour = draw_uniforms.colour;
  
  textured_blackwhite = draw_uniforms.has_texture_blackwhite.xy;
  
  gl_Position = uniforms.projection * uniforms.scale * draw_uniforms.model * vec4(position, 0.0, 1.0);
}
