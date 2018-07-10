#version 450

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;
layout(location = 3) in vec3 toCameraVector;
layout(location = 4) in vec3 toLightVector[4];
layout(location = 8) in vec3 lightColour[4];
layout(location = 12) in vec3 attenuation[4];
layout(location = 16) in float lightType[4];

layout(location = 0) out vec4 f_colour;

// https://freepbr.com/
// Start Cell shading
// float levels = 4.0;
// float level = floor(brightness*levels);
// brightness = level/levels;
// Do same for damped factor
// End Cell Shading

layout(set = 1, binding = 0) uniform MaterialParams {
    vec4 base_colour_factor;
    int base_colour_texture_tex_coord;
    float metallic_factor;
    float roughness_factor;
    int metallic_roughness_texture_tex_coord;
    float normal_texture_scale;
    int normal_texture_tex_coord;
    int occlusion_texture_tex_coord;
    float occlusion_texture_strength;
    int emissive_texture_tex_coord;
    vec3 emissive_factor;
} u_material_params;

layout(set = 1, binding = 1) uniform sampler2D u_base_colour;
layout(set = 1, binding = 2) uniform sampler2D u_metallic_roughness;
layout(set = 1, binding = 3) uniform sampler2D u_normal_texture;
layout(set = 1, binding = 4) uniform sampler2D u_occlusion_texture;
layout(set = 1, binding = 5) uniform sampler2D u_emissive_texture;

const float M_PI = 3.141592653589793;
const float c_MinRoughness = 0.04;
const vec3 c_LightColor = vec3(1.0,1.0,1.0);

// flIP y
//const vec3 c_LightDirection = vec3(-0.4, -0.7, 0.2);
const vec3 c_LightDirection = vec3(-1.0, 0.0, 0.0);

vec3 get_normal() {
  // Get tangents
  vec3 pos_dx = dFdx(v_position);
  vec3 pos_dy = dFdy(v_position);
  vec3 tex_dx = dFdx(vec3(v_uv, 0.0));
  vec3 tex_dy = dFdy(vec3(v_uv, 0.0));
  vec3 t = (tex_dy.t * pos_dx - tex_dx.t * pos_dy) / (tex_dx.s * tex_dy.t - tex_dy.s * tex_dx.t);
  
  vec3 N;
  if (u_material_params.normal_texture_tex_coord != -1) {
    N = normalize(v_normal);
  } else {
    N = cross(pos_dx, pos_dy);
  }
  
  t = normalize(t - N * dot(N, t));
  vec3 b = normalize(cross(N, t));
  mat3 tbn = mat3(t, b, N);
  
  vec3 n = tbn[2].xyz;
  if (u_material_params.normal_texture_tex_coord != -1) {
    n = texture(u_normal_texture, v_uv).rgb;
    n = normalize(tbn * ((2.0 * n - 1.0) * vec3(u_material_params.normal_texture_scale, u_material_params.normal_texture_scale, 1.0)));
  }
  
  // gl front facing?
  
  return n;
}

// Lambertian diffuse, Photometria
vec3 diffuse(vec3 diffuse_colour) {
  return diffuse_colour / M_PI;
}

vec3 specularReflection(vec3 reflectance0, vec3 reflectance90, float VdotH) {
  return reflectance0 + (reflectance90 - reflectance0) * pow(clamp(1.0 - VdotH, 0.0, 1.0), 5);
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

  
  vec3 N = normalize(v_normal);
  
  float metallic = u_material_params.metallic_factor;
  float roughness = u_material_params.roughness_factor;
  
  if (u_material_params.metallic_roughness_texture_tex_coord != -1) {
    vec4 Sample = texture(u_metallic_roughness, v_uv);
    roughness = Sample.g * roughness;
    metallic = Sample.b * metallic;
  }
  
  roughness = clamp(roughness, c_MinRoughness, 1.0);
  metallic = clamp(metallic, 0.0, 1.0);
  
  float alpha_roughness = roughness * roughness;
  
  vec4 base_colour;
  if (u_material_params.base_colour_texture_tex_coord != -1) {
    base_colour = texture(u_base_colour, v_uv) * u_material_params.base_colour_factor;
  } else {
    base_colour = u_material_params.base_colour_factor;
  }
  
  base_colour.rgb = pow(base_colour.rgb, vec3(2.2));
  if(base_colour.a < 0.1) {
    discard;
  }
  
  vec3 f0 = vec3(0.04);
  vec3 diffuse_colour = base_colour.rbg * (vec3(1.0) - f0);
  diffuse_colour *= 1.0 - metallic;
  vec3 specular_colour = mix(f0, base_colour.rgb, metallic);
  
  float reflectance = max(max(specular_colour.r, specular_colour.g), specular_colour.b);
  
  float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
  vec3 specular_eviroment_r0 = specular_colour.rgb;
  vec3 specular_eviroment_r90 = vec3(1.0, 1.0, 1.0) * reflectance90;
  
  // Lights and stuff start here
  N = get_normal();
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
  
  
  if (u_material_params.occlusion_texture_tex_coord != -1) {
    float ao = texture(u_occlusion_texture, v_uv).r;
    colour = mix(colour, colour * ao, u_material_params.occlusion_texture_strength);
  }
  
  vec3 emissive = vec3(0.0);
  if (u_material_params.emissive_texture_tex_coord != -1) {
    emissive = texture(u_emissive_texture, v_uv).rgb * vec3(0.0, 1.0, 0.0);
    //emissive.a = 1.0;
  }
  
  colour.rgb += emissive;
  
  f_colour.rgb = colour.rgb / (colour.rgb + vec3(1.0));
  f_colour.rgb = pow(colour.rgb, vec3(1.0/2.2)); 
  f_colour.a = base_colour.a;
  //f_colour = vec4(colour, base_colour.a);
}
