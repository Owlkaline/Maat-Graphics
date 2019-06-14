#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec3 v_world_pos;
layout(location = 3) in vec3 v_camera_pos;
layout(location = 3) in vec3 v_light_positions[3];
layout(location = 6) in vec3 v_light_colours[3];
layout(location = 9) in float v_light_intensity[3];
layout(location = 12) in vec3 v_sun_direction;
layout(location = 13) in vec4 v_sun_colour;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 2) uniform sampler2D colour_texture;
layout(set = 0, binding = 3) uniform sampler2D mro_texture;
layout(set = 0, binding = 4) uniform sampler2D emissive_texture;
layout(set = 0, binding = 5) uniform sampler2D normal_texture;

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

vec3 BRDF(vec3 L, vec3 V, vec3 N, float metallic, float roughness, vec3 light_position, vec3 light_colour, float intensity) {
  vec3 H = normalize(V+L);
  float dotNV = clamp(dot(N, V), 0.0, 1.0);
  float dotNL = clamp(dot(N, L), 0.0, 1.0);
  float dotLH = clamp(dot(L, H), 0.0, 1.0);
  float dotNH = clamp(dot(N, H), 0.0, 1.0);
  
  vec3 colour = vec3(0.0);
  
  float distance = length(light_position-v_world_pos);
  
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
  vec3 N = texture(normal_texture,uvs).rgb;
  vec3 V = texture(camera_to_world_texture, uvs).rgb;
  
  vec3 Lo = vec3(0.0);
  
  vec3 L = normalize(v_light_positions[0] - v_world_pos);
  
  vec4 base_colour = texture(colour_texture,uvs);
  vec4 mro_colour = texture(mro_texture, uvs);
  
  Lo += BRDF(L, V, N, mro_colour.x, mro_colour.y, v_light_positions[0], v_light_colours[0].xyz, v_light_intensity[0]);
  
  base_colour *= 0.02;
  base_colour.rgb += Lo;
  
  base_colour.rgb = pow(base_colour.rgb, vec3(0.4545));
  
  outColour = base_colour;
}
