layout(triangles) in;
layout(line_strip, max_vertices = 4) out;

layout (location = 0) in vec2 uv_in[3];
layout (location = 0) out vec2 uv_out;

layout (location = 1) in vec3 position_in[3];
layout (location = 1) out vec3 position_out;

void main() {
  for(int i = 0; i < 4; i++) {
    uv_out = uv_in[i % 3];
    position_out = position_in[i % 3];

    gl_Position = gl_in[i % 3].gl_Position;
    EmitVertex();
  }
  EndPrimitive();
}
