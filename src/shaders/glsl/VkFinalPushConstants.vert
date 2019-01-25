#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;

layout(push_constant) uniform PushConstants {
  vec4 model;
  vec4 projection;
} push_constants;

mat4 create_translation_matrix(vec2 pos, float scale) {
  mat4 translate_matrix = mat4(vec4(scale, 0.0,   0.0, 0.0), 
                               vec4(0.0,   scale, 0.0, 0.0), 
                               vec4(0.0,   0.0,   1.0, 0.0), 
                               vec4(pos,          0.0, 1.0));
  
  return translate_matrix;
}

mat4 create_ortho_projection(float near, float far, float right, float bottom) {
  float left = 0.0;
  float top = 0.0;
  right += left;
  bottom += top;
  
  mat4 ortho = mat4(vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
                    vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
                    vec4(0.0, 0.0, -2.0 / (near / far), 0.0),
                    vec4(-(right + left) / (right - left), -(top+bottom)/(top-bottom), 0.0, 1.0));
  
  return ortho;
}

void main() {
  float x      = push_constants.model.x;
  float y      = push_constants.model.y;
  float scale  = push_constants.model.z;
  
  float near   = push_constants.projection.x;
  float far    = push_constants.projection.y;
  float right  = push_constants.projection.z;
  float bottom = push_constants.projection.w; 
  
  mat4 projection = create_ortho_projection(near, far, right, bottom);
  mat4 model = create_translation_matrix(vec2(x, y), scale);
  
  uvs = uv;
  gl_Position = model * vec4(position, 0.0, 1.0);
}
