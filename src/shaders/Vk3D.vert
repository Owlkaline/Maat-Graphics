#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_uv;
layout(location = 2) out vec3 toLightVector;
layout(location = 3) out vec3 lightColour;
layout(location = 4) out vec2 damper_reflectivity;
layout(location = 5) out vec3 toCameraVector;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    // light position
    // light colour
    // vec2 shineDamper and reflectivity
} uniforms;

// to be uniforms
const vec3 LIGHT_POS = vec3(10.0, 0.0, 50.0);
const vec3 LIGHT_COLOUR = vec3(1.0, 1.0, 1.0);
const vec2 shinedamper_refectivity = vec2(10.0, 1.0);

void main() {
    vec4 worldPosition = uniforms.world * vec4(position, 1.0);
    
    gl_Position = uniforms.proj * uniforms.view * worldPosition;
    
    v_uv = uv;
    v_normal = (uniforms.world * vec4(normal, 0.0)).xyz;
    
    lightColour = LIGHT_COLOUR;
    toLightVector = LIGHT_POS - worldPosition.xyz;
    
    toCameraVector = (inverse(uniforms.view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz - worldPosition.xyz;
    
    damper_reflectivity = shinedamper_refectivity;
}
