#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec3 toCameraVector;
layout(location = 3) in vec3 toLightVector[4];
layout(location = 7) in vec3 lightColour[4];
layout(location = 11) in vec3 attenuation[4];
layout(location = 15) in float lightType[4];

layout(location = 0) out vec4 f_colour;

layout(set = 0, binding = 1) uniform sampler2D tex;

// https://freepbr.com/
// Start Cell shading
// float levels = 4.0;
// float level = floor(brightness*levels);
// brightness = level/levels;
// Do same for damped factor
// End Cell Shading

const float PI = 3.14159265359;

const float metallic = 1.0;
const float roughness = 0.0;
const float ao = 0.2;

float DistributionGGX(vec3 N, vec3 H, float roughness) {
  float a = roughness*roughness;
  float a2 = a*a;
  float NdotH = max(dot(N, H), 0.0);
  float NdotH2 = NdotH*NdotH;

  float nom   = a2;
  float denom = (NdotH2 * (a2 - 1.0) + 1.0);
  denom = PI * denom * denom;

  return nom / max(denom, 0.001); // prevent divide by zero for roughness=0.0 and NdotH=1.0
}

float GeometrySchlickGGX(float NdotV, float roughness) {
  float r = (roughness + 1.0);
  float k = (r*r) / 8.0;

  float nom   = NdotV;
  float denom = NdotV * (1.0 - k) + k;

  return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
  float NdotV = max(dot(N, V), 0.0);
  float NdotL = max(dot(N, L), 0.0);
  float ggx2 = GeometrySchlickGGX(NdotV, roughness);
  float ggx1 = GeometrySchlickGGX(NdotL, roughness);

  return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
  return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

void main() {
  vec3 N = normalize(v_normal);
  vec3 V = normalize(toCameraVector);
  
  vec3 base_texture = pow(texture(tex, v_uv).rgb, vec3(2.2));
  
  vec3 F0 = vec3(0.04);
  F0 = mix(F0, base_texture, metallic);
  
  vec3 Lo = vec3(0.0);
  for(int i = 0; i < 4; ++i) {
    vec3 L = normalize(toLightVector[i]);
    vec3 H = normalize(V + L);
    float distance = length(toLightVector[i]);
    
    float lightRadius = 10.0;
    
    float dist_radius = (distance/lightRadius);
    float dist_radius2 = dist_radius*dist_radius*dist_radius*dist_radius;
    float clamp_dist = clamp(1.0 - dist_radius2, 0.0, 1.0);
    
    float falloff = (clamp_dist*clamp_dist) / (distance*distance+1);

//    float falloff = (clamp(1-(distance/lightRadius)^4)^2)/distance*distance+1;
    
    float attenuation = 1.0 / pow(distance, lightType[i]);
    vec3 radiance = lightColour[i] * attenuation; //*falloff; 
    
    float NDF = DistributionGGX(N, H, clamp(roughness, 0.3, 1.0));
    float G   = GeometrySmith(N, V, L, roughness);
    vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);
    
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;
    
    vec3 nominator = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
    vec3 specular = nominator / max(denominator, 0.001);
    
    float NdotL = max(dot(N, L), 0.0);
    Lo += (kD * base_texture / PI + specular) * radiance * NdotL;
  }
  
  vec3 ambient = vec3(0.03) * base_texture * ao;
  vec3 colour = ambient + Lo;
  
  colour = colour / (colour + vec3(1.0));
  colour = pow(colour, vec3(1.0/2.2));
  
  f_colour = vec4(colour, 1.0);
}
