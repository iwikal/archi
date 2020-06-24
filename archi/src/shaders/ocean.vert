uniform mat4 view_projection;
uniform vec2 camera_offset;

const int N = 256;
const float SCALE = 8.0;

layout (location = 0) out vec2 uv_out;
layout (location = 1) out vec3 position_out;

void main() {
  int line_count = N + 1;
  int x = gl_VertexID / line_count;
  int y = gl_VertexID % line_count;
  vec2 position_2d = vec2(x, y);
  uv_out = vec2(x, y) + camera_offset / SCALE;

  position_2d -= N / 2;
  position_2d *= SCALE;
  position_2d += camera_offset;

  vec3 position = vec3(position_2d.x, 0, position_2d.y);

  gl_Position = view_projection * vec4(position, 1.0);
  position_out = position;
}
