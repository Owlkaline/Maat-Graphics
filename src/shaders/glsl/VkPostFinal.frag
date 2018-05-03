#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 2) uniform sampler2D bloom;

void main() {
  outColour = vec4(texture(tex, uvs).rgb + texture(bloom, uvs).rbg, 1.0);
}
