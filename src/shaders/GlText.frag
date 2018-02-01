#version 330 core

in vec2 uvs;

out vec4 outColour;

uniform sampler2D tex;
uniform vec4 colour;
uniform vec4 edge_width;
uniform vec3 outlineColour;

void main() {
  float distance = 1.0 - texture(tex, uvs).a;
  float alpha = 1.0 - smoothstep(edge_width.x, edge_width.x + edge_width.y, distance);
  
  float distance2 = 1.0 - texture(tex, uvs).a;
  float outlineAlpha = 1.0 - smoothstep(edge_width.z, edge_width.z+edge_width.w, distance2);
  
  float overallAlpha = alpha + (1.0 - alpha) * outlineAlpha;
  vec3 overallColour = mix(outlineColour, colour.rgb, alpha / overallAlpha);
  
  outColour = vec4(overallColour, overallAlpha);
}
