#version 330 core

in vec3 v_normal;
in vec2 v_uv;

out vec4 f_colour;

uniform sampler2D tex;

const vec3 LIGHT = vec3(1.0, 1.0, 1.0);

void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT));
    vec3 dark_colour = vec3(0.6, 0.6, 0.6);
    vec3 regular_colour = vec3(1.0, 1.0, 1.0);

    f_colour = vec4(mix(dark_colour, regular_colour, brightness), 1.0) * texture(tex, v_uv);
}
