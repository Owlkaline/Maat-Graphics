#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 colours;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  vec4 tex  = texture(tex, uvs);
  
  outColour = vec4(vec3(colours.rgb*tex.rgb), colours.a*tex.a);// mix(vec4(0.45, 0.55, 0.60, 1.0), 
}
