#version 330 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;
layout (location = 2) in mat4 model;
layout (location = 6) in vec4 new_colour;
layout (location = 7) in float has_texture;

out vec2 uvs;
out vec4 colour;
out float new_texture;

uniform mat4 projection;

void main() {
  uvs = vec2(1.0-uv.x, uv.y);
  colour = new_colour;
  new_texture = has_texture;
  gl_Position = projection * model * vec4(position, 0.0f, 1.0f);
}
