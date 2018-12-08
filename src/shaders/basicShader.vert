#version 430 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec3 color;

layout (location = 3) uniform mat4 model_view_projection;
layout (location = 4) uniform mat4 model;

out vec3 out_normal;
out vec3 out_color;

void main() {
  out_color = mat3(model) * normal;
  gl_Position = model_view_projection * vec4(position, 1.0);
}
