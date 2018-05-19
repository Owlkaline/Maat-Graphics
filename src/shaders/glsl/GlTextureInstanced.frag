#version 330 core

in vec2 uvs;
in vec4 colour;
in float textured;
in float blackwhite;

out vec4 outColour;

uniform sampler2D tex;

void main() {
  vec4 drawTexture = colour;
  if(textured >= 1.0) {
    drawTexture = texture(tex, uvs);
    if (colour.w != -1.0) {
      drawTexture.w *= colour.w;
    }
  }
  
  if(blackwhite >= 1.0) {
    float brightness = dot(drawTexture.rgb*vec3(2.0), vec3(0.2126, 0.7152, 0.0722));
    drawTexture = vec4(vec3(brightness), drawTexture.a);
  }
  
  outColour = drawTexture;
}
