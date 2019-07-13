#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec3 v_camera_pos;
layout(location = 2) in vec3 v_light_positions[3];
layout(location = 5) in vec3 v_light_colours[3];
layout(location = 8) in float v_light_intensity[3];
layout(location = 11) in vec3 v_sun_direction;
layout(location = 12) in vec4 v_sun_colour;

layout(location = 0) out vec4 outColour;

layout (input_attachment_index = 1, binding = 1) uniform subpassInput colour_texture;
layout (input_attachment_index = 2, binding = 2) uniform subpassInput mro_texture;
layout (input_attachment_index = 3, binding = 3) uniform subpassInput occlusion_texture;
layout (input_attachment_index = 4, binding = 4) uniform subpassInput normal_texture;
layout (input_attachment_index = 5, binding = 5) uniform subpassInput position_texture;

const float M_PI = 3.141592653589793;

float cot(float value) {
  return 1.0 / tan(value);
}

float to_radians(float degree) {
  return degree * (M_PI/180.0);
}

float D_GGX(float dotNH, float roughness) {
  float alpha = roughness * roughness;
  float alpha2 = alpha * alpha;
  float denom = dotNH * dotNH * (alpha2 - 1.0) + 1.0;
  //float denom = (dotNH * (alpha2) - dotNH) * dotNH + 1.0;
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

vec3 BRDF(vec3 L, vec3 V, vec3 N, float metallic, float roughness, vec3 light_position, vec3 light_colour, float intensity, vec3 world_pos) {
  vec3 H = normalize(V+L);
  float dotNV = clamp(dot(N, V), 0.0, 1.0);
  float dotNL = clamp(dot(N, L), 0.0, 1.0);
  float dotLH = clamp(dot(L, H), 0.0, 1.0);
  float dotNH = clamp(dot(N, H), 0.0, 1.0);
  
  vec3 colour = vec3(0.0);
  
  float distance = length(light_position-world_pos);
  
  float attenuation = 1.0;
  attenuation *= 1.0 / max(distance * distance, 0.01*0.01);
  
  vec3 radiance = light_colour * intensity * attenuation;
  
  if (dotNL > 0.0 && dotNV > 0.0) {
    float rr = max(0.05, roughness);
    
    float D = D_GGX(dotNH, roughness);
    
    float G = G_SchlicksmithGGX(dotNL, dotNV, roughness);
    
    vec3 F = F_Schlick(dotNV, metallic);
    
    vec3 spec = D *F * G / (4.0 * dotNL * dotNV);
    
    colour += spec * radiance * dotNL;
  }
  
  return colour;
}

/*
float getLinearDepth(vec2 coord) {
    float depth = texture2D(gBufferTexture2, coord).r * 2.0 - 1.0;
    return projection[3][2] / (depth * projection[2][3] - projection[2][2]);
}
*/

void main() {
  vec4 base_colour = subpassLoad(colour_texture);
  
  if (base_colour.a == 0.0) {
    discard;
  }
  
  vec3 world_pos = subpassLoad(position_texture).rgb;
  vec3 N = vec3(subpassLoad(normal_texture).rgb);
  vec3 V = normalize(v_camera_pos.xyz - world_pos);
  
  vec3 Lo = vec3(0.0);
  
  vec3 L[3];
  L[0] = normalize(v_light_positions[0] - world_pos);
  L[1] = normalize(v_light_positions[1] - world_pos);
  L[2] = normalize(v_light_positions[2] - world_pos);
  
  vec4 mro_colour = subpassLoad(mro_texture);
  
  for(int i = 0; i < 3; ++i) {
    Lo += BRDF(L[i], V, N, mro_colour.b, mro_colour.g, v_light_positions[i], v_light_colours[i], v_light_intensity[i], world_pos);
  }
  
  base_colour.rgb += Lo;
  
  base_colour.rgb = pow(base_colour.rgb, vec3(0.4545));
  
  outColour = base_colour;
}


