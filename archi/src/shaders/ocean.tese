#extension GL_ARB_tessellation_shader : enable

layout(quads, fractional_odd_spacing, cw) in;

uniform sampler2D heightmap;

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
  // world position
  vec4 p1 = mix(gl_in[1].gl_Position, gl_in[0].gl_Position, gl_TessCoord.x);
  vec4 p2 = mix(gl_in[2].gl_Position, gl_in[3].gl_Position, gl_TessCoord.x);

  vec4 grid_position = interpolate(
      gl_in[0].gl_Position,
      gl_in[1].gl_Position,
      gl_in[2].gl_Position,
      gl_in[3].gl_Position);

  uv_out = interpolate(
      uv_in[0],
      uv_in[1],
      uv_in[2],
      uv_in[3]);
  uv_out /= 2.0;

  float height = 0.0;
  height += texture(heightmap, uv_out).x;

  gl_Position = grid_position;
  gl_Position.y += height;

  position_out = interpolate(
      position_in[0],
      position_in[1],
      position_in[2],
      position_in[3]);
  position_out.y += height;
}
