#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
  outColour = texture(tex, uvs);
  
  // Convert to grayscale and compute brightness
  float brightness = dot(outColour.rgb, vec3(0.2126, 0.7152, 0.0722));
  
  outColour = brightness > 0.7 ? outColour : vec4(0.0, 0.0, 0.0, 1.0);
}
