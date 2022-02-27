#version 450

// Vertex Input
layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 colour;
layout (location = 2) in vec2 uv;

layout (location = 0) out vec2 o_uv;
layout (location = 1) out vec4 o_colour;

// Instaced Data
layout (location = 3) in vec2 world_position;
layout (location = 4) in vec2 size;
layout (location = 5) in vec4 letter_uvs;
layout (location = 6) in float dummy;
layout (location = 7) in vec4 kerning_offset;
//layout (location = 6) in float text_height;
//layout (location = 7) in vec4 text_colour;
//layout (location = 8) in vec4 outline_colour;
//layout (location = 9) in vec4 edge_width;
//
//layout (location = 0) out vec2 o_uv;
//layout (location = 1) out vec4 o_colour;
//layout (location = 2) out vec4 o_outline_colour;
//layout (location = 3) out vec4 o_outline_width;
//
layout (set = 0, binding = 0) uniform UBO {
  vec2 window_size;
} ubo;
//
//layout(push_constant) uniform PushConstants {
//  vec4 attrib0;
//  vec4 attrib1;
//  vec4 attrib2;
//  vec4 attrib3;
//  vec4 attrib4;
//  vec4 attrib5;
//  vec4 attrib6;
//  vec4 attrib7;
//} push_constants;
//
mat4 ortho_projection(float bottom, float top, float left, float right, float near, float far) {
  mat4 projection = mat4(
    vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
    vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
    vec4(0.0, 0.0, -2.0 / (far - near), 0.0),
    vec4(-(right + left) / (right - left), -(top + bottom) / (top - bottom), -(far + near)/(far - near), 1)
  );

  return projection;
}

void main() {

  mat4 ortho_matrix = ortho_projection(0.0, ubo.window_size.y, 0.0, ubo.window_size.x, 0.1, 1.0);

  if (uv.x > 0.0) {
    o_uv.x = letter_uvs.x;
  } else {
    o_uv.x = letter_uvs.z;
  }

  if (uv.y > 0.0) {
    o_uv.y = letter_uvs.y;
  } else {
    o_uv.y = letter_uvs.w;
  }

  o_colour = vec4(0.0, 0.0, 0.0, 1.0);

  vec4 new_pos = vec4(pos.x, pos.y + kerning_offset.y, pos.zw);

  gl_Position = ortho_matrix * vec4(new_pos.xy * size.xy + world_position.xy, pos.zw);
}

//
//void main() {
//  //vec2 position;
//  //position.x = (gl_VertexIndex == 0 || gl_VertexIndex == 1)? 1.0 : 0.0;
//  //position.y = (gl_VertexIndex == 0 || gl_VertexIndex == 3)? 0.0: 1.0;
//
//  //vec2 glyphSize = size;
//  //vec2 glyphOffset = offset;
//  //glyphOffset.y = position.y - glyphOffset.y;
//
//  //vec2 finalPosition = glyphSize * position + glyphOffset;
//
//  //gl_Position = vec4(finalPosition, 0.0, 1.0);
//
//  //vec2 uvOffset = letter_uvs.xy;
//  //vec2 uvSize = letter_uvs.zw;
//  //o_uv = uvOffset + position*uvSize;
//
//  if (uv.x > 0.0) {
//    o_uv.x = letter_uvs.x;
//  } else {
//    o_uv.x = letter_uvs.z;
//  }
//
//  if (uv.y > 0.0) {
//    o_uv.y = letter_uvs.y;
//  } else {
//    o_uv.y = letter_uvs.w;
//  }
//  
//  o_colour = text_colour;
//  o_outline_colour = outline_colour;
//  o_outline_width = edge_width;
//  
//  mat4 ortho_matrix = ortho_projection(0.0, ubo.window_size.y, 0.0, ubo.window_size.x, 0.1, 1.0);
//  
//  float x = pos.x * size.x + offset.x;
//  float y = pos.y * size.y + offset.y;
//
//  gl_Position = ortho_matrix * vec4(x, y, pos.zw);  
//}
