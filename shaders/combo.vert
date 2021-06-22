#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 colour;
layout (location = 2) in vec2 uv;

layout (location = 0) out vec4 o_colour;
layout (location = 1) out vec3 o_uv_textured;

layout (binding = 1) uniform UBO {
  vec3 window_size;
} ubo;

layout(push_constant) uniform PushConstants {
  vec4 pos_scale; // x, y, scale_x, scale_y
  vec4 colour;  // r g b a
  vec4 is_textured; // is_textured, empty x3 
  vec4 attrib3;
  vec4 attrib4;
  vec4 attrib5; 
  vec4 attrib6;
  vec4 window_size; // empty x2 width height 
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
  o_uv_textured = vec3(uv, push_constants.is_textured.x);
  o_colour = push_constants.colour;
  
  mat4 ortho_matrix = ortho_projection(0.0, push_constants.window_size.w, 0.0, push_constants.window_size.z, 0.1, 1.0);
                                                                 //720.0, 0.0, 1280.0, 0.1, 1.0);
  
  float x = (pos.x*push_constants.pos_scale.z) + push_constants.pos_scale.x;
  float y = (pos.y*push_constants.pos_scale.w) + push_constants.pos_scale.y;
  
  gl_Position = ortho_matrix * vec4(x, y, pos.zw);
}
