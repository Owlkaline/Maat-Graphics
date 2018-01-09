#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

out vec3 v_normal;
out vec2 v_uv;

uniform mat4 world;
uniform mat4 view;
uniform mat4 proj;

void main() {
    mat4 worldview = view * world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    v_uv = uv;
    gl_Position = proj * worldview * vec4(position, 1.0);
}
