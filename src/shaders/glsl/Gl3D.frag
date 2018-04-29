#version 330 core

in vec3 v_normal;
in vec2 v_uv;
in vec3 toCameraVector;
in vec2 damper_reflectivity;
in vec3 toLightVector[4];
in vec3 lightColour[4];
in vec3 attenuation[4];

out vec4 f_colour;

uniform sampler2D tex;
uniform float shine_damper;
uniform float reflectivity;

void main() {
  vec3 unitNormal = normalize(v_normal);
  vec3 unitVectorToCamera = normalize(toCameraVector);
  
  vec3 total_diffuse = vec3(0.0);
  vec3 total_specular = vec3(0.0);
  
  for(int i = 0; i < 4; ++i) {
    if(lightColour[i] == vec3(0.0)) {
      continue;
    }
    
    vec3 unitLightVector = normalize(toLightVector[i]);
    vec3 lightDirection = -unitLightVector;
    
    // Brightness
    float nDot1 = dot(unitNormal, unitLightVector);
    float brightness = max(nDot1, 0.0);
    
    float distance = length(toLightVector[i]);
    float attFactor = attenuation[i].x + (attenuation[i].y * distance) + (attenuation[i].z * distance * distance);
    
    vec3 reflectedLightDirection = reflect(lightDirection, unitNormal);
    
    float specularFactor = dot(reflectedLightDirection, unitVectorToCamera);
    specularFactor = max(specularFactor, 0.0);
    
    float dampedFactor = pow(specularFactor, shine_damper);
    
    total_diffuse += (brightness * lightColour[i]) / attFactor;
    total_specular += (dampedFactor * reflectivity * lightColour[i]) / attFactor;
  }
  
  total_diffuse = max(total_diffuse, 0.2);
  
  f_colour = vec4(total_diffuse, 1.0) * texture(tex, v_uv) + vec4(total_specular, 1.0);
}
