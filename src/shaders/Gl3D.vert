#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

out vec3 v_normal;
out vec2 v_uv;
out vec3 toCameraVector;
out vec2 damper_reflectivity;
out vec3 toLightVector[4];
out vec3 lightColour[4];
out vec3 attenuation[4];

uniform mat4 transformation;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 lightpositions;
uniform mat4 lightcolours;
uniform mat4 attenuations;

void main() {
    vec4 worldPosition = transformation * vec4(position, 1.0);

    v_uv = uv;
    v_normal = mat3(transpose(inverse(transformation))) * normal;
    
    toCameraVector = (inverse(view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz - worldPosition.xyz;
    
    for(int i = 0; i < 4; ++i) {
      attenuation[i]   = attenuations[i].xyz;
      lightColour[i]   = lightcolours[i].xyz;
      toLightVector[i] = lightpositions[i].xyz - worldPosition.xyz;
    }
    
    gl_Position = projection * view * worldPosition;
}
