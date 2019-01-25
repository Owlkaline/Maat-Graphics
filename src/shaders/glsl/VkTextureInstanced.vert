#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

// Instanced.
layout(location = 2) in vec4 model;//x y scale rot
layout(location = 4) in vec4 sprite_sheet; // block_x, block_y, num_of_rows, image_scale

layout(location = 0) out vec2 uvs;

// 128 bytes, float 4 bytes
layout(push_constant) uniform PushConstants {
  vec4 projection;
} push_constants;

void main() {
  float num_rows = sprite_sheet.z;
  float block_x = sprite_sheet.x;
  float block_y = sprite_sheet.y;
  float scale = sprite_sheet.w;
  
  vec2 texture_pos = model.xy;
  float texture_scale = model.z;
  float rotation = model.w;
  
  if (num_rows < 0.0) {
    num_rows *= -1;
  }
  
  vec2 texcoords = uv.xy;
  texcoords += vec2(block_x, block_y);
  texcoords /= num_rows;
  uvs = texcoords;
  
  float x_offset = push_constants.projection.x;
  float y_offset = push_constants.projection.y;
  float near = 1.0;
  float far = -1.0;
  float right = x_offset + push_constants.projection.z;
  float left = x_offset;
  float bottom = y_offset + push_constants.projection.w;
  float top = y_offset;
  mat4 projection = mat4(vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
                          vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
                          vec4(0.0, 0.0, -2.0 / (near / far), 0.0),
                          vec4(-(right + left) / (right - left), -(top+bottom)/(top-bottom), 0.0, 1.0));
  
  
  float s = sin(rotation);
  float c = cos(rotation);
  float oc = 1.0 - c;
  
  mat4 rotation_matrix = mat4(vec4(c,   0.0, 0.0,  0.0), 
                              vec4(s,   c,   0.0,  0.0), 
                              vec4(0.0, 0.0, oc+c, 0.0), 
                              vec4(0.0, 0.0,  0.0,  1.0));
  
  mat4 model_matrix = mat4(vec4(texture_scale,  0.0,           0.0, 0.0), 
                              vec4(0.0,         texture_scale, 0.0, 0.0), 
                              vec4(0.0,         0.0,           1.0, 0.0), 
                              vec4(texture_pos,                0.0, 1.0));
                              
  mat4 scale_matrix = mat4(vec4(scale,  0.0,   0.0, 0.0), 
                              vec4(0.0, scale, 0.0, 0.0), 
                              vec4(0.0, 0.0,   1.0, 0.0), 
                              vec4(0.0, 0.0,   0.0, 1.0));
  
  gl_Position = projection * scale_matrix * model_matrix * rotation_matrix * vec4(position, 0.0, 1.0);
}
