#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_colour;
layout(location = 2) in vec4 v_base_colour_factor;
layout(location = 3) in vec4 v_alpha_cutoff;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D base_texture;

void main() {
  vec4 base_colour = vec4(1.0);
  base_colour = texture(base_texture, uvs);
  base_colour.rgb *= v_base_colour_factor.rgb;
  base_colour *= v_colour;
  
  float alpha_cutoff = v_alpha_cutoff.x;
  float alpha_mask = v_alpha_cutoff.y;
  
  float alpha = base_colour.a;
  /*
  if (alpha_mask == 1) {
    alpha = 1.0;
  }*/
  
  if (alpha_mask == 2) {
    if (alpha < alpha_cutoff) {
      discard;
    } else {
      alpha = 1.0;
    }
  }
  
  outColour = vec4(base_colour.rgb, alpha);
}
