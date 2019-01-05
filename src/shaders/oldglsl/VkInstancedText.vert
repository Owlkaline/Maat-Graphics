#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in mat4 model;
layout(location = 6) in vec4 letter_uv;
layout(location = 7) in vec4 edge_width;
layout(location = 8) in vec4 colour;
layout(location = 9) in vec4 outline_colour;

layout(location = 0) out vec2 v_uvs;
layout(location = 1) out vec4 v_new_colour;
layout(location = 2) out vec3 v_outlineColour;
layout(location = 3) out vec4 v_edge_width;

layout(set = 0, binding = 1) uniform Data {
  mat4 scale;
  mat4 projection;
} uniforms;

void main() {
  vec2 new_uv = uv;
  vec2 new_pos = position;
  
  if(uv.x == 0) {
    new_uv.x += letter_uv.x;
    new_pos.x = 0;
  } else
  if(uv.x == 1) {
    new_uv.x = letter_uv.z;
    new_pos.x = letter_uv.z - letter_uv.x;
  }
  
  if(uv.y == 0) {
    new_uv.y = letter_uv.w;
    new_pos.y = 0;
  } else
  if(uv.y == 1) {
    new_uv.y = letter_uv.y;
    new_pos.y = letter_uv.w - letter_uv.y;
  }
  
  gl_Position = uniforms.projection * uniforms.scale * model * vec4(new_pos, 0.0, 1.0);
  
  v_uvs = new_uv;
  v_outlineColour = outline_colour.rgb;
  v_new_colour = colour;
  v_edge_width = edge_width;
}
