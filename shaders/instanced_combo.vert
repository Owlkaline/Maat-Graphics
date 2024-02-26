#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

// vertex
layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 colour;
layout (location = 2) in vec4 uv;

// instanced data
layout (location = 3) in vec4 pos_scale; // x, y, scale_x, scale_y
layout (location = 4) in vec4 other_colour;  // r g b a
layout (location = 5) in vec4 is_textured_rotation_overlay_mix; // is_textured, rotation, overlay_mix, empty
layout (location = 6) in vec4 sprite_sheet; // rows, texture number, empty
layout (location = 7) in vec4 flip_xy; // flip x y
layout (location = 8) in vec4 overlay_colour; // overlay colour 
layout (location = 9) in vec4 attrib6;
layout (location = 10) in vec4 camera_intensity_time; // camera x y, intensity time

// output to fragment
layout (location = 0) out vec4 o_colour;
layout (location = 1) out vec4 o_uv;
layout (location = 2) out vec4 o_overaly_colour;
layout (location = 3) out vec4 time_intensity;

layout(push_constant) uniform PushConstants {
  vec2 window_size;
} ubo;

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
  int rows = int(sprite_sheet.x);
  int idx = int(sprite_sheet.y);
  float flip_x = flip_xy.x;
  float flip_y = flip_xy.y;

  float flipped_uvx = (flip_x*-1.0 + uv.x);
  float flipped_uvy = (flip_y*-1.0 + uv.y);

  vec2 new_uv_coords = vec2(flipped_uvx / rows, flipped_uvy / rows);
  float column = (idx % rows);
  float x_offset = column / rows;

  float row = floor(idx / rows);
  float y_offset = row / rows;

  float uvx = new_uv_coords.x + x_offset;
  float uvy = new_uv_coords.y + y_offset;

  o_uv = vec4(uvx, uvy, 0.0, 0.0);
  o_colour = other_colour;
  o_overaly_colour = vec4(overlay_colour.rgb, 0.0);
  time_intensity = vec4(camera_intensity_time.w, camera_intensity_time.z, 0.0, 0.0);
  
  mat4 ortho_matrix = ortho_projection(0.0, ubo.window_size.y, 0.0, ubo.window_size.x, 0.1, 1.0);
                                                                 //720.0, 0.0, 1280.0, 0.1, 1.0);
  
  vec2 unrotated_pos = pos.xy - vec2(0.5, 0.5);
  float rotation = radians(is_textured_rotation_overlay_mix.y);
  
  unrotated_pos.x *= pos_scale.z;
  unrotated_pos.y *= pos_scale.w;
  
  mat2 rot_mat = mat2(vec2(cos(rotation), -sin(rotation)),
                      vec2(sin(rotation), cos(rotation)));
  
  vec2 rotated_pos = rot_mat * unrotated_pos;
  
  rotated_pos += pos_scale.zw*0.5;

  vec2 camera_pos = camera_intensity_time.xy;

  float x = rotated_pos.x + pos_scale.x + camera_pos.x;
  float y = rotated_pos.y + pos_scale.y + camera_pos.y;
  
  gl_Position = ortho_matrix * vec4(x, y, pos.zw);
}

