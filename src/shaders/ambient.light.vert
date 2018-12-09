#version 430 core

layout (location = 0) in vec2 a_position;
layout (location = 1) uniform vec3 u_color;

out vec3 color;
out vec2 uv;

void main () {
  color = u_color;
  gl_Position = vec4(a_position, 0, 1);
  uv = a_position / 2 + 0.5;
}
