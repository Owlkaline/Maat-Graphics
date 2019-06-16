#version 450

layout(location = 0) in vec2 uvs;
layout(location = 1) in vec4 v_camera_pos;
layout(location = 2) in vec4 v_camera_center;
layout(location = 3) in vec4 v_camera_up;
layout(location = 4) in vec3 v_light_positions[3];
layout(location = 7) in vec3 v_light_colours[3];
layout(location = 10) in float v_light_intensity[3];
layout(location = 13) in vec3 v_sun_direction;
layout(location = 14) in vec4 v_sun_colour;

layout(location = 0) out vec4 outColour;

layout (input_attachment_index = 0, binding = 0) uniform subpassInput colour_texture;
layout (input_attachment_index = 0, binding = 1) uniform subpassInput depth_texture;
layout (input_attachment_index = 1, binding = 1) uniform subpassInput mro_texture;
layout (input_attachment_index = 2, binding = 2) uniform subpassInput emissive_texture;
layout (input_attachment_index = 3, binding = 3) uniform subpassInput normal_texture;

const float M_PI = 3.141592653589793;

float cot(float value) {
  return 1.0 / tan(value);
}

float to_radians(float degree) {
  return degree * (M_PI/180.0);
}

float D_GGX(float dotNH, float roughness) {
  float alpha = roughness * roughness;
  float alpha2 = alpha * alpha;
  float denom = dotNH * dotNH * (alpha2 - 1.0) + 1.0;
  return (alpha2)/(M_PI * denom*denom); 
}

float G_SchlicksmithGGX(float dotNL, float dotNV, float roughness) {
  float r = (roughness + 1.0);
  float k = (r*r) / 8.0;
  float GL = dotNL / (dotNL * (1.0 - k) + k);
  float GV = dotNV / (dotNV * (1.0 - k) + k);
  
  return GL * GV;
}

vec3 F_Schlick(float cosTheta, float metallic) {
  vec3 F0 = mix(vec3(0.04), vec3(0.2, 0.2, 0.2), metallic);
  vec3 F = F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0); 
  
  return F; 
}

vec3 BRDF(vec3 L, vec3 V, vec3 N, float metallic, float roughness, vec3 light_position, vec3 light_colour, float intensity, vec3 world_pos) {
  vec3 H = normalize(V+L);
  float dotNV = clamp(dot(N, V), 0.0, 1.0);
  float dotNL = clamp(dot(N, L), 0.0, 1.0);
  float dotLH = clamp(dot(L, H), 0.0, 1.0);
  float dotNH = clamp(dot(N, H), 0.0, 1.0);
  
  vec3 colour = vec3(0.0);
  
  float distance = length(light_position-world_pos);
  
  float attenuation = 1.0;
  attenuation *= 1.0 / max(distance * distance, 0.01*0.01);
  
  vec3 radiance = light_colour * intensity * attenuation;
  
  if (dotNL > 0.0) {
    float rr = max(0.05, roughness);
    
    float D = D_GGX(dotNH, roughness);
    
    float G = G_SchlicksmithGGX(dotNL, dotNV, roughness);
    
    vec3 F = F_Schlick(dotNV, metallic);
    
    vec3 spec = D *F * G / (4.0 * dotNL * dotNV);
    
    colour += spec * radiance * dotNL; // * light_colour;
  }
  
  return colour;
}

mat4 create_perspective_matrix(float fov, float aspect, float near, float far) {
  float f = cot(to_radians(fov) / 2.0);
  
  mat4 perspective = mat4(
                      vec4(f / aspect, 0.0,   0.0,                               0.0),
                      vec4(0.0,        f,     0.0,                               0.0),
                      vec4(0.0,        0.0,   (far + near) / (near - far),      -1.0),
                      vec4(0.0,        0.0,   (2.0 * far * near) / (near - far), 0.0)
                    );
                
  return perspective;
}

// center is a point not a direction
mat4 create_view_matrix(vec3 eye, vec3 center, vec3 up) {
  vec3 dir = center - eye;
  
  vec3 f = normalize(dir);
  vec3 s = normalize(cross(f, up));
  vec3 u = cross(s,f);
  
  mat4 look_at_matrix = mat4(vec4(s.x,           u.x,        -f.x,         0.0), 
                             vec4(s.y,           u.y,        -f.y,         0.0), 
                             vec4(s.z,           u.z,        -f.z,         0.0), 
                             vec4(-dot(eye, s), -dot(eye, u), dot(eye, f), 1.0));
  
  return look_at_matrix;
}

vec3 depth_to_world_position(float depth_value, mat4 invProjection, mat4 invView) {
  
  vec4 clip_space = vec4(uvs * 2.0 - 1.0, depth_value, 1.0);
  vec4 view_space = invProjection * clip_space;
  
  vec4 world_pos = invView * view_space;
  
  return world_pos.xyz;
}

void main() {
  float aspect = v_camera_up.w/v_camera_center.w;
  mat4 projection = create_perspective_matrix(v_camera_pos.w, aspect, 0.1, 256.0);
  mat4 view = create_view_matrix(v_camera_pos.xyz, v_camera_center.xyz, v_camera_up.xyz);
  mat4 invProjection = inverse(projection);
  mat4 invView = inverse(view);
  
  float depth = subpassLoad(depth_texture).a;
  
  vec3 world_pos = depth_to_world_position(depth, invProjection, invView);
  
  vec3 N = subpassLoad(normal_texture).rgb;
  vec3 V = normalize(v_camera_pos.xyz - world_pos);
  
  vec3 Lo = vec3(0.0);
  
  vec3 L = normalize(v_light_positions[0] - world_pos);
  
  vec4 base_colour = subpassLoad(colour_texture);
  vec4 mro_colour = subpassLoad(mro_texture);
  
  Lo += BRDF(L, V, N, mro_colour.x, mro_colour.y, v_light_positions[0], v_light_colours[0].xyz, v_light_intensity[0], world_pos);
  
  base_colour *= 0.02;
  base_colour.rgb += Lo;
  
  base_colour.rgb = pow(base_colour.rgb, vec3(0.4545));
  
  outColour = base_colour;
}
