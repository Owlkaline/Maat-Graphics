#version 450

layout(location = 0) in vec2 positions;
layout(location = 1) in vec3 colour;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 colours;
layout(location = 1) out vec2 uvs;

layout (binding = 0) uniform Uniform {
  vec2 translation;
} uniforms;

layout (push_constant) uniform PushConstants {
  vec2 translation;
} push_constants;

void main() {
  colours = colour;
  uvs = uv;
  gl_Position = vec4(positions + push_constants.translation, 0.0, 1.0);
}
