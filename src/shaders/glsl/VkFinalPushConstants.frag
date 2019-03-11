#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D texture_image;
layout(set = 1, binding = 0) uniform sampler2D model_image;

void main() {
  vec4 tex = texture(texture_image, uvs);
  vec4 model = texture(model_image, uvs);
  
  vec4 final_colour = vec4(tex.rgb, 1.0);
  if (tex == vec4(0.0)) {
    //drawmodel
    //final_colour = model;
  }
  
  outColour = tex+model;
}
