#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex2d;

void main() {
  vec4 colour2d = texture(tex2d, uvs);
  
  outColour = colour2d;
}
