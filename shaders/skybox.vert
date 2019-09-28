in vec3 position;
in vec2 uv;

out vec3 sample_direction;

uniform mat4 view_projection;

void main() {
  sample_direction = position;
  gl_Position = view_projection * vec4(position, 1.0);
  gl_Position.z = gl_Position.w;
}
