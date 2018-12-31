#version 450

layout(location = 0) in vec2 positions;
layout(location = 1) in vec3 colour;

layout(location = 0) out vec3 colours;

layout (binding = 0) uniform Uniform {
  vec2 translation;
} uniforms;

void main() {
  colours = colour;
  gl_Position = vec4(positions + uniforms.translation, 0.0, 1.0);
}
