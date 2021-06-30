#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (set = 1, binding = 0) uniform sampler2D samplerColour;

layout (location = 0) in vec4 o_colour;
layout (location = 1) in vec3 o_uv_textured;

layout (location = 0) out vec4 uFragColor;

void main() {
  float overall_alpha = o_colour.a;
  vec4 texture_colour = texture(samplerColour, o_uv_textured.xy);
  vec4 plain_colour = o_colour;
  
  float texture_alpha = o_uv_textured.z; // 1.0 only texture, 0.0 only colour 
  
  uFragColor = mix(plain_colour, texture_colour, texture_alpha)*overall_alpha;
}
