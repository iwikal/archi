
#version 430 core

in vec4 gl_FragCoord;
in vec2 uv;

layout (location = 1) uniform vec3 light_dir;
layout (location = 2) uniform vec3 light_color;

layout (binding = 0) uniform sampler2D color_buffer;
layout (binding = 1) uniform sampler2D normal_buffer;

out vec4 FragColor;

void main () {
  vec3 color = texture(color_buffer, uv).xyz;
  vec3 normal = texture(normal_buffer, uv).xyz;
  float diff = max(dot(normal, -light_dir), 0.0);
  FragColor = vec4(color * light_color * diff, 0);
}
