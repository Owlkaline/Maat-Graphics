#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 v_uvs;
layout(location = 1) out vec4 v_new_colour;
layout(location = 2) out vec3 v_outlineColour;
layout(location = 3) out vec4 v_edge_width;

layout(set = 0, binding = 1) uniform Data {
  mat4 scale;
  mat4 projection;
} uniforms;

layout(set = 0, binding = 2) uniform DrawData {
  mat4 model;
  vec4 letter_uv;
  vec4 edge_width; 
  vec4 colour;
  vec4 outline_colour;
} draw_uniforms;

void main() {
  vec2 new_uv = uv;
  vec2 new_pos = position;
  
  if(uv.x == 0) {
    new_uv.x += draw_uniforms.letter_uv.x;
    new_pos.x = 0;
  } else
  if(uv.x == 1) {
    new_uv.x = draw_uniforms.letter_uv.z;
    new_pos.x = draw_uniforms.letter_uv.z - draw_uniforms.letter_uv.x;
  }
  
  if(uv.y == 0) {
    new_uv.y = draw_uniforms.letter_uv.w;
    new_pos.y = 0;
  } else
  if(uv.y == 1) {
    new_uv.y = draw_uniforms.letter_uv.y;
    new_pos.y = draw_uniforms.letter_uv.w - draw_uniforms.letter_uv.y;
  }
  
  gl_Position = uniforms.projection * uniforms.scale * draw_uniforms.model * vec4(new_pos, 0.0, 1.0);
  
  v_uvs = new_uv;
  v_outlineColour = draw_uniforms.outline_colour.rgb;
  v_new_colour = draw_uniforms.colour;
  v_edge_width = draw_uniforms.edge_width;
}
