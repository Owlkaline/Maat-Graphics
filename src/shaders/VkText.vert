#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 new_colour;
layout(location = 2) out vec3 outlineColour;
layout(location = 3) out vec4 edge_width;

layout(set = 0, binding = 1) uniform Data {
  mat4 projection;
  mat4 model;
  vec4 letter_uv;
  vec4 edge_width;
  vec4 colour;
  vec3 outlineColour;
} uniforms;

void main() {
  vec2 new_uv = uv;
  vec2 new_pos = position;
  
  if(uv.x == 0) {
    new_uv.x += uniforms.letter_uv.x;
    new_pos.x = 0;
  } else
  if(uv.x == 1) {
    new_uv.x = uniforms.letter_uv.z;
    new_pos.x = uniforms.letter_uv.z - uniforms.letter_uv.x;
  }
  
  if(uv.y == 0) {
    new_uv.y = uniforms.letter_uv.w;
    new_pos.y = 0;
  } else
  if(uv.y == 1) {
    new_uv.y = uniforms.letter_uv.y;
    new_pos.y = uniforms.letter_uv.w - uniforms.letter_uv.y;
  }
  
  gl_Position = uniforms.projection * uniforms.model * vec4(new_pos, 0.0, 1.0);
  
  uvs = new_uv;
  outlineColour = uniforms.outlineColour;
  new_colour = uniforms.colour;
  edge_width = uniforms.edge_width;
}
