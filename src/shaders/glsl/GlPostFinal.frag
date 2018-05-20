#version 330 core

in vec2 uvs;

out vec4 outColour;

uniform sampler2D tex;
uniform sampler2D bloom;

uniform float bloom_enabled;

void main() {
  vec3 colour = texture(tex, uvs).rgb;
  vec3 bloom = texture(bloom, uvs).rgb;
  
  if (bloom_enabled > 1.0) {
    const float gamma = 2.2;
    
    bloom = vec3(1.0) - exp(-bloom * 1.0);
    bloom = pow(bloom, vec3(1.0 / gamma));
    colour += bloom;
  }
  
  outColour = vec4(colour, 1.0);
}
