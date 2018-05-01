#version 330 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;

out vec2 uvs;

uniform vec4 letter_uv;
uniform mat4 model;
uniform mat4 projection;

void main() {
  vec2 new_uv = uv;
  vec2 new_pos = position;
  
  if(uv.x == 0) {
    new_uv.x += letter_uv.x;
    new_pos.x = 0;
  } else
  if(uv.x == 1) {
    new_uv.x = letter_uv.z;
    new_pos.x = letter_uv.z - letter_uv.x;
  }
  
  if(uv.y == 0) {
    new_uv.y = letter_uv.w;
    new_pos.y = 0;
  } else
  if(uv.y == 1) {
    new_uv.y = letter_uv.y;
    new_pos.y = letter_uv.w - letter_uv.y;
  }
  
  gl_Position = projection * model * vec4(new_pos, 0.0, 1.0);
  uvs = new_uv;
}