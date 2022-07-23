#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (set = 1, binding = 0) uniform sampler2D samplerColour;

layout (location = 0) in vec4 o_colour;
layout (location = 1) in vec4 o_uv;
layout (location = 2) in vec3 o_overlay_colour;

layout (location = 0) out vec4 uFragColor;

float rand(vec2 co) {
  return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
  vec4 texture_colour = texture(samplerColour, o_uv.xy);
  //float intensity = 0.2;
  ////texture_colour.rgb *= rand(o_uv.xy);
  ////
  //float rnd = 0.2;
  //float f = rand(vec2(int(gl_FragCoord.y/3*rnd), int(gl_FragCoord.x/3*rnd)))*rand(vec2(int(gl_FragCoord.y/3*rnd), int(gl_FragCoord.x/3*rnd)));
  //float c1 = texture_colour.r + texture_colour.b + texture_colour.g;

  //float col = (1.0 - min(c1, 0.5)) * f;
  //vec4 colour = vec4(-col*0.3, col*0.9f, col, 0) * intensity;

  //texture_colour += colour;
  
  uFragColor = vec4(texture_colour.rgb + pow(o_overlay_colour*texture_colour.a, vec3(2.2)), texture_colour.a);
}
