#version 430 core

in vec2 tex_coords;
layout(binding = 0) uniform sampler2D color_buffer;
layout(binding = 1) uniform sampler2D depth_buffer;

out vec4 FragColor;

void main () {
  vec4 color = texture(color_buffer, tex_coords);
  vec4 depth = -texture(depth_buffer, tex_coords) + 1.0;
  FragColor = color * depth;
}
