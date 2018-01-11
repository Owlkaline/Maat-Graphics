#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec3 toLightVector;
layout(location = 3) in vec3 lightColour;
layout(location = 4) in vec2 damper_reflectivity;
layout(location = 5) in vec3 toCameraVector;
layout(location = 6) in vec3 attenuation;

layout(location = 0) out vec4 f_colour;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
  float shinedamper = damper_reflectivity.x;
  float reflectivity = damper_reflectivity.y;

  vec3 unitNormal = normalize(v_normal);
  vec3 unitLightVector = normalize(toLightVector);
  
  float distance = length(toLightVector);
  float attFactor = attenuation.x + (attenuation.y * distance) + (attenuation.z * distance * distance);
  
  // Brightness
  float nDot1 = dot(unitNormal, unitLightVector);
  float brightness = max(nDot1, 0.2);
  
  // Start Cell shading
  //float level = floor(brightness*4.0);
  //brightness = level/4;
  // Do same for damped factor
  // End Cell Shading
  
  vec3 diffuse = (brightness * lightColour) / attFactor;
  
  vec3 unitVectorToCamera = normalize(toCameraVector);
  vec3 lightDirection = -unitLightVector;
  
  vec3 reflectedLightDirection = reflect(lightDirection, unitNormal);
  
  float specularFactor = dot(reflectedLightDirection, unitVectorToCamera);
  specularFactor = max(specularFactor, 0.0);
  
  float dampedFactor = pow(specularFactor, shinedamper);
  
  vec3 finalSpecular = (dampedFactor * reflectivity * lightColour) / attFactor;
  
  f_colour = vec4(diffuse, 1.0) * texture(tex, v_uv) + vec4(finalSpecular, 1.0);
}
