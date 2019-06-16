#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(set = 0, binding = 4) uniform UniformBuffer {
  vec4 c_position; // x, y, z, fov
  vec4 c_center;   // x, y, z, width
  vec4 c_up;       // x, y, z, height
} uniforms;

//From vertex shader
layout(push_constant) uniform PushConstants {
  vec4 light1_position; // x, y, z, intensity1
  vec4 light1_colour; // r, g, b, _
  vec4 light2_position; // x,y,z, intensity2
  vec4 light2_colour; // r, g, b, _
  vec4 light3_position; // x,y,z, intensity3
  vec4 light3_colour; // r,g,b, _
  // 8 floats spare
  vec4 sun_direction; // xyz, _
  vec4 sun_colour; // rgb intensity
} push_constants;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec4 v_camera_pos;
layout(location = 2) out vec4 v_camera_center;
layout(location = 3) out vec4 v_camera_up;
layout(location = 4) out vec3 v_light_positions[3];
layout(location = 7) out vec3 v_light_colours[3];
layout(location = 10) out float v_light_intensity[3];
layout(location = 13) out vec3 v_sun_direction;
layout(location = 14) out vec4 v_sun_colour;

// center is a point not a direction
mat4 create_view_matrix(vec3 eye, vec3 center, vec3 up) {
  vec3 dir = center - eye;
  
  vec3 f = normalize(dir);
  vec3 s = normalize(cross(f, up));
  vec3 u = cross(s,f);
  
  mat4 look_at_matrix = mat4(vec4(s.x,           u.x,        -f.x,         0.0), 
                             vec4(s.y,           u.y,        -f.y,         0.0), 
                             vec4(s.z,           u.z,        -f.z,         0.0), 
                             vec4(-dot(eye, s), -dot(eye, u), dot(eye, f), 1.0));
  
  return look_at_matrix;
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
  uvs = uv;
  v_camera_pos = uniforms.c_position;
  v_camera_center = uniforms.c_center;
  v_camera_up = uniforms.c_up;
  v_light_positions[0] = push_constants.light1_position.xyz;
  v_light_positions[1] = push_constants.light2_position.xyz;
  v_light_positions[2] = push_constants.light3_position.xyz;
  v_light_colours[0] = push_constants.light1_colour.xyz;
  v_light_colours[1] = push_constants.light2_colour.xyz;
  v_light_colours[2] = push_constants.light3_colour.xyz;
  v_light_intensity[0] = push_constants.light1_position.w;
  v_light_intensity[1] = push_constants.light2_position.w;
  v_light_intensity[2] = push_constants.light3_position.w;
  v_sun_direction = push_constants.sun_direction.xyz;
  v_sun_colour = push_constants.sun_colour;
  
  float near   = 1.0;
  float far    = -1.0;
  float right  = uniforms.c_center.w;
  float bottom = uniforms.c_up.w; 
  
  mat4 ortho_projection = create_ortho_projection(near, far, right, bottom);
  
  gl_Position = ortho_projection * vec4(position, 0.0, 1.0);
}
