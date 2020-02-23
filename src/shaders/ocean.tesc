#extension GL_ARB_tessellation_shader : enable

layout(vertices = 4) out;

layout (location = 0) in vec2 uv_in[];
layout (location = 0) out vec2 uv_out[];

uniform int tessellation_factor = 200;
uniform float tessellation_slope = 2.0;
uniform float tessellation_shift = 0.01;

float lod_factor(float dist) {
  float level = tessellation_factor / pow(dist, tessellation_slope);
  return max(0.0, level + tessellation_shift);
}

const int AB = 1;
const int BC = 0;
const int CD = 3;
const int DA = 2;

void main() {
  if (gl_InvocationID == 0) {
    vec3 a = gl_in[0].gl_Position.xyz;
    vec3 b = gl_in[1].gl_Position.xyz;
    vec3 c = gl_in[2].gl_Position.xyz;
    vec3 d = gl_in[3].gl_Position.xyz;

    float dist_a_b = length(a + b) / 2.0;
    float dist_b_c = length(b + c) / 2.0;
    float dist_c_d = length(c + d) / 2.0;
    float dist_d_a = length(d + a) / 2.0;

    gl_TessLevelOuter[AB] = mix(1, gl_MaxTessGenLevel, lod_factor(dist_a_b));
    gl_TessLevelOuter[BC] = mix(1, gl_MaxTessGenLevel, lod_factor(dist_b_c));
    gl_TessLevelOuter[CD] = mix(1, gl_MaxTessGenLevel, lod_factor(dist_c_d));
    gl_TessLevelOuter[DA] = mix(1, gl_MaxTessGenLevel, lod_factor(dist_d_a));

    gl_TessLevelInner[0] = (gl_TessLevelOuter[BC] + gl_TessLevelOuter[DA]) / 2.0;
    gl_TessLevelInner[1] = (gl_TessLevelOuter[AB] + gl_TessLevelOuter[CD]) / 2.0;	
  }

  gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;
  uv_out[gl_InvocationID] = uv_in[gl_InvocationID];
}
