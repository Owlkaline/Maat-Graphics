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
layout(location = 5) out vec3 toCameraVector;
layout(location = 6) out vec4 v_position_light_space;
layout(location = 7) out vec3 toLightVector[4];
layout(location = 11) out vec3 lightColour[4];
layout(location = 15) out vec3 attenuation[4];
layout(location = 19) out float lightType[4];
layout(location = 23) out mat3 v_tbn;

layout(set = 0, binding = 0) uniform Data {
    mat4 transformation;
    mat4 view;
    mat4 proj;
    mat4 lightpositions;
    mat4 lightcolours;
    mat4 lightspace_matrix;
    mat4 attenuations;
} uniforms;

void main() {
    vec3 position = vec3(-position.x, position.yz);
    vec3 normal = vec3(-normal.x, normal.yz);
    
    vec4 worldPosition = uniforms.transformation * vec4(position, 1.0);
    
    v_position = vec3(worldPosition.xyz) / worldPosition.w;
    v_uv = vec2(uv.x, uv.y);
    // v_normal = mat3(transpose(inverse(uniforms.transformation))) * normal;
    //v_tangent = mat3(transpose(inverse(uniforms.transformation))) * tangent;
    v_colours = colour;
    
    //normal.y *= -1.0;
    v_normal = normalize(vec3(uniforms.transformation * vec4(normal.xyz, 0.0)));
    v_tangent = normalize(vec4(uniforms.transformation * tangent));
    
    vec3 normalW = v_normal;
    vec3 tangentW = normalize(vec3(uniforms.transformation * vec4(tangent.xyz, 0.0)));
    vec3 bitangentW = cross(normalW, tangentW) * tangent.w;
    v_tbn = mat3(tangentW, bitangentW, normalW);
    
    toCameraVector = (inverse(uniforms.view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz - worldPosition.xyz;
    
    for(int i = 0; i < 4; ++i) {
      attenuation[i]   = uniforms.attenuations[i].xyz;
      lightColour[i]   = uniforms.lightcolours[i].xyz;
      lightType[i]     = uniforms.lightcolours[i].w;
      toLightVector[i] = uniforms.lightpositions[i].xyz - worldPosition.xyz;
    }
    
    v_position_light_space = uniforms.lightspace_matrix * vec4(worldPosition.xyz, 1.0);
    
    gl_Position = uniforms.proj * uniforms.view * worldPosition;
}
//3905
