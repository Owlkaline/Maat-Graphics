#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 new_colour;
layout(location = 2) out vec2 textured_blackwhite;

layout(set = 0, binding = 1) uniform Data {
  mat4 projection;
} uniforms;

layout(push_constant) uniform PushConstants {
  mat4 model;
  vec4 colour;
  vec4 has_texture_blackwhite;
} push_constants;

void main() {
  uvs = uv;
  new_colour = push_constants.colour;
  
  textured_blackwhite = push_constants.has_texture_blackwhite.xy;
  
  gl_Position = uniforms.projection * push_constants.model * vec4(position, 0.0, 1.0);
}
