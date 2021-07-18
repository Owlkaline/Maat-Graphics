#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 colour;
layout (location = 2) in vec2 uv;

layout (location = 0) out vec4 o_colour;
layout (location = 1) out vec3 o_uv_textured;

layout (set = 0, binding = 0) uniform UBO {
  vec3 window_size;
} ubo;

layout(push_constant) uniform PushConstants {
  vec4 pos_scale; // x, y, scale_x, scale_y
  vec4 colour;  // r g b a
  vec4 is_textured_rotation; // is_textured, rotation, empty x2
  vec4 attrib3;
  vec4 attrib4;
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
  o_uv_textured = vec3(uv, push_constants.is_textured_rotation.x);
  o_colour = push_constants.colour;
  
  mat4 ortho_matrix = ortho_projection(0.0, ubo.window_size.y, 0.0, ubo.window_size.x, 0.1, 1.0);
                                                                 //720.0, 0.0, 1280.0, 0.1, 1.0);
  
  vec2 unrotated_pos = pos.xy - vec2(0.5, 0.5);
  float rotation = radians(push_constants.is_textured_rotation.y);
  
  unrotated_pos.x *= push_constants.pos_scale.z;
  unrotated_pos.y *= push_constants.pos_scale.w;
  
  mat2 rot_mat = mat2(vec2(cos(rotation), -sin(rotation)),
                      vec2(sin(rotation), cos(rotation)));
  
  vec2 rotated_pos = rot_mat * unrotated_pos;
  
  rotated_pos += push_constants.pos_scale.zw*0.5;
  
  float x = rotated_pos.x + push_constants.pos_scale.x;
  float y = rotated_pos.y + push_constants.pos_scale.y;
  
  //vec2 rotated_scale = rot_mat * push_constants.pos_scale.zw;
  
  //rotated_pos += vec2(0.5, 0.5);
  
  //float x = (rotated_pos.x*rotated_scale.x) + push_constants.pos_scale.x;
  //float y = (rotated_pos.y*rotated_scale.y) + push_constants.pos_scale.y;
  
  gl_Position = ortho_matrix * vec4(x, y, pos.zw);
}
