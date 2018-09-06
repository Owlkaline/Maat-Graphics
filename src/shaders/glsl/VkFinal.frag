#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex2d;
//layout(set = 0, binding = 2) uniform sampler2D tex3d;

void main() {
 // vec4 final_colour = texture(tex3d, uvs);
  vec4 colour2d = texture(tex2d, uvs);
  
  //final_colour.rgb += colour2d.rgb * colour2d.a;
  
  outColour = colour2d;//vec4(final_colour.rgb, 1.0);
}
