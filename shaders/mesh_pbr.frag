#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable


layout (location = 0) in vec3 o_normal;
layout (location = 1) in vec3 o_colour;
layout (location = 2) in vec2 o_uv;
layout (location = 3) in vec3 o_view_vec;
layout (location = 4) in vec3 o_light_vec;

layout (location = 0) out vec4 uFragColor;

layout (set = 2, binding = 0) uniform UBO {
  vec4 base_colour_factor;
  float roughness;
  float metallic;
  float double_sided;
  vec3 emissive;
} pbr_ubo;

layout (set = 2, binding = 1) uniform sampler2D base_colour;
layout (set = 2, binding = 2) uniform sampler2D normal_map;
layout (set = 2, binding = 3) uniform sampler2D metallic_roughness;
layout (set = 2, binding = 4) uniform sampler2D occlusion;
layout (set = 2, binding = 5) uniform sampler2D emissive;

const float M_PI = 3.141592653589793;
const float c_MinRoughness = 0.04;

vec3 diffuse(vec3 diffuse_colour) {
  return diffuse_colour / M_PI;
}

// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
float microfacetDistribution(float alpha_roughness, float NdotH) {
  float roughnessSq = alpha_roughness * alpha_roughness;
  float f = (NdotH * roughnessSq - NdotH) * NdotH + 1.0;
  
  return roughnessSq / (M_PI * f * f);
}

// This calculates the specular geometric attenuation (aka G()),
// where rougher material will reflect less light back to the viewer.
// This implementation is based on [1] Equation 4, and we adopt their modifications to
// alphaRoughness as input as originally proposed in [2].
float geometricOcclusion(float NdotL, float NdotV, float alpha_roughness) {
  float r = alpha_roughness;
  
  float attenuationL = 2.0 * NdotL / (NdotL + sqrt(r * r + (1.0 - r * r) * (NdotL * NdotL)));
  float attenuationV = 2.0 * NdotV / (NdotV + sqrt(r * r + (1.0 - r * r) * (NdotV * NdotV)));
  return attenuationL * attenuationV;
}

//vec3 specularReflection(PBRInfo pbrInputs) {
//	return pbrInputs.reflectance0 + (pbrInputs.reflectance90 - pbrInputs.reflectance0) * pow(clamp(1.0 - pbrInputs.VdotH, 0.0, 1.0), 5.0);
//}

void main() {
  float perceptualRoughness;
  float metallic;
  vec3 diffuseColor;
  vec4 baseColor = texture(base_colour, o_uv) * vec4(o_colour, 1.0) * pbr_ubo.base_colour_factor;

  vec3 f0 = vec3(0.04);

  perceptualRoughness = 0.6; //material.roughnessFactor;
  metallic = 0.4; //material.metallicFactor;
  
  perceptualRoughness = clamp(perceptualRoughness, c_MinRoughness, 1.0);
  metallic = clamp(metallic, 0.0, 1.0);
  

  diffuseColor = baseColor.rgb * (vec3(1.0) - f0);
  diffuseColor *= 1.0 - metallic;
  
  float alpha_roughness = perceptualRoughness * perceptualRoughness;

  vec3 specularColor = mix(f0, baseColor.rgb, metallic);

  // Compute reflectance.
  float reflectance = max(max(specularColor.r, specularColor.g), specularColor.b);
  
  // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
  // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
  float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
  //vec3 specularEnvironmentR0 = specularColor.rgb;
  //vec3 specularEnvironmentR90 = vec3(1.0, 1.0, 1.0) * reflectance90;
  
  vec3 n = normalize(o_normal);
  vec3 v = normalize(o_view_vec);    // Vector from surface point to camera
  vec3 l = normalize(o_light_vec);     // Vector from surface point to light
  vec3 h = normalize(l+v);                        // Half vector between both l and v
  vec3 reflection = -normalize(reflect(v, n));
  reflection.y *= -1.0f;
  
  float NdotL = clamp(dot(n, l), 0.001, 1.0);
  float NdotV = clamp(abs(dot(n, v)), 0.001, 1.0);
  float NdotH = clamp(dot(n, h), 0.0, 1.0);
  float LdotH = clamp(dot(l, h), 0.0, 1.0);
  float VdotH = clamp(dot(v, h), 0.0, 1.0);

  //vec3 F = specularReflection(pbrInputs);
  float G = geometricOcclusion(NdotL, NdotV, alpha_roughness);
  float D = microfacetDistribution(alpha_roughness, NdotH);
  
  const vec3 u_LightColor = vec3(1.0);

	// Calculation of analytical lighting contribution
	vec3 diffuseContrib = (1.0 - 0.0) * diffuse(diffuseColor); // (1.0 - F) * diffuse(diffuseColor);
	vec3 specContrib = vec3(1.0) * G * D / (4.0 * NdotL * NdotV);  //F * G * D / (4.0 * NdotL * NdotV);
	// Obtain final intensity as reflectance (BRDF) scaled by the energy of the light (cosine law)
	vec3 color = NdotL * u_LightColor * (diffuseContrib + specContrib);

	// Calculate lighting contribution from image based lighting source (IBL)
	// color += getIBLContribution(pbrInputs, n, reflection);
  color += diffuseColor;//baseColor.rgb;
  
  

  uFragColor = vec4(color, baseColor.a);








  //vec4 colour = texture(base_colour, o_uv) * vec4(o_colour, 1.0);
  //
  //vec3 N = normalize(o_normal);
  //vec3 L = normalize(o_light_vec);
  //vec3 V = normalize(o_view_vec);
  //vec3 R = reflect(-L, N);
  //vec3 diffuse = max(dot(N, L), 0.15) * o_colour;
  //vec3 specular = pow(max(dot(R, V), 0.0), 16.0) * vec3(0.75);
  //
  //uFragColor = vec4(diffuse * colour.rgb + specular, 1.0);
}
