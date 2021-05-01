out vec2 uv;

uniform mat4 view_projection;
uniform mat4 model;

void main() {
  vec4 position;
  position.x = float(gl_VertexID % 2);
  position.y = float(gl_VertexID / 2);
  position.xy -= vec2(0.5);
  position.z = 0.0;
  position.w = 1.0;

  uv = vec2(gl_VertexID % 2, gl_VertexID / 2);

  gl_Position = view_projection * model * position;
}
