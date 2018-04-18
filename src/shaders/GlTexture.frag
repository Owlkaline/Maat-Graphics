#version 330 core

in vec2 uvs;

out vec4 outColour;

uniform sampler2D tex;
uniform vec4 new_colour;
uniform float has_texture;

void main() {
  vec4 drawTexture = new_colour;
  if(has_texture == 1.0) {
    drawTexture = texture(tex, uvs);
    if (new_colour.w != -1.0) {
      drawTexture.w *= new_colour.w;
    }
  }
  
  outColour = drawTexture;
}
