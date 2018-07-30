#version 450

layout(location = 0) out vec4 f_colour;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInputMS u_colour;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInputMS u_normal;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInputMS u_position;
layout(input_attachment_index = 3, set = 0, binding = 3) uniform subpassInputMS u_uv;
layout(input_attachment_index = 4, set = 0, binding = 4) uniform subpassInputMS u_mr;

layout(push_constant) uniform PushConstants {
  mat4 view;
  vec3 camera_pos;
} push_constants;

#ifdef VULKAN
  
#else
  dicks
#endif

const float M_PI = 3.141592653589793;
const float c_MinRoughness = 0.04;

// flIP y
//const vec3 c_LightDirection = vec3(-0.4, 1.35, 0.2);
const vec3 c_LightDirection = vec3(0.5, 0.5, 0.5);
const vec3 c_LightColor = vec3(1.0,1.0,1.0);

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
  vec4 worldPosition = subpassLoad(u_position, gl_SampleID);
  
  vec4 base_colour = subpassLoad(u_colour, gl_SampleID);
  vec3 normal = subpassLoad(u_normal, gl_SampleID).rgb;
  vec4 uv = subpassLoad(u_uv, gl_SampleID);
  
  float roughness = subpassLoad(u_mr, gl_SampleID).g;
  float metallic = subpassLoad(u_mr, gl_SampleID).b;
  
  float ao = subpassLoad(u_mr, gl_SampleID).r;
  float ao_strength = subpassLoad(u_mr, gl_SampleID).a;
  vec3 emissive = vec3(worldPosition.a, uv.zw);
  
  vec3 toCameraVector = push_constants.camera_pos-worldPosition.xyz;//(inverse(push_constants.view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz - worldPosition.xyz;
  
  float alpha_roughness = roughness*roughness;
  
  vec3 f0 = vec3(0.04);
  vec3 diffuse_colour = base_colour.rgb * (vec3(1.0) - f0);
  diffuse_colour *= 1.0 - metallic;
  vec3 specular_colour = mix(f0, base_colour.rgb, metallic);
  
  float reflectance = max(max(specular_colour.r, specular_colour.g), specular_colour.b);
  
  float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
  vec3 specular_eviroment_r0 = specular_colour.rgb;
  vec3 specular_eviroment_r90 = vec3(1.0, 1.0, 1.0) * reflectance90;
  
  vec3 colour = base_colour.rgb;
  colour.rgb = vec3(1.0, 1.0, 1.0) * 0.1 * colour.rgb;
 // colour = pow(colour.rgb, vec3(1.0/2.2)); 
  
  // Lights and stuff start here
  vec3 N = normal;
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
  colour += NdotL * c_LightColor * (diffuse_contrib + specular_contrib);
  
  //colour.rgb = mix(colour.rgb, colour.rgb * ao, ao_strength);
  
  //colour.rgb += emissive;
  
  f_colour.rgb = pow(colour.rgb, vec3(1.0/2.2));
  f_colour.a =  1.0;
}
