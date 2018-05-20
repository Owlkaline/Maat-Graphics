#version 330 core

in vec2 uvs;

out vec4 outColour;

uniform sampler2D tex;

void main() {
  vec4 bloom = texture(tex, uvs);
  
  // Convert to grayscale and compute brightness
  float brightness = dot(bloom.rgb, vec3(0.2126, 0.7152, 0.0722));
  
  if(brightness < 0.5) {
    bloom = vec4(0.0, 0.0, 0.0, 1.0);
  }
  
  outColour = bloom;
}
