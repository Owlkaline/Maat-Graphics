#version 330 core

in vec2 uvs;

out vec4 outColour;

uniform sampler2D image;
uniform vec4 colour;

void main() {
  vec4 drawTexture = colour;
  if(colour.w == -1)
   drawTexture = texture(image, uvs);
  
  outColour = drawTexture;
}
