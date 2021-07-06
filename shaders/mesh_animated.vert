#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 uv;
layout (location = 3) in vec3 colour;
layout (location = 4) in vec4 joint_indices;
layout (location = 5) in vec4 joint_weights;

layout (location = 0) out vec3 o_normal;
layout (location = 1) out vec3 o_colour;
layout (location = 2) out vec2 o_uv;
layout (location = 3) out vec3 o_view_vec;
layout (location = 4) out vec3 o_light_vec;

layout (set = 0, binding = 0) uniform UBO {
  mat4 projection;
  mat4 view;
  vec4 light_pos;
} ubo;

layout(push_constant) uniform PushConstants {
  mat4 model;
  vec4 offset; // x y z
  vec4 scale; // sx sy sz
  vec4 attrib2;
  vec4 attrib3;
} push_constants;

layout(set = 1, binding = 0) readonly buffer JointMatrices {
  mat4 joint_matrices[];
};

mat4 scale_matrix(vec3 scale) {
  mat4 s_m = mat4(1.0);
  
  s_m[0][0] = scale[0];
  s_m[1][1] = scale[1];
  s_m[2][2] = scale[2];
  
  return s_m;
}

void main() {
  o_normal = normal;
  o_colour = colour;
  o_uv = uv;
  
  vec3 obj_pos = pos + push_constants.offset.xyz;
  mat4 scale_matrix = scale_matrix(push_constants.scale.xyz);
  
  mat4 skin_mat = joint_weights.x * joint_matrices[int(joint_indices.x)] +
                  joint_weights.y * joint_matrices[int(joint_indices.y)] +
                  joint_weights.z * joint_matrices[int(joint_indices.z)] +
                  joint_weights.w * joint_matrices[int(joint_indices.w)];
  
  gl_Position = ubo.projection * ubo.view * scale_matrix * push_constants.model  * skin_mat * vec4(obj_pos.xyz, 1.0);
  
  vec4 pos = ubo.view * vec4(obj_pos, 1.0);
  o_normal = mat3(ubo.view) * normal;
  vec3 l_pos = mat3(ubo.view) * ubo.light_pos.xyz;
  o_light_vec = l_pos - pos.xyz;
  o_view_vec = -pos.xyz;
}
