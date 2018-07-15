#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
  vec4 bloom = texture(tex, uvs);
  
  // Convert to grayscale and compute brightness
  float brightness = dot(bloom.rgb, vec3(0.2126, 0.7152, 0.0722));
  
  if(brightness < 0.95) {
    bloom = vec4(0.0, 0.0, 0.0, 1.0);
  }
  
  outColour = bloom;
}
