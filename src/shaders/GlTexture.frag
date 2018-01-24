#version 330 core

in vec2 uvs;

out vec4 outColour;

uniform sampler2D tex;
uniform vec4 new_colour;
uniform vec4 has_texture;

void main() {
  vec4 drawTexture = new_colour;
  if(has_texture.x == 1.0)
   drawTexture = texture(tex, uvs);
  
  outColour = drawTexture;
}
