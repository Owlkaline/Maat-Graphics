#version 330 core

in vec2 uvs;

out vec4 outColour;

uniform sampler2D tex;
uniform vec4 new_colour;
uniform vec2 textured_blackwhite;

void main() {
  vec4 drawTexture = new_colour;
  if(textured_blackwhite.x >= 1.0) {
    drawTexture = texture(tex, uvs);
    if (new_colour.w != -1.0) {
      drawTexture.w *= new_colour.w;
    }
  }
  
  if(textured_blackwhite.y >= 1.0) {
    float brightness = dot(drawTexture.rgb*vec3(1.0), vec3(0.2126, 0.7152, 0.0722));
    drawTexture = vec4(vec3(brightness), drawTexture.a);
  }
  
  outColour = drawTexture;
}
