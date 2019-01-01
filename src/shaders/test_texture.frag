#version 450

layout (location = 0) in vec3 colours;
layout (location = 1) in vec2 uvs;

layout (location = 0) out vec4 outFragColor;

layout (binding = 1) uniform sampler2D tex;

void main() {
  outFragColor = vec4(colours * texture(tex, uvs).rgb, 1.0);
}
