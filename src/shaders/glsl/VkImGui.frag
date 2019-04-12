#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 colours;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  outColour = colours * texture(tex, uvs);
}
