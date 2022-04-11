#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (set = 1, binding = 0) uniform sampler2D samplerColour;

layout (location = 0) in vec4 o_colour;
layout (location = 1) in vec4 o_uv_textured_mix;

layout (location = 0) out vec4 uFragColor;

void main() {
  vec4 texture_colour = texture(samplerColour, o_uv_textured_mix.xy);
  vec4 plain_colour = o_colour;
  float mix_amount = o_uv_textured_mix.w;
  
  //uFragColor = vec4(texture_colour.rgb, 1.0);
  uFragColor = vec4(mix(plain_colour, texture_colour, mix_amount).rgb, plain_colour.a);
  //uFragColor = mix(plain_colour, texture_colour, texture_alpha)*overall_alpha;
}
