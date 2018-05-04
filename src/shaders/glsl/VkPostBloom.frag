#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
  outColour = texture(tex, uvs);
  
  if (outColour.r < 0.4) {
    outColour.r = 0.0;
  }
  if (outColour.g < 0.4) {
    outColour.g = 0.0;
  }
  if (outColour.b < 0.4) {
    outColour.b = 0.0;
  }
  
  if (outColour.r > 0.6) {
    outColour.r = 0.0;
  }
  if (outColour.g > 0.6) {
    outColour.g = 0.0;
  }
  if (outColour.b > 0.6) {
    outColour.b = 0.0;
  }
  
  outColour = outColour;
  
  // Convert to grayscale and compute brightness
  float brightness = dot(outColour.rgb*vec3(10.6), vec3(0.2126, 0.7152, 0.0722));
 // float brightness = outColour.b; //- (outColour.r + outColour.g);
  
  //outColour = brightness > 1.0 ? outColour : vec4(0.0, 0.0, 0.0, 1.0);
}
