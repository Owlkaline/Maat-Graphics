#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(set = 0, binding = 0) uniform UniformBuffer {
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

void main() {
  v_camera_pos = uniforms.c_position;//vec4(3.93659, 1.21074, -15.50323, 60.0);//
  v_camera_center = uniforms.c_center;//vec4(2.7219, 0.94058, -14.56466, 1280.0);//;
  v_camera_up = uniforms.c_up;//vec4(0.06024, 0.96281, -0.26336, 1080.0);//
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
  
  uvs = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
  gl_Position = vec4(uvs * 2.0f - 1.0f, 0.0f, 1.0f);
}
