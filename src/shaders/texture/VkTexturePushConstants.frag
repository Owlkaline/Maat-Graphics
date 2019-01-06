#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 new_colour;
layout(location = 2) in vec2 textured_blackwhite;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex;

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
