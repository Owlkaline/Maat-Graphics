#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
  mat4 transformation;
  mat4 light_space;
} uniforms;

void main() {
  mat4 light_space = uniforms.light_space;
  mat4 transformation = uniforms.transformation;
  
  gl_Position = light_space * transformation * vec4(position, 1.0);
}
