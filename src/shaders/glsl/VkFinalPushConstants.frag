#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D texture_image;
layout(set = 1, binding = 0) uniform sampler2D model_image;

void main() {
  vec4 tex = texture(texture_image, uvs);
  vec4 model = texture(model_image, uvs);
  
  vec4 final_colour = tex;
  
  if (tex.w != 1.0) {
    if (tex.w == 0.0) {
      if (model.w == 0.0) {
        discard;
      }
      
      final_colour = model;
    }
  
    if (model.w != 0.0) {
      final_colour = vec4(mix(tex.rgb, model.rgb, 1.0-tex.a), 1.0);
    }
  }
  

  
  outColour = final_colour;
}
