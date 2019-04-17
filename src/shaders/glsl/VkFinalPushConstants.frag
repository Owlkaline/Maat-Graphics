#version 450

layout(location = 0) in vec2 uvs;

layout(location = 0) out vec4 outColour;

layout(set = 0, binding = 0) uniform sampler2D texture_image;
layout(set = 1, binding = 0) uniform sampler2D model_image;

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
  outColour = texture(texture_image, uvs);
}

/*
void main() {
  vec4 tex = texture(texture_image, uvs);
  vec4 model = texture(model_image, uvs);
  
  vec4 tex_transparent = or(when_eq(vec4(tex.a), vec4(0.0)), when_lt(vec4(tex.a), vec4(0.0)));
  
  vec4 model_transparent = or(when_eq(vec4(model.a), vec4(0.0)), when_lt(vec4(model.a), vec4(0.0)));
  
  if (and(tex_transparent, model_transparent).a == 1.0) {
    discard;
  }
  
  vec4 draw_tex = not(tex_transparent);
  vec4 draw_model = not(model_transparent);
  
  outColour = and(not(draw_model), draw_tex)      * tex + 
              and(not(draw_tex), draw_model)      * model +
              and(draw_model, draw_tex) * vec4(mix(tex.rgb, model.rgb, 1.0-tex.a), 1.0);
}*/

/*
//Human readable version
//void main() {
  vec4 tex = texture(texture_image, uvs);
  vec4 model = texture(model_image, uvs);
  
  vec4 final_colour = tex;
  
  if (tex.w != 1.0) {
    if (tex.w == 0.0) {
      if (model.w == 0.0) {
        discard;
      }
      
      final_colour = model;
    }
  
    if (model.w != 0.0) {
      final_colour = vec4(mix(tex.rgb, model.rgb, 1.0-tex.a), 1.0);
    }
  }
  
  outColour = final_colour;
}*/
