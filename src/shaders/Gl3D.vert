#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

out vec3 v_normal;
out vec2 v_uv;

uniform mat4 transformation;
uniform mat4 view;
uniform mat4 proj;

void main() {
  v_normal = normal;
  v_uv = uv;
  
  vec4 worldPosition = transformation * vec4(position, 1.0); 
  
  gl_Position = proj * view * worldPosition;
}
