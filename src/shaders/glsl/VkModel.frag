#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_colour;
layout(location = 2) in vec4 v_base_colour_factor;
layout(location = 3) in vec4 v_alpha_cutoff;
layout(location = 4) in vec3 v_normal;
layout(location = 5) in vec3 v_world_pos;
layout(location = 6) in vec3 v_camera_pos;
layout(location = 10) in vec3 v_scanline;
layout(location = 11) in vec4 v_use_textures;
layout(location = 12) in vec2 v_mr;

layout(location = 0) out vec4 outColour;
layout(location = 1) out vec4 outAlbedo;
layout(location = 2) out vec4 outMro;
layout(location = 3) out vec4 outEmissive;
layout(location = 4) out vec4 outNormal;
layout(location = 5) out vec4 outPosition;

layout(set = 0, binding = 1) uniform sampler2D base_texture;

const float M_PI = 3.141592653589793;

vec4 when_gt(vec4 x, vec4 y) {
  return max(sign(x - y), 0.0);
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
  
  float halpha = hologram_alpha(v_scanline.x, v_scanline.y);
  vec4 use_scanline = when_gt(vec4(v_scanline.z), vec4(0.0));
  
  alpha = use_scanline.a      * halpha + 
          not(use_scanline).a * alpha;
  
  outAlbedo = vec4(base_colour, alpha);
  outMro = vec4(v_mr.x, v_mr.y, 0.0, 1.0);
  outEmissive = vec4(0.0, 0.0, 0.0, 0.0);
  outNormal = vec4(normalize(v_normal), 1.0);
  outPosition = vec4(v_world_pos, 1.0);
}
