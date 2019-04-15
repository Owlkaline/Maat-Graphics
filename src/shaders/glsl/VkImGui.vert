#version 450

layout(push_constant) uniform PushConstants {
  vec4 model;
} push_constants;

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 colour;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 colours;

mat4 create_imgui_matrix() {
  vec2 size = vec2(1280.0, 720.0);//push_constants.model.xy;
  
  mat4 imgui_matrix = mat4(
    vec4(2.0/size.x, 0.0, 0.0, 0.0),
    vec4(0.0, 2.0/size.y, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(-1.0, -1.0, 0.0, 1.0)
  );
  
  return imgui_matrix;
}

void main() {
  uvs = uv;
  colours = colour;
  gl_Position = create_imgui_matrix() * vec4(position.x, position.y, 0, 1);
}
