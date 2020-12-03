#version 450

layout(location = 0) in vec2 v_uvs;
layout(location = 1) in vec4 v_new_colour;
layout(location = 2) in vec3 v_outline_colour;
layout(location = 3) in vec4 v_edge_width;

layout(location = 0) out vec4 colour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  float distance = 1.0 - texture(tex, v_uvs).a;
  float alpha = 1.0 - smoothstep(v_edge_width.x, v_edge_width.x + v_edge_width.y, distance);
  
  float distance2 = 1.0 - texture(tex, v_uvs).a;
  float outlineAlpha = 1.0 - smoothstep(v_edge_width.z, v_edge_width.z+v_edge_width.w, distance2);
  
  float overallAlpha = alpha + (1.0 - alpha) * outlineAlpha;
  
  if (overallAlpha == 0.0 || v_new_colour.w == 0.0) {
    discard;
  }
  
  vec3 overallColour = mix(v_outline_colour, v_new_colour.rgb, alpha / overallAlpha);
  
  colour = vec4(overallColour, overallAlpha*v_new_colour.w);
}
