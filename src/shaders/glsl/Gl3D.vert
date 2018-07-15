#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 tangent;
layout(location = 3) in vec2 uv;
layout(location = 4) in vec4 colour;

out vec3 v_position;
out vec3 v_normal;
out vec4 v_tangent;
out vec2 v_uv;
out vec4 v_colours;
out mat3 v_tbn;
out vec3 toCameraVector;

uniform mat4 u_transformation;
uniform mat4 u_view;
uniform mat4 u_projection;

void main() {
  vec3 position = vec3(-position.x, position.yz);
  vec3 normal = vec3(-normal.x, normal.yz);
  vec4 worldPosition = u_transformation * vec4(position, 1.0);
  
  v_position = vec3(worldPosition.xyz) / worldPosition.w;
  v_uv = vec2(uv.x, uv.y);
  v_colours = colour;
  v_normal = normalize(vec3(u_transformation * vec4(normal.xyz, 0.0)));
  v_tangent = normalize(vec4(u_transformation * tangent));
  
  vec3 normalW = v_normal;
  vec3 tangentW = normalize(vec3(u_transformation * vec4(tangent.xyz, 0.0)));
  vec3 bitangentW = cross(normalW, tangentW) * tangent.w;
  v_tbn = mat3(tangentW, bitangentW, normalW);
  
  toCameraVector = (inverse(u_view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz - worldPosition.xyz;
  
  gl_Position = u_projection * u_view * worldPosition;
}
