#version 430 core

uniform mat4 projection_view;

layout (location = 0) in vec2 position;

void main() {
  gl_Position = projection_view * vec4(position, 0.0, 1.0);
}
