#version 430 core

layout (location = 0) in vec2 position;

out vec2 uv;

void main () {
  gl_Position = vec4(position, 0, 1);
  uv = position / 2 + 0.5;
}
