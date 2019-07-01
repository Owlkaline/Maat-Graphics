#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 uvs;
layout(location = 1) out vec3 v_camera_pos;
layout(location = 2) out vec3 v_light_positions[3];
layout(location = 5) out vec3 v_light_colours[3];
layout(location = 8) out float v_light_intensity[3];
layout(location = 11) out vec3 v_sun_direction;
layout(location = 12) out vec4 v_sun_colour;

//From vertex shader
layout(push_constant) uniform PushConstants {
  vec4 light1_position; // x, y, z, intensity1
  vec4 light1_colour; // r, g, b, camera_x
  vec4 light2_position; // x,y,z, intensity2
  vec4 light2_colour; // r, g, b, camera_y
  vec4 light3_position; // x,y,z, intensity3
  vec4 light3_colour; // r,g,b, camera_z
  // 8 floats spare
  vec4 sun_direction; // xyz, _
  vec4 sun_colour; // rgb intensity
} push_constants;

void main() {
  v_camera_pos = vec3(push_constants.light1_colour.w, push_constants.light2_colour.w, push_constants.light3_colour.w);
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
