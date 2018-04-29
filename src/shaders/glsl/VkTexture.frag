#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 new_colour;
layout(location = 2) in vec2 has_texture;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  vec4 drawTexture = new_colour;
  if(has_texture.x == 1.0)
    drawTexture = texture(tex, uvs);
  
  outColour = drawTexture;
}
