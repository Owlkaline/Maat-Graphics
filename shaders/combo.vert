#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 colour;
layout (location = 2) in vec2 uv;

layout (location = 0) out vec4 o_colour;
layout (location = 1) out vec4 o_uv_textured_mix;

layout (set = 0, binding = 0) uniform UBO {
  vec2 window_size;
} ubo;

layout(push_constant) uniform PushConstants {
  vec4 pos_scale; // x, y, scale_x, scale_y
  vec4 colour;  // r g b a
  vec4 is_textured_rotation_overlay_mix; // is_textured, rotation, overlay_mix, empty
  vec4 sprite_sheet; // rows, texture number, empty
  vec4 flip_xy; // flip x y
  vec4 attrib5; 
  vec4 attrib6;
  vec4 attrib7;
} push_constants;

mat4 ortho_projection(float bottom, float top, float left, float right, float near, float far) {
  mat4 projection = mat4(
    vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
    vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
    vec4(0.0, 0.0, -2.0 / (far - near), 0.0),
    vec4(-(right + left) / (right - left), -(top + bottom) / (top - bottom), -(far + near)/(far - near), 1)
  );
  
  return projection;
}

void main() {
  int rows = int(push_constants.sprite_sheet.x);
  int idx = int(push_constants.sprite_sheet.y);
  float flip_x = push_constants.flip_xy.x;
  float flip_y = push_constants.flip_xy.y;

  float flipped_uvx = (flip_x*-1.0 + uv.x);
  float flipped_uvy = (flip_y*-1.0 + uv.y);

  vec2 new_uv_coords = vec2(flipped_uvx / rows, flipped_uvy / rows);
  float column = idx % rows;
  float x_offset = column / rows;

  float row = floor(idx / rows);
  float y_offset = row / rows;

  float uvx = new_uv_coords.x + x_offset;
  float uvy = new_uv_coords.y + y_offset;

  //float column = 1.0 - mod(idx, rows);

  //float x_offset = idx / coloum;
  //float column = (idx % columns);
  //float row = floor(idx / rows);

  //float x_offset = column / columns;
  //float y_offset = row / rows;


  //float uvx = flipped_uvx / column + x_offset;
  //float uvy = flipped_uvy / rows + y_offset;

  o_uv_textured_mix = vec4(uvx, uvy, push_constants.is_textured_rotation_overlay_mix.xz);
  o_colour = push_constants.colour;
  
  mat4 ortho_matrix = ortho_projection(0.0, ubo.window_size.y, 0.0, ubo.window_size.x, 0.1, 1.0);
                                                                 //720.0, 0.0, 1280.0, 0.1, 1.0);
  
  vec2 unrotated_pos = pos.xy - vec2(0.5, 0.5);
  float rotation = radians(push_constants.is_textured_rotation_overlay_mix.y);
  
  unrotated_pos.x *= push_constants.pos_scale.z;
  unrotated_pos.y *= push_constants.pos_scale.w;
  
  mat2 rot_mat = mat2(vec2(cos(rotation), -sin(rotation)),
                      vec2(sin(rotation), cos(rotation)));
  
  vec2 rotated_pos = rot_mat * unrotated_pos;
  
  rotated_pos += push_constants.pos_scale.zw*0.5;
  
  float x = rotated_pos.x + push_constants.pos_scale.x;
  float y = rotated_pos.y + push_constants.pos_scale.y;
  
  gl_Position = ortho_matrix * vec4(x, y, pos.zw);
}
