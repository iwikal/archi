#extension GL_ARB_tessellation_shader : enable

layout(quads, equal_spacing, cw) in;

void main() {
  // world position
  vec4 p1 = mix(gl_in[1].gl_Position, gl_in[0].gl_Position, gl_TessCoord.x);
  vec4 p2 = mix(gl_in[2].gl_Position, gl_in[3].gl_Position, gl_TessCoord.x);

  gl_Position = mix(p1, p2, gl_TessCoord.y);
}
