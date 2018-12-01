#version 430 core

uniform mat4 model_view_projection;

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 color;

out vec3 out_color;

void main() {
  out_color = color;
  gl_Position = model_view_projection * vec4(position, 1.0);
}
