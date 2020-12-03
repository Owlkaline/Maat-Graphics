#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

// Instanced
layout(location = 2) in vec4 model; // vec4(x, y, scale, window_width)
layout(location = 3) in vec4 letter_uv; 
layout(location = 4) in vec4 edge_width; 
layout(location = 5) in vec4 colour; //vec4(r,g,b, a)
layout(location = 6) in vec4 outline_colour; //vec4(r,g,b, window_height)

layout(location = 0) out vec2 v_uvs;
layout(location = 1) out vec4 v_new_colour;
layout(location = 2) out vec3 v_outlineColour;
layout(location = 3) out vec4 v_edge_width;

vec4 when_eq(vec4 x, vec4 y) {
  return 1.0 - abs(sign(x - y));
}

vec4 when_neq(vec4 x, vec4 y) {
  return abs(sign(x - y));
}

vec4 when_gt(vec4 x, vec4 y) {
  return max(sign(x - y), 0.0);
}

vec4 when_lt(vec4 x, vec4 y) {
  return max(sign(y - x), 0.0);
}

vec4 when_ge(vec4 x, vec4 y) {
  return 1.0 - when_lt(x, y);
}

vec4 when_le(vec4 x, vec4 y) {
  return 1.0 - when_gt(x, y);
}

vec4 and(vec4 a, vec4 b) {
  return a * b;
}

vec4 or(vec4 a, vec4 b) {
  return min(a + b, 1.0);
}

vec4 not(vec4 a) {
  return 1.0 - a;
}

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
  
  mat4 ortho = mat4(vec4(2.0 / (right - left), 0.0, 0.0, 0.0),
                    vec4(0.0, 2.0 / (top - bottom), 0.0, 0.0),
                    vec4(0.0, 0.0, -2.0 / (near / far), 0.0),
                    vec4(-(right + left) / (right - left), -(top+bottom)/(top-bottom), 0.0, 1.0));
  
  return ortho;
}

void main() {
  vec2 new_uv = uv;
  vec2 new_pos = position;
  
  float text_scale = model.z;
  vec2 text_pos = model.xy;
  float window_width = model.w;
  float window_height = outline_colour.w;
  
  //if(uv.x == 0) {
  //  new_uv.x += letter_uv.x;
  //  new_pos.x = 0;
  //} else {//if(uv.x == 1) {
  //  new_uv.x = letter_uv.z;
  //  new_pos.x = letter_uv.z - letter_uv.x;
  //}
  
  vec4 uvx_eq_zero = when_eq(vec4(uv.x), vec4(0.0));
  
  new_uv.x =     uvx_eq_zero.x  * (new_uv.x + letter_uv.x) +
             not(uvx_eq_zero).x * (letter_uv.z);
  new_pos.x =    uvx_eq_zero.x *  (0) +
             not(uvx_eq_zero).x * (letter_uv.z - letter_uv.x);
  
  //if(uv.y == 0) {
  //  new_uv.y = letter_uv.w;
  //  new_pos.y = 0;
  //} else { //if(uv.y == 1) {
  //  new_uv.y = letter_uv.y;
  //  new_pos.y = letter_uv.w - letter_uv.y;
  //}
  
  vec4 uvy_eq_zero = when_eq(vec4(uv.y), vec4(0.0));
  
  new_uv.y =     uvy_eq_zero.x  * (letter_uv.w) +
             not(uvy_eq_zero).x * (letter_uv.y);
  new_pos.y =    uvy_eq_zero.x *  (0) +
             not(uvy_eq_zero).x * (letter_uv.w - letter_uv.y);
  
  mat4 model_matrix = create_translation_matrix(text_pos, text_scale);
  mat4 projection = create_ortho_projection(1.0, -1.0, window_width, window_height);
  
  v_uvs = new_uv;
  v_outlineColour = outline_colour.rgb;
  v_new_colour = colour;
  v_edge_width = edge_width;
  
  gl_Position = projection * model_matrix * vec4(new_pos, 0.0, 1.0);
}
