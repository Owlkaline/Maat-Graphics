#version 330 core

in vec2 position;
in vec2 uv;

out vec2 uvs;

uniform mat4 projection;
uniform mat4 model;

void main() {
  uvs = uv;
  gl_Position = projection * model * vec4(position.x*-1.0, position.y, 0.0, 1.0);
}
