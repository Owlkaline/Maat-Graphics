#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_uv;
layout(location = 2) out vec3 toLightVector;
layout(location = 3) out vec3 toCameraVector;
layout(location = 4) out vec3 lightColour;
layout(location = 5) out vec3 attenuation;
layout(location = 6) out vec2 damper_reflectivity;

layout(set = 0, binding = 0) uniform Data {
    mat4 transformation;
    mat4 view;
    mat4 proj;
    vec4 lightposition_shinedamper;
    vec4 lightcolour_reflectivity;
    vec4 attenuation;
} uniforms;

void main() {
    damper_reflectivity = vec2(uniforms.lightposition_shinedamper.w, uniforms.lightcolour_reflectivity.w);
    attenuation = uniforms.attenuation.xyz;
    lightColour = uniforms.lightcolour_reflectivity.xyz;
    
    v_uv = uv;
    v_normal = (uniforms.transformation * vec4(normal, 0.0)).xyz;
    
    vec4 worldPosition = uniforms.transformation * vec4(position, 1.0);
    toLightVector = uniforms.lightposition_shinedamper.xyz - worldPosition.xyz;
    toCameraVector = (inverse(uniforms.view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz - worldPosition.xyz;
    
    gl_Position = uniforms.proj * uniforms.view * worldPosition;
}
