#version 430 core

in vec3 out_color;
in vec3 out_normal;
in vec3 out_position;

layout (location = 0) out vec3 color;
layout (location = 1) out vec3 normal;
layout (location = 2) out vec3 position;

void main() {
  color = out_color;
  normal = out_normal;
  position = out_position;
}
