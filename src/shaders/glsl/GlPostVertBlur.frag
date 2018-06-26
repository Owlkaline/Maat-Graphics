#version 330 core

in vec2 uvs;
in vec2 dir;

out vec4 outColour;

uniform vec2 direction;
uniform sampler2D tex;

void main() {
  float blurScale = 1.0;
  float blurStrength = 1.0;
  
  float weight[5];
  
  weight[0] = 0.227027;
  weight[1] = 0.1945946;
  weight[2] = 0.1216216;
  weight[3] = 0.054054;
  weight[4] = 0.016216;

  vec2 tex_offset = 1.0 / textureSize(tex, 0) * blurScale; // gets size of single texel
  vec3 result = texture(tex, uvs).rgb * weight[0]; // current fragment's contribution
  for(int i = 1; i < 5; ++i) {
    // V
    result += texture(tex, uvs + vec2(0.0, tex_offset.y * i)).rgb * weight[i] * blurStrength;
    result += texture(tex, uvs - vec2(0.0, tex_offset.y * i)).rgb * weight[i] * blurStrength;
  }
  
  outColour = vec4(result, 1.0);
}
