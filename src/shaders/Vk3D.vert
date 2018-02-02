#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_uv;
layout(location = 2) out vec3 toCameraVector;
layout(location = 3) out vec2 damper_reflectivity;
layout(location = 4) out vec3 toLightVector[4];
layout(location = 8) out vec3 lightColour[4];
layout(location = 12) out vec3 attenuation[4];
layout(location = 16) out float lightType[4];

layout(set = 0, binding = 0) uniform Data {
    mat4 transformation;
    mat4 view;
    mat4 proj;
    mat4 lightpositions;
    mat4 lightcolours;
    mat4 attenuations;
} uniforms;

void main() {
    vec4 worldPosition = uniforms.transformation * vec4(position, 1.0);

    v_uv = uv;
    v_normal = mat3(transpose(inverse(uniforms.transformation))) * normal;
    damper_reflectivity = vec2(uniforms.lightcolours[0].w, uniforms.lightcolours[1].w);
    
    toCameraVector = (inverse(uniforms.view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz - worldPosition.xyz;
    
    for(int i = 0; i < 4; ++i) {
      attenuation[i]   = uniforms.attenuations[i].xyz;
      lightColour[i]   = uniforms.lightcolours[i].xyz;
      lightType[i]     = uniforms.lightcolours[i].w;
      toLightVector[i] = uniforms.lightpositions[i].xyz - worldPosition.xyz;
    }
    
    gl_Position = uniforms.proj * uniforms.view * worldPosition;
}
