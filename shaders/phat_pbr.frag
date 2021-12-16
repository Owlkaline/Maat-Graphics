#version 450

// From the Filament design doc
// https://google.github.io/filament/Filament.html#table_symbols
// Symbol Definition
// v    View unit vector
// l    Incident light unit vector
// n    Surface normal unit vector
// h    Half unit vector between l and v
// f    BRDF
// f_d    Diffuse component of a BRDF
// f_r    Specular component of a BRDF
// α    Roughness, remapped from using input perceptualRoughness
// σ    Diffuse reflectance
// Ω    Spherical domain
// f0    Reflectance at normal incidence
// f90    Reflectance at grazing angle
// χ+(a)    Heaviside function (1 if a>0 and 0 otherwise)
// nior    Index of refraction (IOR) of an interface
// ⟨n⋅l⟩    Dot product clamped to [0..1]
// ⟨a⟩    Saturated value (clamped to [0..1])

// The Bidirectional Reflectance Distribution Function (BRDF) describes the surface response of a standard material
// and consists of two components, the diffuse component (f_d) and the specular component (f_r):
// f(v,l) = f_d(v,l) + f_r(v,l)
//
// The form of the microfacet model is the same for diffuse and specular
// f_r(v,l) = f_d(v,l) = 1 / { |n⋅v||n⋅l| } ∫_Ω D(m,α) G(v,l,m) f_m(v,l,m) (v⋅m) (l⋅m) dm
//
// In which:
// D, also called the Normal Distribution Function (NDF) models the distribution of the microfacets
// G models the visibility (or occlusion or shadow-masking) of the microfacets
// f_m is the microfacet BRDF and differs between specular and diffuse components
//
// The above integration needs to be approximated.

// reflects the constants defined bevy_pbr/src/render_graph/mod.rs
const int MAX_POINT_LIGHTS = 10;
const int MAX_DIRECTIONAL_LIGHTS = 1;

struct PointLight {
    vec4 pos;
    vec4 colour;
    vec4 lightParams;
};
 
struct DirectionalLight {
    vec4 direction;
    vec4 colour;
};

//layout(location = 0) in vec3 o_world_position;
//layout(location = 1) in vec3 o_normal;
//layout(location = 2) in vec2 o_uv;
//
//#ifdef NORMAL_MAP
//layout(location = 3) in vec4 v_WorldTangent;
//#endif

layout (location = 0) in vec3 o_colour;
layout (location = 1) in vec3 o_normal;
layout (location = 2) in vec2 o_uv;
layout (location = 3) in vec4 o_world_position;
layout (location = 4) in vec4 camera_pos;
layout (location = 5) in mat4 o_view;
//layout (location = 3) in vec3 o_view_vec;
//layout (location = 4) in vec3 o_light_vec;

layout(location = 0) out vec4 o_Target;

layout(set = 3, binding = 0) uniform sampler2D base_colour_texture;
layout(set = 3, binding = 1) uniform sampler2D metallic_roughness_texture;
layout(set = 3, binding = 2) uniform sampler2D normal_map;
layout(set = 3, binding = 3) uniform sampler2D occlusion_texture;
layout(set = 3, binding = 4) uniform sampler2D emissive_texture;

layout (set = 3, binding = 5) uniform UBO {
  float roughness,
  float metallic,
  vec4 emissive,
} pbr_ubo;

layout(set = 2, binding = 0) uniform Lights {
  vec4 Ambientcolour;
  uvec4 NumLights; // x = point lights, y = directional lights
  PointLight PointLights[MAX_POINT_LIGHTS];
  DirectionalLight DirectionalLights[MAX_DIRECTIONAL_LIGHTS];
} lights;


#    define saturate(x) clamp(x, 0.0, 1.0)
const float PI = 3.141592653589793;

float pow5(float x) {
    float x2 = x * x;
    return x2 * x2 * x;
}

// distanceAttenuation is simply the square falloff of light intensity
// combined with a smooth attenuation at the edge of the light radius
//
// light radius is a non-physical construct for efficiency purposes,
// because otherwise every light affects every fragment in the scene
float getDistanceAttenuation(float distanceSquare, float inverseRangeSquared) {
    float factor = distanceSquare * inverseRangeSquared;
    float smoothFactor = saturate(1.0 - factor * factor);
    float attenuation = smoothFactor * smoothFactor;
    return attenuation * 1.0 / max(distanceSquare, 1e-4);
}

// Normal distribution function (specular D)
// Based on https://google.github.io/filament/Filament.html#citation-walter07

// D_GGX(h,α) = α^2 / { π ((n⋅h)^2 (α2−1) + 1)^2 }

// Simple implementation, has precision problems when using fp16 instead of fp32
// see https://google.github.io/filament/Filament.html#listing_speculardfp16
float D_GGX(float roughness, float NoH, const vec3 h) {
    float oneMinusNoHSquared = 1.0 - NoH * NoH;
    float a = NoH * roughness;
    float k = roughness / (oneMinusNoHSquared + a * a);
    float d = k * k * (1.0 / PI);
    return d;
}

// Visibility function (Specular G)
// V(v,l,a) = G(v,l,α) / { 4 (n⋅v) (n⋅l) }
// such that f_r becomes
// f_r(v,l) = D(h,α) V(v,l,α) F(v,h,f0)
// where
// V(v,l,α) = 0.5 / { n⋅l sqrt((n⋅v)^2 (1−α2) + α2) + n⋅v sqrt((n⋅l)^2 (1−α2) + α2) }
// Note the two sqrt's, that may be slow on mobile, see https://google.github.io/filament/Filament.html#listing_approximatedspecularv
float V_SmithGGXCorrelated(float roughness, float NoV, float NoL) {
    float a2 = roughness * roughness;
    float lambdaV = NoL * sqrt((NoV - a2 * NoV) * NoV + a2);
    float lambdaL = NoV * sqrt((NoL - a2 * NoL) * NoL + a2);
    float v = 0.5 / (lambdaV + lambdaL);
    return v;
}

// Fresnel function
// see https://google.github.io/filament/Filament.html#citation-schlick94
// F_Schlick(v,h,f_0,f_90) = f_0 + (f_90 − f_0) (1 − v⋅h)^5
vec3 F_Schlick(const vec3 f0, float f90, float VoH) {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow5(1.0 - VoH);
}

float F_Schlick(float f0, float f90, float VoH) {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow5(1.0 - VoH);
}

vec3 fresnel(vec3 f0, float LoH) {
    // f_90 suitable for ambient occlusion
    // see https://google.github.io/filament/Filament.html#lighting/occlusion
    float f90 = saturate(dot(f0, vec3(50.0 * 0.33)));
    return F_Schlick(f0, f90, LoH);
}

// Specular BRDF
// https://google.github.io/filament/Filament.html#materialsystem/specularbrdf

// Cook-Torrance approximation of the microfacet model integration using Fresnel law F to model f_m
// f_r(v,l) = { D(h,α) G(v,l,α) F(v,h,f0) } / { 4 (n⋅v) (n⋅l) }
vec3 specular(vec3 f0, float roughness, const vec3 h, float NoV, float NoL,
              float NoH, float LoH, float specularIntensity) {
    float D = D_GGX(roughness, NoH, h);
    float V = V_SmithGGXCorrelated(roughness, NoV, NoL);
    vec3 F = fresnel(f0, LoH);

    return (specularIntensity * D * V) * F;
}

// Diffuse BRDF
// https://google.github.io/filament/Filament.html#materialsystem/diffusebrdf
// fd(v,l) = σ/π * 1 / { |n⋅v||n⋅l| } ∫Ω D(m,α) G(v,l,m) (v⋅m) (l⋅m) dm

// simplest approximation
// float Fd_Lambert() {
//     return 1.0 / PI;
// }
//
// vec3 Fd = diffusecolour * Fd_Lambert();

// Disney approximation
// See https://google.github.io/filament/Filament.html#citation-burley12
// minimal quality difference
float Fd_Burley(float roughness, float NoV, float NoL, float LoH) {
    float f90 = 0.5 + 2.0 * roughness * LoH * LoH;
    float lightScatter = F_Schlick(1.0, f90, NoL);
    float viewScatter = F_Schlick(1.0, f90, NoV);
    return lightScatter * viewScatter * (1.0 / PI);
}

// From https://www.unrealengine.com/en-US/blog/physically-based-shading-on-mobile
vec3 EnvBRDFApprox(vec3 f0, float perceptual_roughness, float NoV) {
    const vec4 c0 = { -1, -0.0275, -0.572, 0.022 };
    const vec4 c1 = { 1, 0.0425, 1.04, -0.04 };
    vec4 r = perceptual_roughness * c0 + c1;
    float a004 = min(r.x * r.x, exp2(-9.28 * NoV)) * r.x + r.y;
    vec2 AB = vec2(-1.04, 1.04) * a004 + r.zw;
    return f0 * AB.x + AB.y;
}

float perceptualRoughnessToRoughness(float perceptualRoughness) {
    // clamp perceptual roughness to prevent precision problems
    // According to Filament design 0.089 is recommended for mobile
    // Filament uses 0.045 for non-mobile
    float clampedPerceptualRoughness = clamp(perceptualRoughness, 0.089, 1.0);
    return clampedPerceptualRoughness * clampedPerceptualRoughness;
}

// from https://64.github.io/tonemapping/
// reinhard on RGB oversaturates colours
vec3 reinhard(vec3 colour) {
    return colour / (1.0 + colour);
}

vec3 reinhard_extended(vec3 colour, float max_white) {
    vec3 numerator = colour * (1.0f + (colour / vec3(max_white * max_white)));
    return numerator / (1.0 + colour);
}

// luminance coefficients from Rec. 709.
// https://en.wikipedia.org/wiki/Rec._709
float luminance(vec3 v) {
    return dot(v, vec3(0.2126, 0.7152, 0.0722));
}

vec3 change_luminance(vec3 c_in, float l_out) {
    float l_in = luminance(c_in);
    return c_in * (l_out / l_in);
}

vec3 reinhard_luminance(vec3 colour) {
    float l_old = luminance(colour);
    float l_new = l_old / (1.0f + l_old);
    return change_luminance(colour, l_new);
}

vec3 reinhard_extended_luminance(vec3 colour, float max_white_l) {
    float l_old = luminance(colour);
    float numerator = l_old * (1.0f + (l_old / (max_white_l * max_white_l)));
    float l_new = numerator / (1.0f + l_old);
    return change_luminance(colour, l_new);
}

vec3 point_light(PointLight light, float roughness, float NdotV, vec3 N, vec3 V, vec3 R, vec3 F0, vec3 diffusecolour) {
    vec3 light_to_frag = light.pos.xyz - o_world_position.xyz;
    float distance_square = dot(light_to_frag, light_to_frag);
    float rangeAttenuation =
        getDistanceAttenuation(distance_square, light.lightParams.r);

    // Specular.
    // Representative Point Area Lights.
    // see http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf p14-16
    float a = roughness;
    float radius = light.lightParams.g;
    vec3 centerToRay = dot(light_to_frag, R) * R - light_to_frag;
    vec3 closestPoint = light_to_frag + centerToRay * saturate(radius * inversesqrt(dot(centerToRay, centerToRay)));
    float LspecLengthInverse = inversesqrt(dot(closestPoint, closestPoint));
    float normalizationFactor = a / saturate(a + (radius * 0.5 * LspecLengthInverse));
    float specularIntensity = normalizationFactor * normalizationFactor;

    vec3 L = closestPoint * LspecLengthInverse; // normalize() equivalent?
    vec3 H = normalize(L + V);
    float NoL = saturate(dot(N, L));
    float NoH = saturate(dot(N, H));
    float LoH = saturate(dot(L, H));

    vec3 specular = specular(F0, roughness, H, NdotV, NoL, NoH, LoH, specularIntensity);

    // Diffuse.
    // Comes after specular since its NoL is used in the lighting equation.
    L = normalize(light_to_frag);
    H = normalize(L + V);
    NoL = saturate(dot(N, L));
    NoH = saturate(dot(N, H));
    LoH = saturate(dot(L, H));

    vec3 diffuse = diffusecolour * Fd_Burley(roughness, NdotV, NoL, LoH);

    // Lout = f(v,l) Φ / { 4 π d^2 }⟨n⋅l⟩
    // where
    // f(v,l) = (f_d(v,l) + f_r(v,l)) * light_colour
    // Φ is light intensity

    // our rangeAttentuation = 1 / d^2 multiplied with an attenuation factor for smoothing at the edge of the non-physical maximum light radius
    // It's not 100% clear where the 1/4π goes in the derivation, but we follow the filament shader and leave it out

    // See https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminanceEquation
    // TODO compensate for energy loss https://google.github.io/filament/Filament.html#materialsystem/improvingthebrdfs/energylossinspecularreflectance
    // light.colour.rgb is premultiplied with light.intensity on the CPU
    return ((diffuse + specular) * light.colour.rgb) * (rangeAttenuation * NoL);
}

vec3 dir_light(DirectionalLight light, float roughness, float NdotV, vec3 normal, vec3 view, vec3 R, vec3 F0, vec3 diffusecolour) {
    vec3 incident_light = light.direction.xyz;

    vec3 half_vector = normalize(incident_light + view);
    float NoL = saturate(dot(normal, incident_light));
    float NoH = saturate(dot(normal, half_vector));
    float LoH = saturate(dot(incident_light, half_vector));

    vec3 diffuse = diffusecolour * Fd_Burley(roughness, NdotV, NoL, LoH);
    float specularIntensity = 1.0;
    vec3 specular = specular(F0, roughness, half_vector, NdotV, NoL, NoH, LoH, specularIntensity);

    return (specular + diffuse) * light.colour.rgb * NoL;
}

void main() {
    vec4 output_colour = base_colour;
    output_colour *= texture(base_colour_texture, o_uv);

    // calculate non-linear roughness from linear perceptualRoughness
    vec4 metallic_roughness = texture(metallic_roughness_texture, o_uv);
    // Sampling from GLTF standard channels for now
    float metallic = metallic * metallic_roughness.b;
    float perceptual_roughness = perceptual_roughness * metallic_roughness.g;

    float roughness = perceptualRoughnessToRoughness(perceptual_roughness);

    vec3 N = normalize(o_normal);

    vec3 T = normalize(v_WorldTangent.xyz);
    vec3 B = cross(N, T) * v_WorldTangent.w;

//#    ifdef DOUBLE_SIDED
//    N = gl_FrontFacing ? N : -N;
//#        ifdef NORMAL_MAP
//    T = gl_FrontFacing ? T : -T;
//    B = gl_FrontFacing ? B : -B;
//#        endif
//
//#    ifdef NORMAL_MAP
//    mat3 TBN = mat3(T, B, N);
//    N = TBN * normalize(texture(sampler2D(normal_map, normal_map_sampler), o_uv).rgb * 2.0 - 1.0);
//#    endif

    float occlusion = texture(occlusion_texture, o_uv).r;
    float occlusion = 1.0;

    vec4 emissive = emissive;
    // TODO use .a for exposure compensation in HDR
    emissive.rgb *= texture(emissive_texture, o_uv).rgb;

    vec3 V;
    if (ViewProj[3][3] != 1.0) { // If the projection is not orthographic
        V = normalize(CameraPos.xyz - o_world_position.xyz); // Only valid for a perpective projection
    } else {
        V = normalize(vec3(-ViewProj[0][2],-ViewProj[1][2],-ViewProj[2][2])); // Ortho view vec
    }
    // Neubelt and Pettineo 2013, "Crafting a Next-gen Material Pipeline for The Order: 1886"
    float NdotV = max(dot(N, V), 1e-4);

    // Remapping [0,1] reflectance to F0
    // See https://google.github.io/filament/Filament.html#materialsystem/parameterization/remapping
    vec3 F0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + output_colour.rgb * metallic;

    // Diffuse strength inversely related to metallicity
    vec3 diffusecolour = output_colour.rgb * (1.0 - metallic);

    vec3 R = reflect(-V, N);

    // accumulate colour
    vec3 light_accum = vec3(0.0);
    for (int i = 0; i < int(NumLights.x) && i < MAX_POINT_LIGHTS; ++i) {
        light_accum += point_light(PointLights[i], roughness, NdotV, N, V, R, F0, diffusecolour);
    }
    for (int i = 0; i < int(NumLights.y) && i < MAX_DIRECTIONAL_LIGHTS; ++i) {
        light_accum += dir_light(DirectionalLights[i], roughness, NdotV, N, V, R, F0, diffusecolour);
    }

    vec3 diffuse_ambient = EnvBRDFApprox(diffusecolour, 1.0, NdotV);
    vec3 specular_ambient = EnvBRDFApprox(F0, perceptual_roughness, NdotV);

    output_colour.rgb = light_accum;
    output_colour.rgb += (diffuse_ambient + specular_ambient) * Ambientcolour.xyz * occlusion;
    output_colour.rgb += emissive.rgb * output_colour.a;

    // tone_mapping
    output_colour.rgb = reinhard_luminance(output_colour.rgb);
    // Gamma correction.
    // Not needed with sRGB buffer
    // output_colour.rgb = pow(output_colour.rgb, vec3(1.0 / 2.2));

    o_Target = output_colour;
}
