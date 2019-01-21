#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

// Instanced.
layout(location = 2) in mat4 model;
layout(location = 6) in vec4 colour;
layout(location = 7) in vec4 sprite_sheet; // block_x, block_y, num_of_rows, image_scale
layout(location = 8) in vec4 has_texture_blackwhite;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 new_colour;
layout(location = 2) out vec2 textured_blackwhite;

// 128 bytes, float 4 bytes
layout(push_constant) uniform PushConstants {
  mat4 projection;
} push_constants;

void main() {
  float num_rows = sprite_sheet.z;
  float block_x = sprite_sheet.x;
  float block_y = sprite_sheet.y;
  float scale = sprite_sheet.w;
  
  vec2 texcoords = uv.xy;
  texcoords += vec2(block_x, block_y);
  texcoords /= num_rows;
  uvs = texcoords;
  
  new_colour = colour;
  
  textured_blackwhite = has_texture_blackwhite.xy;
  
  mat4 scale_matrix = mat4(vec4(scale, 0.0, 0.0, 0.0), 
                           vec4(0.0, scale, 0.0, 0.0), 
                           vec4(0.0, 0.0, scale, 0.0), 
                           vec4(0.0, 0.0, 0.0, 1.0));
  
  gl_Position = push_constants.projection * scale_matrix * model * vec4(position, 0.0, 1.0);
}
