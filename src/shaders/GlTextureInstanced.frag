#version 330 core

in vec2 uvs;
in vec4 colour;
in float new_texture;

out vec4 outColour;

uniform sampler2D tex;

void main() {
  vec4 drawTexture = colour;
  if(new_texture == 1.0)
   drawTexture = texture(tex, uvs);
  
  outColour = drawTexture;
}
