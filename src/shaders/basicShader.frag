#version 430 core

in vec2 out_uv;
in vec3 out_normal;
in vec3 out_position;

layout (location = 0) out vec3 color;
layout (location = 1) out vec3 normal;
layout (location = 2) out vec3 position;

layout (binding = 0) uniform sampler2D diffuse_texture;

void main() {
  color = pow(texture(diffuse_texture, out_uv).xyz, vec3(2.4));
  normal = out_normal;
  position = out_position;
}
