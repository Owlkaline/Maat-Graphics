#version 330 core

in vec2 TexCoords;

out vec4 outColour;

uniform sampler2D text;
uniform vec4 fragColour;

void main() {
  vec4 sampled = vec4(1.0, 1.0, 1.0, texture(text, TexCoords).r);
  outColour = vec4(fragColour) * sampled;
}
