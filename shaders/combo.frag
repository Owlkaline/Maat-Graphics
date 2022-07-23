#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (set = 1, binding = 0) uniform sampler2D samplerColour;

layout (location = 0) in vec4 o_colour;
layout (location = 1) in vec4 o_uv;
layout (location = 2) in vec3 o_overlay_colour;

layout (location = 0) out vec4 uFragColor;

void main() {
  vec4 texture_colour = texture(samplerColour, o_uv.xy);
  
  uFragColor = vec4(texture_colour.rgb + o_overlay_colour*texture_colour.a, texture_colour.a);
}
