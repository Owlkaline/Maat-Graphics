#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_colour;
layout(location = 2) in vec4 v_base_colour_factor;
layout(location = 3) in vec4 v_alpha_cutoff;
layout(location = 4) in vec3 v_normal;
layout(location = 5) in vec3 v_world_pos;
layout(location = 6) in vec3 v_camera_pos;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D base_texture;

const float M_PI = 3.141592653589793;

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
  
//  vec3 light_colour = colour;//vec3(1.0);
  
  vec3 colour = vec3(0.0);
  
  if (dotNL > 0.0) {
    float rr = max(0.05, roughness);
    
    float D = D_GGX(dotNH, roughness);
    
    float G = G_SchlicksmithGGX(dotNL, dotNV, roughness);
    
    vec3 F = F_Schlick(dotNV, metallic);
    
    vec3 spec = D *F * G / (4.0 * dotNL * dotNV);
    
    colour += spec * dotNL * light_colour;
  }
  
  return colour;
}

void main() {
 /* vec3 light_pos = vec3(0.0, 0.0, 0.0);
  
  vec3 N = normalize(v_normal);
  vec3 V = normalize(v_camera_pos - v_world_pos);
  
  vec3 Lo = vec3(0.0);
  
  vec3 L = normalize(light_pos - v_world_pos);
  
  
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
  
  Lo += BRDF(L, V, N, 0.0, 1.0, base_colour);
  base_colour *= 0.02;
  base_colour += Lo;
  
  base_colour = pow(base_colour, vec3(0.4545));
  
  outColour = vec4(base_colour, alpha);*/
  
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
