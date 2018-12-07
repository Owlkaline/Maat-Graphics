#version 450

layout (location = 0) out vec4 outFragColor;

layout (location = 0) in vec3 colour;

void main() {
  outFragColor = vec4(colour, 1.0);
}
