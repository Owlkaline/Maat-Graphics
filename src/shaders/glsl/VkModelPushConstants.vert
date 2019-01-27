#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec4 colour;
layout(location = 4) in vec4 tangent;

layout(location = 0) out vec2 uvs;

layout(push_constant) uniform PushConstants {
  mat4 view;
  vec4 model; // x, y, z, scale,
  vec4 projection; // angle_of_view, right, top, 
} push_constants;

const float M_PI = 3.141592653589793;

mat4 create_perspective_matrix(float near, float far, float angle_of_view) {
  float scale = 1.0 / tan(angle_of_view * 0.5 * M_PI / 180.0);
  mat4 perspective = mat4(
                      vec4(scale, 0.0,   0.0,                    0.0),
                      vec4(0.0,   scale, 0.0,                    0.0),
                      vec4(0.0,   0.0,   -far / (far-near),     -1.0),
                      vec4(0.0,   0.0,   -far*near / (far-near), 0.0)
                    );
                
  return perspective;
}

mat4 create_translation_matrix(vec3 pos, float scale) {
  mat4 translate_matrix = mat4(vec4(scale, 0.0,   0.0, 0.0), 
                               vec4(0.0,   scale, 0.0, 0.0), 
                               vec4(0.0,   0.0,   scale, 0.0), 
                               vec4(pos,               1.0));
  
  return translate_matrix;
}

void main() {
  mat4 projection = create_perspective_matrix(0.1, 100.0, 90.0);
  mat4 model = create_translation_matrix(push_constants.model.xyz, push_constants.model.w);
  
  uvs = uv;
  gl_Position = projection * push_constants.view * model * vec4(position, 1.0);
}
