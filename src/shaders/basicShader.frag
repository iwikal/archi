#version 430 core

in vec2 out_uv;
in mat3 out_tbn;
in vec3 out_position;

layout (location = 0) out vec3 color;
layout (location = 1) out vec3 normal;
layout (location = 2) out vec3 position;

layout (binding = 0) uniform sampler2D diffuse_texture;
layout (binding = 1) uniform sampler2D normal_texture;

float gamma = 2.2;

void main() {
  color = pow(texture(diffuse_texture, out_uv).xyz, vec3(gamma));
  vec3 normal_map = texture(normal_texture, out_uv).xyz;
  vec3 local_normal = vec3(normal_map.xy * 2.0 - 1.0, normal_map.z);
  normal = out_tbn * local_normal;
  position = out_position;
}
