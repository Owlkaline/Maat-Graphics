#version 450

layout(location = 0) in vec3 uvs_alpha;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
  vec4 image = texture(tex, uvs_alpha.xy);
  image.a*=uvs_alpha.z;
  outColour = image;
}
