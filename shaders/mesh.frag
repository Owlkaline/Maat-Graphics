#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (set = 1, binding = 0) uniform sampler2D samplerColour;

layout (location = 0) in vec3 o_normal;
layout (location = 1) in vec3 o_colour;
layout (location = 2) in vec2 o_uv;
layout (location = 3) in vec3 o_view_vec;
layout (location = 4) in vec3 o_light_vec;

layout (location = 0) out vec4 uFragColor;

void main() {
  vec4 colour = texture(samplerColour, o_uv) * vec4(o_colour, 1.0);
  
  vec3 N = normalize(o_normal);
  vec3 L = normalize(o_light_vec);
  vec3 V = normalize(o_view_vec);
  vec3 R = reflect(-L, N);
  vec3 diffuse = max(dot(N, L), 0.15) * o_colour;
  vec3 specular = pow(max(dot(R, V), 0.0), 16.0) * vec3(0.75);
  
  uFragColor = vec4(diffuse * colour.rgb + specular, 1.0);
}
