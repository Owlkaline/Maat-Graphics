#version 450

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 uv;
layout (location = 2) in vec4 colour;

layout (location = 0) out vec2 o_uv;
layout (location = 1) out vec4 o_colour;

//uniform vec2 translation;
layout(push_constant) uniform PushConstants {
  vec2 translation;
  vec2 window_size;
} push_constants;

mat4 ortho_projection(float bottom, float top, float left, float right, float near, float far) {
  mat4 projection = mat4(
    vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
    vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
    vec4(0.0, 0.0, -2.0 / (far - near), 0.0),
    vec4(-(right + left) / (right - left), -(top + bottom) / (top - bottom), -(far + near)/(far - near), 1)
  );

  return projection;
}

void main(){
  o_uv = uv.xy;
  o_colour = colour;

  mat4 ortho_matrix = ortho_projection(0.0, push_constants.window_size.y, 0.0, push_constants.window_size.x, 0.1, 1.0);

  //vec2 translation = vec2(0.5, 0.5);//push_constants.translation;//vec2(0.5, 0.5);
  //gl_Position = vec4(position.xy + translation * vec2(2.0, -2.0), 0.0, 1.0);

  gl_Position = ortho_matrix * vec4(position.xy + push_constants.translation, -1.0, 1.0);
}
