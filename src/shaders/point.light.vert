#version 430 core

layout (location = 0) in vec3 a_position;
layout (location = 1) uniform mat4 mvp;

void main () {
  gl_Position = mvp * vec4(a_position, 1.0);
}
