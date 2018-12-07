#version 450

layout(location = 0) in vec2 positions;

layout(location = 0) out vec3 colour;

vec2 positionsstatic[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5)
);

vec3 colours[3] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);

void main() {
  colour = colours[gl_VertexIndex];
  gl_Position = vec4(positions, 0.0, 1.0);
}
