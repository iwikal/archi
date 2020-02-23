uniform mat4 view;
uniform mat4 view_projection;
uniform vec2 offset;

const int N = 256;
const float SCALE = 8.0;

layout (location = 0) out vec2 uv_out;

vec3 position_at_coordinates(int x, int y) {
  vec2 position = vec2(x, y) + offset * N;
  vec3 result;
  result.x = position.x;
  result.y = 0.0;
  result.z = position.y;
  return result * SCALE;
}

vec3 average_normal(int x, int y) {
  float height_left  = position_at_coordinates(x - 1, y).y;
  float height_right = position_at_coordinates(x + 1, y).y;
  float height_up    = position_at_coordinates(x, y - 1).y;
  float height_down  = position_at_coordinates(x, y + 1).y;

  vec3 normal;
  normal.x = height_left - height_right;
  normal.y = height_down - height_up;
  normal.z = 2.0;

  return normalize(normal);
}

void main() {
  int line_count = N + 1;
  int x = gl_VertexID / line_count;
  int y = gl_VertexID % line_count;
  vec3 position = position_at_coordinates(x, y);

  gl_Position = view_projection * vec4(position, 1.0);
  uv_out = vec2(x, y);
}
