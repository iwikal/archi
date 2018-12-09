#version 430 core

in vec3 color;
in vec2 uv;

uniform sampler2D color_buffer;

out vec4 FragColor;

void main () {
  FragColor = texture(color_buffer, uv) * vec4(color, 1);
}
