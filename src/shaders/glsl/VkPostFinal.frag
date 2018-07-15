#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in float bloom_enabled;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 2) uniform sampler2D bloom;

void main() {
  vec3 colour = texture(tex, uvs).rgb;
  vec3 bloom = texture(bloom, uvs).rgb;
  
 
  if (bloom_enabled > 1.0) {
    const float gamma = 2.2;
    const float exposure = 0.75;
    bloom = vec3(1.0) - exp(-bloom * exposure);
    bloom = pow(bloom, vec3(1.0 / gamma));
    colour += bloom;
    
   // vec3 mapped = colour / (colour + vec3(1.0));
    
    //colour = pow(mapped, vec3(1.0 / gamma));
  }
  
  outColour = vec4(colour, 1.0);
}
