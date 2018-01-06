#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 new_colour;

layout(location = 0) out vec4 colour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  if(new_colour.w == -1) {
    colour = texture(tex, uvs);
  } else {
    colour = new_colour;
  }
}
