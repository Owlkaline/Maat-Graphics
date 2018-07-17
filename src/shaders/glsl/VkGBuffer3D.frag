#version 450

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec4 v_tangent;
layout(location = 3) in vec2 v_uv;
layout(location = 4) in vec4 v_colours;
layout(location = 5) in mat3 v_tbn;

layout(location = 0) out vec4 f_colour;
layout(location = 1) out vec4 f_normal;
layout(location = 2) out vec4 f_position;
layout(location = 3) out vec4 f_texcoord;
layout(location = 4) out vec4 f_mr;

layout(set = 1, binding = 0) uniform MaterialParams {
    vec4 base_colour_factor;
    int base_colour_texture_tex_coord;
    float metallic_factor;
    float roughness_factor;
    int metallic_roughness_texture_tex_coord;
    float normal_texture_scale;
    int normal_texture_tex_coord;
    int occlusion_texture_tex_coord;
    float occlusion_texture_strength;
    int emissive_texture_tex_coord;
    vec3 emissive_factor;
    float alpha_cutoff;
    int forced_alpha;
    int has_normals;
    int has_tangents;
} u_material_params;

layout(set = 1, binding = 1) uniform sampler2D u_base_colour;
layout(set = 1, binding = 2) uniform sampler2D u_metallic_roughness;
layout(set = 1, binding = 3) uniform sampler2D u_normal_texture;
layout(set = 1, binding = 4) uniform sampler2D u_occlusion_texture;
layout(set = 1, binding = 5) uniform sampler2D u_emissive_texture;

const float M_PI = 3.141592653589793;
const float c_MinRoughness = 0.04;
const vec3 c_LightColor = vec3(0.4,0.4,0.4);

// flIP y
const vec3 c_LightDirection = vec3(-0.4, 0.35, 0.2);
//const vec3 c_LightDirection = vec3(0.0, 1.0, 0.0);

vec3 get_normal() {
  mat3 tbn;
  if (u_material_params.has_tangents == 1) { // has_tangents
    tbn = v_tbn;
  } else {
    // doesnt have tangents
    vec3 pos_dx = dFdx(vec3(v_position.x, v_position.y*-1.0, v_position.z));
    vec3 pos_dy = dFdy(vec3(v_position.x, v_position.y*-1.0, v_position.z));
    vec3 tex_dx = dFdx(vec3(v_uv, 0.0));
    vec3 tex_dy = dFdy(vec3(v_uv, 0.0));
    vec3 t = (tex_dy.t * pos_dx - tex_dx.t * pos_dy) / (tex_dx.s * tex_dy.t - tex_dy.s * tex_dx.t);
    
    vec3 N;
    if (u_material_params.has_normals == 1) {
      N = normalize(v_normal);
    } else {
      N = cross(pos_dx, pos_dy);
    }
    
    t = normalize(t - N * dot(N, t));
    vec3 b = normalize(cross(N, t));
    tbn = mat3(t, b, N);
  }
  
  vec3 n = tbn[2].xyz;
  if (u_material_params.normal_texture_tex_coord == 0) {
    n = texture(u_normal_texture, v_uv).rgb;
    n = normalize(tbn * ((2.0 * n - 1.0) * vec3(u_material_params.normal_texture_scale, u_material_params.normal_texture_scale, 1.0)));
  }
  
  // gl front facing?
  // n *= (2.0 * float(gl_FrontFacing) - 1.0);
  
  return n;
}

void main() {
  vec3 n = get_normal();
  
  float metallic = u_material_params.metallic_factor;
  float roughness = u_material_params.roughness_factor;
  float ao = 1.0;
  
  if (u_material_params.metallic_roughness_texture_tex_coord == 0) {
    vec4 Sample = texture(u_metallic_roughness, v_uv);
    roughness = Sample.g * roughness;
    metallic = Sample.b * metallic;
  }
  
  roughness = clamp(roughness, c_MinRoughness, 1.0);
  metallic = clamp(metallic, 0.0, 1.0);
  
  vec4 base_colour = vec4(1.0, 1.0, 1.0, 1.0);
  if (u_material_params.base_colour_texture_tex_coord == 0) {
    base_colour = texture(u_base_colour, v_uv);
  }
  base_colour *= u_material_params.base_colour_factor;
  base_colour *= v_colours;
  
  float alpha_cutoff = u_material_params.alpha_cutoff;
  float alpha = base_colour.a;
  if (u_material_params.forced_alpha == 1) { // Opaque
    alpha = 1.0;
  }
  
  if (u_material_params.forced_alpha == 2) { // Mask
    if(alpha < alpha_cutoff) {
      discard;
    } else {
      alpha = 1.0;
    }
  }
  
  if (u_material_params.occlusion_texture_tex_coord != -1) {
    float ao = texture(u_occlusion_texture, v_uv).r;
    base_colour.rgb = mix(base_colour.rgb, base_colour.rgb * ao, u_material_params.occlusion_texture_strength);
  }
  
  vec3 emissive = vec3(0.0);
  if (u_material_params.emissive_texture_tex_coord != -1) {
    emissive = texture(u_emissive_texture, v_uv).rgb * u_material_params.emissive_factor;
  }
  
  f_colour = base_colour;
  f_normal = vec4(n, 1.0);
  f_position = vec4(v_position, 1.0);
  f_texcoord = vec4(v_uv.xy, 0.0, 1.0);
  f_mr = vec4(0.0, roughness, metallic, 1.0);
}
