#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (binding = 1) uniform sampler2D samplerColour;

layout (binding = 0) uniform UBO {
  vec3 color;
} ubo;

layout (location = 0) in vec4 o_colour;
layout (location = 1) in vec2 o_uv;

layout (location = 0) out vec4 uFragColor;

void main() {
  vec4 texture_colour = texture(samplerColour, o_uv);
  vec4 plain_colour = o_colour;
  
  float texture_alpha = 0.5; // 1.0 only texture, 0.0 only colour 
  
  uFragColor = mix(plain_colour, texture_colour, texture_alpha);
}
