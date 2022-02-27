#version 450

layout (set = 1, binding = 0) uniform sampler2D samplerColour;

layout(location = 0) in vec2 o_uv;
layout(location = 1) in vec4 o_colour;
//layout(location = 2) in vec4 o_outline_colour;
//layout(location = 3) in vec4 o_outline_width;

layout(location = 0) out vec4 uFragColour;

const float smoothing = 1.0/16.0;

void main() {
  float distance = texture(samplerColour, o_uv).a;
  float alpha = smoothstep(0.5 - smoothing, 0.5 + smoothing, distance);

  uFragColour = vec4(o_colour.rgb, o_colour.a * alpha);

  //float distance = 1.0 - texture(samplerColour, o_uv).a;
  //float alpha = 1.0 - smoothstep(o_outline_width.x, o_outline_width.x + o_outline_width.y, distance);
  //
  //float distance2 = 1.0 - texture(samplerColour, o_uv).a;
  //float outlineAlpha = 1.0 - smoothstep(o_outline_width.z, o_outline_width.z+o_outline_width.w, distance2);
  //
  //float overallAlpha = alpha + (1.0 - alpha) * outlineAlpha;
  //
  //vec3 overallColour = mix(o_outline_colour.rgb, o_colour.rgb, alpha / overallAlpha);
  //
  ////if (overallAlpha == 0.0 || v_new_colour.w == 0.0) {
  ////  discard;
  ////}
  //
  //uFragColor = vec4(overallColour, overallAlpha*o_colour.w);
}
