#version 450

layout (location = 0) out vec4 outFragColor;

layout (location = 0) in vec3 colours;

void main() {
  outFragColor = vec4(colours, 1.0);
}
