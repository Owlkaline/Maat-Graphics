#version 330 core

in vec2 uvs;
in vec4 colour;
in float textured;
in float blackwhite;

out vec4 outColour;

uniform sampler2D tex;

vec4 when_eq(vec4 x, vec4 y) {
  return 1.0 - abs(sign(x - y));
}

vec4 when_neq(vec4 x, vec4 y) {
  return abs(sign(x - y));
}

vec4 when_gt(vec4 x, vec4 y) {
  return max(sign(x - y), 0.0);
}

vec4 when_lt(vec4 x, vec4 y) {
  return max(sign(y - x), 0.0);
}

vec4 when_ge(vec4 x, vec4 y) {
  return 1.0 - when_lt(x, y);
}

vec4 when_le(vec4 x, vec4 y) {
  return 1.0 - when_gt(x, y);
}


vec4 and(vec4 a, vec4 b) {
  return a * b;
}

vec4 or(vec4 a, vec4 b) {
  return min(a + b, 1.0);
}
/*
vec4 xor(vec4 a, vec4 b) {
  return (a + b) % 2.0;
}*/

vec4 not(vec4 a) {
  return 1.0 - a;
}

void main() {
  vec4 option = vec4(textured, colour.w, blackwhite, 0.0);
  
  vec4 option_ge = when_ge(option, vec4(1.0));
  
  // Plain colour automatically
  vec4 colourTexture = colour;
  
  // If is texture, adds the texture and negates the colours, otherwise adds 0
  colourTexture = (texture(tex, uvs)) * option_ge.x;
  
  // Change alpha value of the texture
  colourTexture.a = (colourTexture.w * colour.w) * when_neq(option, vec4(-1.0)).y;
  
  // If black and white mode is enabled, vec4(0.0) when isnt
  //float brightness = dot(colourTexture.rgb*vec3(2.0), vec3(0.2126, 0.7152, 0.0722));
 // vec4 blackwhiteTexture = vec4(vec3(brightness), colourTexture.a) * option_ge.z;
  
  outColour = colourTexture; //+ blackwhiteTexture;
}
