#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_colour;
layout(location = 2) in vec4 v_base_colour_factor;
layout(location = 3) in vec4 v_alpha_cutoff;
layout(location = 4) in vec3 v_normal;
layout(location = 5) in vec3 v_world_pos;
layout(location = 6) in vec3 v_camera_pos;
layout(location = 7) in vec3 v_light_pos;
layout(location = 8) in vec4 v_light_col;
layout(location = 9) in vec3 v_to_light;
layout(location = 10) in vec3 v_scanline;
layout(location = 11) in vec4 v_use_textures;
layout(location = 12) in vec2 v_mr;

layout(location = 0) out vec4 outColour;

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

float D_GGX(float dotNH, float roughness) {
  float alpha = roughness * roughness;
  float alpha2 = alpha * alpha;
  float denom = dotNH * dotNH * (alpha2 - 1.0) + 1.0;
  return (alpha2)/(M_PI * denom*denom); 
}

float G_SchlicksmithGGX(float dotNL, float dotNV, float roughness) {
  float r = (roughness + 1.0);
  float k = (r*r) / 8.0;
  float GL = dotNL / (dotNL * (1.0 - k) + k);
  float GV = dotNV / (dotNV * (1.0 - k) + k);
  
  return GL * GV;
}

vec3 F_Schlick(float cosTheta, float metallic) {
  vec3 F0 = mix(vec3(0.04), vec3(0.2, 0.2, 0.2), metallic);
  vec3 F = F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0); 
  
  return F; 
}

vec3 BRDF(vec3 L, vec3 V, vec3 N, float metallic, float roughness, vec3 light_colour) {
  vec3 H = normalize(V+L);
  float dotNV = clamp(dot(N, V), 0.0, 1.0);
  float dotNL = clamp(dot(N, L), 0.0, 1.0);
  float dotLH = clamp(dot(L, H), 0.0, 1.0);
  float dotNH = clamp(dot(N, H), 0.0, 1.0);
  
  vec3 colour = vec3(0.0);
  
  float distance = length(v_light_pos-v_world_pos);//abs(length(v_to_light));
  
  float intensity = v_light_col.w;
  float attenuation = 1.0;
  attenuation *= 1.0 / max(distance * distance, 0.01*0.01);
  
  vec3 radiance = light_colour * intensity * attenuation;
  
  if (dotNL > 0.0) {
    float rr = max(0.05, roughness);
    
    float D = D_GGX(dotNH, roughness);
    
    float G = G_SchlicksmithGGX(dotNL, dotNV, roughness);
    
    vec3 F = F_Schlick(dotNV, metallic);
    
    vec3 spec = D *F * G / (4.0 * dotNL * dotNV);
    
    colour += spec * radiance * dotNL; // * light_colour;
  }
  
  return colour;
}

void main() {
  //vec3 light_pos = vec3(0.0, 0.0, 0.0);
  
  vec3 N = normalize(v_normal);
  vec3 V = normalize(v_camera_pos - v_world_pos);
  
  vec3 Lo = vec3(0.0);
  
  vec3 L = normalize(v_to_light);//light_pos - v_world_pos);
  
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
  
  Lo += BRDF(L, V, N, v_mr.x, v_mr.y, v_light_col.xyz);
  
  base_colour *= 0.02;
  base_colour += Lo;
  
  base_colour = pow(base_colour, vec3(0.4545));
  
  float halpha = hologram_alpha(v_scanline.x, v_scanline.y);
  vec4 use_scanline = when_gt(vec4(v_scanline.z), vec4(0.0));
  
  alpha = use_scanline.a      * halpha + 
          not(use_scanline).a * alpha;
  
  outColour = vec4(base_colour, alpha);
  /*
  vec3 base_colour = vec3(1.0);
  float alpha = v_colour.a;
  
  if (v_colour.a < -0.0) {
    base_colour *= texture(base_texture, uvs).rgb;
    alpha += 1.1;
    alpha *= texture(base_texture, uvs).a;
  }
  
  outColour = vec4(base_colour, alpha);*/
}
