#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;

layout(set = 0, binding = 0) uniform Data {
  mat4 projection;
} uniforms;

layout(push_constant) uniform PushConstants {
  mat4 model;
} push_constants;

void main() {
  uvs = uv;
  gl_Position = uniforms.projection * push_constants.model * vec4(position, 0.0, 1.0);
}
