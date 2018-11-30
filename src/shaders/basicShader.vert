#version 430 core

uniform mat4 projection_view;

layout (location = 0) in vec3 position;

void main() {
  gl_Position = projection_view * vec4(position, 1.0);
}
