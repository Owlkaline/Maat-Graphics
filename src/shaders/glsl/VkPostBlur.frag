#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec2 dir;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
  /*float[6] kernal;
  kernal[0] = 0.382925;
  kernal[1] = 0.24173;
  kernal[2] = 0.060598;
  kernal[3] = 0.005977;
  kernal[4] = 0.000229;
  kernal[5] = 0.000003;
  
  vec2 two_px = dir * vec2(2) / vec2(textureSize(tex, 0));
  vec2 half_px = two_px / 4.0;
  
  vec4 colour_sum = kernal[0] * texture(tex, uvs);
  
  for (int i = 1; i < 5; ++i) {
    float k = kernal[i];
    vec2 offset = two_px * float(i) - half_px;
    colour_sum += k * texture(tex, -offset + uvs);
    colour_sum += k * texture(tex, -offset + uvs);
  }*/
  
  float[5] kernal;
  kernal[0] = 0.227027;
  kernal[1] = 0.1945946;
  kernal[2] = 0.1216216;
  kernal[3] = 0.054054;
  kernal[4] = 0.016216;
  
  vec2 tex_offset = 1.0 / textureSize(tex, 0);
  vec3 result = texture(tex, uvs).rgb * kernal[0];
  if (dir.x == 1.0) {
    for (int i = 1; i < 5; ++i) {
      result += texture(tex, uvs + vec2(tex_offset.x * i, 0.0)).rgb * kernal[i];
      result += texture(tex, uvs - vec2(tex_offset.x * i, 0.0)).rgb * kernal[i];
    }
  } else {
    for (int i = 1; i < 5; ++i) {
      result += texture(tex, uvs + vec2(tex_offset.y * i, 0.0)).rgb * kernal[i];
      result += texture(tex, uvs - vec2(tex_offset.y * i, 0.0)).rgb * kernal[i];
    }
  }
  
  outColour = vec4(result, 1.0);
}
