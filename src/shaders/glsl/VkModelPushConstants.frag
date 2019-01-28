#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_colour;
layout(location = 2) in vec4 v_base_colour_factor;
layout(location = 3) in vec4 v_alpha_cutoff;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D base_texture;

void main() {
  vec3 base_colour = vec3(1.0);
  float alpha = v_colour.a;
  
  if (v_colour.a < -0.0) {
    base_colour *= texture(base_texture, uvs).rgb;
    alpha += 1.1;
    alpha *= texture(base_texture, uvs).a;
  }
  
  base_colour *= v_base_colour_factor.rgb;
  base_colour *= v_colour.rgb;
  alpha *= v_base_colour_factor.a;
  
  float alpha_cutoff = v_alpha_cutoff.x;
  float alpha_mask = v_alpha_cutoff.y;
  
  if (alpha_mask == 1.0) { //opaque
    alpha = 1.0;
  } else if (alpha_mask == 2.0) { // mask
    if (alpha < alpha_cutoff) { // draw nothing
      discard;
    } else {
      alpha = alpha_cutoff;
    }
  }
  
  outColour = vec4(base_colour, alpha);
}
