layout (location = 0) in vec2 uv;
layout (location = 1) in vec3 position;

out vec4 frag;

uniform sampler2D heightmap;
uniform sampler2D normalmap;

const vec3 light_dir = vec3(1.0, 0.25, 0.0);

vec3 normal() {
  return normalize(texture(normalmap, uv).xyz);
}

void main() {
  vec3 world_normal = normal();
  vec3 dir = normalize(light_dir);
  frag = vec4(vec3(max(0.0, dot(dir, world_normal))), 1.0);
}
