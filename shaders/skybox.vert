in vec3 position;
in vec2 uv;

out vec2 v_uv;

uniform mat4 view_projection;

void main() {
  gl_Position = view_projection * vec4(position, 1.0);
  v_uv = uv;
}
