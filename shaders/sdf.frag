#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (set = 1, binding = 0) uniform sampler2D samplerColour;

layout (location = 0) in vec2 o_uv;
layout (location = 1) in vec4 o_outline_colour;
layout (location = 2) in vec2 o_outline_width;

layout (location = 0) out vec4 uFragColor;

void main() {
  float distance = texture(samplerColour, o_uv).a;
  float smooth_width = fwidth(distance);
  float alpha = smoothstep(0.5 - smooth_width, 0.5 + smooth_width, distance);
  
  vec3 rgb = vec3(alpha);
  if (o_outline_width.x > 0.0) {
    float w = 1.0 - o_outline_width.y;
    alpha = smoothstep(w - smooth_width, w + smooth_width, distance);
    rgb += mix(vec3(alpha), o_outline_colour.rgb, alpha);
  }
  
  uFragColor = vec4(rgb, alpha);
}
