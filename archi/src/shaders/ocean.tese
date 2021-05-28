#extension GL_ARB_tessellation_shader : enable

layout(quads, fractional_odd_spacing, cw) in;

uniform sampler2D displacement_map;

uniform mat4 view_projection;

uniform float time = 0;

layout (location = 0) in vec2 uv_in[gl_MaxPatchVertices];
layout (location = 0) out vec2 uv_out;

layout (location = 1) in vec3 position_in[gl_MaxPatchVertices];
layout (location = 1) out vec3 position_out;

vec2 interpolate(vec2 a, vec2 b, vec2 c, vec2 d) {
  return mix(
      mix(b, a, gl_TessCoord.x),
      mix(c, d, gl_TessCoord.x),
      gl_TessCoord.y);
}

vec3 interpolate(vec3 a, vec3 b, vec3 c, vec3 d) {
  return mix(
      mix(b, a, gl_TessCoord.x),
      mix(c, d, gl_TessCoord.x),
      gl_TessCoord.y);
}

vec4 interpolate(vec4 a, vec4 b, vec4 c, vec4 d) {
  return mix(
      mix(b, a, gl_TessCoord.x),
      mix(c, d, gl_TessCoord.x),
      gl_TessCoord.y);
}

void main() {
  uv_out = interpolate(
      uv_in[0],
      uv_in[1],
      uv_in[2],
      uv_in[3]);

  vec3 displacement = vec3(0);
  displacement += texture(displacement_map, uv_out).xyz;
  displacement.y = 0;

  position_out = interpolate(
      position_in[0],
      position_in[1],
      position_in[2],
      position_in[3]);

  position_out.xyz += displacement;

  gl_Position.xyz = position_out + displacement;
  gl_Position.w = 1;

  gl_Position = view_projection * gl_Position;
}
