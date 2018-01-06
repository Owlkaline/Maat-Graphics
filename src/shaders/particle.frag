#version 330 core

out vec4 outColour;

in vec2 textureCoords1;
in vec2 textureCoords2;
in float blend;
in float fade;

uniform sampler2D particleTexture;

void main() {
  vec4 colour1 = texture(particleTexture, textureCoords1);
  vec4 colour2 = texture(particleTexture, textureCoords2);
  
  outColour = mix(colour1, colour2, blend) * vec4(vec3(1), 1-fade);
}
