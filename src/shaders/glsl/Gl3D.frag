#version 330 core

in vec3 v_position;
in vec3 v_normal;
in vec4 v_tangent;
in vec2 v_uv;
in vec4 v_colours;
in mat3 v_tbn;
in vec3 toCameraVector;

out vec4 f_colour;

uniform vec4  u_base_colour_factor;
uniform vec2  u_metallic_roughness_factor;

uniform float u_alpha_cutoff;
uniform float u_normal_scale;
uniform float u_occlusion_strength;
uniform vec3 u_emissive_factor;

uniform int u_forced_alpha;
uniform int u_has_normals;
uniform int u_has_tangents;

uniform int u_has_colour_texture;
uniform int u_has_metallic_roughness_texture;
uniform int u_has_normal_texture;
uniform int u_has_occlusion_texture;
uniform int u_has_emissive_texture;

uniform sampler2D u_base_colour;
uniform sampler2D u_metallic_roughness;
uniform sampler2D u_normal_texture;
uniform sampler2D u_occlusion_texture;
uniform sampler2D u_emissive_texture;

const float M_PI = 3.141592653589793;
const float c_MinRoughness = 0.04;
const vec3 c_LightColor = vec3(0.4,0.4,0.4);
//const vec3 c_LightDirection = vec3(0.0, 1.0, 0.0);
const vec3 c_LightDirection = vec3(-0.4, 0.35, 0.2);

vec3 get_normal() {
  mat3 tbn;
  if (u_has_tangents == 1) { // has_tangents
    tbn = v_tbn;
  } else {
    // doesnt have tangents
    vec3 pos_dx = dFdx(vec3(v_position.x, v_position.y, v_position.z));
    vec3 pos_dy = dFdy(vec3(v_position.x, v_position.y, v_position.z));
    vec3 tex_dx = dFdx(vec3(v_uv, 0.0));
    vec3 tex_dy = dFdy(vec3(v_uv, 0.0));
    vec3 t = (tex_dy.t * pos_dx - tex_dx.t * pos_dy) / (tex_dx.s * tex_dy.t - tex_dy.s * tex_dx.t);
    
    vec3 N;
    if (u_has_normals == 1) {
      N = normalize(v_normal);
    } else {
      N = cross(pos_dx, pos_dy);
    }
    
    t = normalize(t - N * dot(N, t));
    vec3 b = normalize(cross(N, t));
    tbn = mat3(t, b, N);
  }
  
  vec3 n = tbn[2].xyz;
  if (u_has_normal_texture == 0) {
    n = texture(u_normal_texture, v_uv).rgb;
    n = normalize(tbn * ((2.0 * n - 1.0) * vec3(u_normal_scale, u_normal_scale, 1.0)));
  }
  
  // gl front facing?
  // n *= (2.0 * float(gl_FrontFacing) - 1.0);
  
  return n;
}

// Lambertian diffuse, Photometria
vec3 diffuse(vec3 diffuse_colour) {
  return diffuse_colour / M_PI;
}

vec3 specularReflection(vec3 reflectance0, vec3 reflectance90, float VdotH) {
  return reflectance0 + (reflectance90 - reflectance0) * pow(clamp(1.0 - VdotH, 0.0, 1.0), 5.0);
}

// Attenuation
float geometricOcclusion(float NdotL, float NdotV, float r) {
  float rr = r * r;
  
  float attenuationL = 2.0 * NdotL / (NdotL + sqrt(rr + (1.0 - rr) * (NdotL * NdotL)));
  float attenuationV = 2.0 * NdotV / (NdotV + sqrt(rr + (1.0 - rr) * (NdotV * NdotV)));
  return attenuationL * attenuationV;
}

float microfacetDistribution(float r, float NdotH) {
  float rr = r * r;
  float f = (NdotH * rr - NdotH) * NdotH + 1.0;
  return rr / (M_PI * f * f);
}

void main() {
  float metallic = u_metallic_roughness_factor.x;
  float roughness = u_metallic_roughness_factor.y;
  
  if (u_has_metallic_roughness_texture == 0) {
    vec4 Sample = texture(u_metallic_roughness, v_uv);
    roughness = Sample.g * roughness;
    metallic = Sample.b * metallic;
  }
  
  roughness = clamp(roughness, c_MinRoughness, 1.0);
  metallic = clamp(metallic, 0.0, 1.0);
  
  float alpha_roughness = roughness * roughness;
  
  vec4 base_colour = vec4(1.0, 1.0, 1.0, 1.0);
  if (u_has_colour_texture == 0) {
    base_colour = texture(u_base_colour, v_uv);
  }
  base_colour *= u_base_colour_factor;
  base_colour *= v_colours;
  
  float alpha_cutoff = u_alpha_cutoff;
  float alpha = base_colour.a;
  if (u_forced_alpha == 1) { // Opaque
    alpha = 1.0;
  }
  
  if (u_forced_alpha == 2) { // Mask
    if(alpha < alpha_cutoff) {
      discard;
    } else {
      alpha = 1.0;
    }
  }
  
  vec3 f0 = vec3(0.04);
  vec3 diffuse_colour = base_colour.rgb * (vec3(1.0) - f0);
  diffuse_colour *= 1.0 - metallic;
  vec3 specular_colour = mix(f0, base_colour.rgb, metallic);
  
  float reflectance = max(max(specular_colour.r, specular_colour.g), specular_colour.b);
  
  float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
  vec3 specular_eviroment_r0 = specular_colour.rgb;
  vec3 specular_eviroment_r90 = vec3(1.0, 1.0, 1.0) * reflectance90;
  
  // Lights and stuff start here
  vec3 N = get_normal();
  vec3 V = normalize(toCameraVector);
  vec3 L = normalize(c_LightDirection);
  vec3 H = normalize(L+V);
  vec3 reflection = -normalize(reflect(V, N));
  
  float NdotL = clamp(dot(N, L), 0.001, 1.0);
  float NdotV = abs(dot(N, V)) + 0.001;
  float NdotH = clamp(dot(N, H), 0.0, 1.0);
  float LdotH = clamp(dot(L, H), 0.0, 1.0);
  float VdotH = clamp(dot(V, H), 0.0, 1.0);
  
  vec3 F = specularReflection(specular_eviroment_r0, specular_eviroment_r90, LdotH);
  float G = geometricOcclusion(NdotL, NdotV, alpha_roughness);
  float D = microfacetDistribution(alpha_roughness, NdotH);
  
  vec3 diffuse_contrib = (1.0 - F) * diffuse(diffuse_colour);
  vec3 specular_contrib = F * G * D / (4.0 * NdotL * NdotV);
  vec3 colour = NdotL * c_LightColor * (diffuse_contrib + specular_contrib);
  
  // colour += Ambient light colour + intensity + base colour
  colour += vec3(1.0, 1.0, 1.0) * 0.2 * base_colour.xyz;
  
  if (u_has_occlusion_texture != -1) {
    float ao = texture(u_occlusion_texture, v_uv).r;
    colour = mix(colour, colour * ao, u_occlusion_strength);
  }
  
  vec3 emissive = vec3(0.0);
  if (u_has_emissive_texture != -1) {
    emissive = texture(u_emissive_texture, v_uv).rgb * u_emissive_factor;
  }
  
  colour.rgb += emissive;
  
  f_colour.rgb = pow(colour.rgb, vec3(1.0/2.2));
  f_colour.a = alpha;
}
