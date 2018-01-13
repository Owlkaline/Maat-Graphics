#version 330 core

layout(location = 0) in vec3 position;
//layout(location = 1) in vec3 normal;
//layout(location = 2) in vec2 uv;

out float camera_dist;

uniform mat4 transformation;
uniform mat4 view;
uniform mat4 proj;

void main() {
 // mat4 worldview = view * transformation;
  vec4 worldPosition = transformation * vec4(position, 1.0); 
  
  gl_Position = proj * view * worldPosition;
}
