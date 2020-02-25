layout (location = 0) in vec2 uv;
layout (location = 1) in vec3 position;

out vec4 frag;

uniform sampler2D heightmap;

const vec3 light_dir = vec3(1.0, 0.25, 0.0);

vec3 get_normal() {
  float height = 0.0;
  height += texture(heightmap, uv).y;
  height += texture(heightmap, uv / 0x100).y;

  vec3 p = position;
  p.y = height;

  return normalize(cross(dFdx(p), dFdy(p)));
}


void main() {
  vec3 world_normal = get_normal();
  vec3 dir = normalize(light_dir);
  frag = vec4(vec3(max(0.0, dot(dir, world_normal))), 1.0);
}
