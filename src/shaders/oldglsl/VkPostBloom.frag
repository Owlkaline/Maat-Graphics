#version 450

layout(location = 0) out vec4 f_colour;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_colour;

void main() {
  vec4 bloom = subpassLoad(u_colour);
  
  // Convert to grayscale and compute brightness
  float brightness = dot(bloom.rgb, vec3(0.2126, 0.7152, 0.0722));
  
  if(brightness < 0.95) {
    bloom = vec4(0.0, 0.0, 0.0, 1.0);
  }
  
  f_colour = bloom;
}
