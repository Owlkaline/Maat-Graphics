#version 330 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

out vec2 uvs;
out vec2 dir;

uniform mat4 projection;
uniform mat4 model;

void main() {
  uvs = uv;
  gl_Position = projection * model * vec4(position, 0.0, 1.0);
}