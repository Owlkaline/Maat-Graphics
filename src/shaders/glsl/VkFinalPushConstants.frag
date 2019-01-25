#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D tex2d;

void main() {
  outColour = vec4(0.0, 1.0, 0.0, 1.0);//texture(tex2d, uvs);
}
