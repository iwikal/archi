#version 430 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 tangent;
layout (location = 3) in vec2 uv;

layout (location = 3) uniform mat4 model_view_projection;
layout (location = 4) uniform mat4 model;

out vec2 out_uv;
out mat3 out_tbn;
out vec3 out_position;

void main() {
  gl_Position = model_view_projection * vec4(position, 1.0);
  out_position = (model * vec4(position, 1)).xyz;

  vec3 T = mat3(model) * tangent.xyz;
  vec3 N = mat3(model) * normal;
  T = normalize(T - dot(T, N) * N);
  vec3 B = cross(N, T);

  out_tbn = mat3(T, B, N);
  out_uv = uv;
}
