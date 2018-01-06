#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 new_colour;
layout(location = 2) in vec3 outlineColour;
layout(location = 3) in vec4 edge_width;

layout(location = 0) out vec4 colour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  float distance = 1.0 - texture(tex, uvs).a;
  float alpha = 1.0 - smoothstep(edge_width.x, edge_width.x + edge_width.y, distance);
  
  float distance2 = 1.0 - texture(tex, uvs).a;
  float outlineAlpha = 1.0 - smoothstep(edge_width.z, edge_width.z+edge_width.w, distance2);
  
  float overallAlpha = alpha + (1.0 - alpha) * outlineAlpha;
  
  vec3 overallColour = mix(outlineColour, new_colour.rgb, alpha / overallAlpha);
  
  colour = vec4(overallColour, overallAlpha);
}
