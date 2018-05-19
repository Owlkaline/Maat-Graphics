#version 330 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;
layout (location = 2) in mat4 model;
layout (location = 6) in vec4 new_colour;
layout (location = 7) in vec2 textured_blackwhite;

out vec2 uvs;
out vec4 colour;
out float textured;
out float blackwhite;

uniform mat4 projection;

void main() {
  uvs = vec2(1.0-uv.x, uv.y);
  colour = new_colour;
  textured = textured_blackwhite.x;
  blackwhite = textured_blackwhite.y;
  gl_Position = projection * model * vec4(position, 0.0f, 1.0f);
}
