#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_colour;
layout(location = 2) in vec4 v_base_colour_factor;
layout(location = 3) in vec4 v_alpha_cutoff;
layout(location = 4) in vec3 v_normal;
layout(location = 5) in vec3 v_to_light;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D base_texture;

const float M_PI = 3.141592653589793;
const vec3 light_colour = vec3(0.5, 0.5, 0.5);

void main() {
  vec3 N = normalize(v_normal);
  vec3 to_light = normalize(v_to_light);
  
  float ndot1 = dot(N, to_light);
  float brightness = max(ndot1, 0.2);
  
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
  
  base_colour.xyz *= brightness;
  
  outColour = vec4(base_colour, alpha);
}
