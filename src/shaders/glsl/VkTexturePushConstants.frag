#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 new_colour;
layout(location = 2) in float use_texture;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  vec4 drawTexture = new_colour;
  if (use_texture < 0.0) {
    drawTexture = texture(tex, uvs);
    if (new_colour.w != -1.0) {
      drawTexture.w *= new_colour.w;
    }
  }
  
  outColour = drawTexture*vec4(new_colour.xyz, 1.0);
}
