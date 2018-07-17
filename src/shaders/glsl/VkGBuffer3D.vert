#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 tangent;
layout(location = 3) in vec2 uv;
layout(location = 4) in vec4 colour;

layout(location = 0) out vec3 v_position;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec4 v_tangent;
layout(location = 3) out vec2 v_uv;
layout(location = 4) out vec4 v_colours;
layout(location = 5) out mat3 v_tbn;

layout(set = 0, binding = 0) uniform Data {
    mat4 transformation;
    mat4 view;
    mat4 proj;
    mat4 lightpositions;
    mat4 lightcolours;
    mat4 attenuations;
} uniforms;

void main() {
    vec3 position = vec3(-position.x, position.yz);
    vec3 normal = vec3(-normal.x, normal.yz);
    
    vec4 worldPosition = uniforms.transformation * vec4(position, 1.0);
    
    v_position = vec3(worldPosition.xyz) / worldPosition.w;
    v_uv = vec2(uv.x, uv.y);
    v_colours = colour;
    
    v_normal = normalize(vec3(uniforms.transformation * vec4(normal.xyz, 0.0)));
    v_tangent = normalize(vec4(uniforms.transformation * tangent));
    
    vec3 normalW = v_normal;
    vec3 tangentW = normalize(vec3(uniforms.transformation * vec4(tangent.xyz, 0.0)));
    vec3 bitangentW = cross(normalW, tangentW) * tangent.w;
    v_tbn = mat3(tangentW, bitangentW, normalW);
    
    gl_Position = uniforms.proj * uniforms.view * worldPosition;
}
//3905
