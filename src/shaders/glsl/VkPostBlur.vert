#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec2 dir;

layout(set = 0, binding = 0) uniform Data {
  mat4 projection;
  mat4 model;
  vec2 direction;
} uniforms;

void main() {
  
  uvs = uv;
  gl_Position = uniforms.projection * uniforms.model * vec4(position, 0.0, 1.0);
}
