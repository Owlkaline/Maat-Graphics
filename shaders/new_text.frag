#version 450

layout (location = 0) in vec2 o_uv;
layout (location = 1) in vec4 o_colour;

layout (location = 0) out vec4 out_colour;

//uniform vec3 colour;
layout (set = 0, binding = 0) uniform sampler2D fontAtlas;

const float smoothing = 1.0/16.0;

void main(void){
  float distance = texture(fontAtlas, o_uv).a;
  float alpha = smoothstep(0.5 - smoothing, 0.5 + smoothing, distance);

  out_colour = vec4(o_colour.rgb*alpha, alpha);
}

