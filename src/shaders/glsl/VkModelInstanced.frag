#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_colour;
layout(location = 2) in vec4 v_base_colour_factor;
layout(location = 3) in vec4 v_alpha_cutoff;
layout(location = 4) in vec3 v_normal;
layout(location = 5) in vec3 v_to_light[2];
layout(location = 7) in vec3 v_scanline;
layout(location = 8) in vec4 v_use_textures;
layout(location = 9) in vec4 v_overwrite_colour;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D base_texture;

const float M_PI = 3.141592653589793;
const vec3 light_colour = vec3(0.5, 0.5, 0.5);

vec4 when_gt(vec4 x, vec4 y) {
  return max(sign(x - y), 0.0);
}

vec4 when_lt(vec4 x, vec4 y) {
  return max(sign(y - x), 0.0);
}

vec4 not(vec4 a) {
  return 1.0 - a;
}

float hologram_alpha(float scanline, float y_offset) {
  
  float blah = 0.25f * sin(10.0*y_offset + scanline*-15.0);
  blah += 0.4f;
  
  float n_offset = (y_offset+1.0f)/2;
  
  float alpha = cos(M_PI*n_offset - (scanline*M_PI)) + 1;
  alpha = 0.017-alpha;
  alpha = alpha *100;
  
  alpha = max(blah, alpha);
  alpha = alpha * 0.8;
  
  return alpha;
}

void main() {
  vec3 N = normalize(v_normal);
  vec3 to_light = normalize(v_to_light[0]);
  vec3 to_light2 = normalize(v_to_light[1]);
  
  float ndot1 = dot(N, to_light);
  float ndot2 = dot(N, to_light2);
  float brightness2 = max(ndot2, 0.0);
  float brightness1 = max(ndot1, 0.0);
  
  float brightness = max(brightness1+brightness2, 0.1);
  
  vec3 base_colour = vec3(1.0);
  float alpha = v_colour.a;
  
  vec4 use_base_texture = when_gt(vec4(v_use_textures.x), vec4(0.0));
  base_colour = use_base_texture.rgb      * texture(base_texture, uvs).rgb + 
                not(use_base_texture).rgb * base_colour;
  
  alpha = use_base_texture.a    * texture(base_texture, uvs).a + 
          not(use_base_texture).a * alpha;
  
  
  base_colour *= v_base_colour_factor.rgb;
  base_colour *= v_colour.rgb;
  alpha *= v_base_colour_factor.a;
  
  base_colour.xyz *= brightness;
  
  float halpha = hologram_alpha(v_scanline.x, v_scanline.y);
  vec4 use_scanline = when_gt(vec4(v_scanline.z), vec4(0.0));
  
  alpha = use_scanline.a      * halpha + 
          not(use_scanline).a * alpha;
  
  vec4 overwrite_colour = when_lt(vec4(v_overwrite_colour.a), vec4(0.0));
  base_colour = overwrite_colour.xyz      * v_overwrite_colour.xyz +
                not(overwrite_colour).xyz * base_colour;
  
  outColour = vec4(base_colour, alpha);
}
