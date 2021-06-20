#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 colour;
layout (location = 2) in vec2 uv;

layout (location = 0) out vec4 o_colour;
layout (location = 1) out vec3 o_uv_textured;

layout(push_constant) uniform PushConstants {
  vec4 offset_textured;
  vec4 colour; 
  vec4 attrib2; 
  vec4 attrib3;
  vec4 attrib4;
  vec4 attrib5; 
  vec4 attrib6;
  vec4 attrib7;
} push_constants;

void main() {
  o_uv_textured = vec3(uv, push_constants.offset_textured.z);
  o_colour = push_constants.colour;
  
  gl_Position = pos + vec4(push_constants.offset_textured.xy, vec2(0.0));
}
