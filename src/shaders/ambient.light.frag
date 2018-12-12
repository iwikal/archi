#version 430 core

layout (location = 1) uniform vec3 color;
uniform sampler2D color_buffer;

in vec2 uv;

out vec4 FragColor;

void main () {
  FragColor = texture(color_buffer, uv) * vec4(color, 1);
}
