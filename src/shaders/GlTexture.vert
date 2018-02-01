#version 330 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;

out vec2 uvs;

uniform mat4 model;
uniform mat4 projection;

void main() {
  uvs = uv;
  gl_Position = projection * model * vec4(position, 0.0f, 1.0f);
}
