layout (location = 0) in vec2 uv;
layout (location = 1) in vec3 position;

out vec4 frag;

uniform sampler2D heightmap;
uniform sampler2D normalmap;

const vec3 light_dir = vec3(1.0, 0.25, 0.0);

vec3 sobel_normal() {
  // z0 -- z1 -- z2
  // |     |     |
  // z3 -- h  -- z4
  // |     |     |
  // z5 -- z6 -- z7

  float texel = 1.0 / textureSize(heightmap, 0).x;

  float z0 = texture(heightmap, uv + vec2(-texel, -texel)).r;
  float z1 = texture(heightmap, uv + vec2(     0, -texel)).r;
  float z2 = texture(heightmap, uv + vec2( texel, -texel)).r;
  float z3 = texture(heightmap, uv + vec2(-texel,      0)).r;
  float z4 = texture(heightmap, uv + vec2( texel,      0)).r;
  float z5 = texture(heightmap, uv + vec2(-texel,  texel)).r;
  float z6 = texture(heightmap, uv + vec2(     0,  texel)).r;
  float z7 = texture(heightmap, uv + vec2( texel,  texel)).r;

  vec3 normal;

  normal.z = 1.0 / 8.0;
  normal.x = z0 + 2*z3 + z5 - z2 - 2*z4 - z7;
  normal.y = z0 + 2*z1 + z2 -z5 - 2*z6 - z7;

  return normalize(normal);
}

void main() {
  vec3 world_normal = sobel_normal();
  vec3 dir = normalize(light_dir);
  frag = vec4(vec3(max(0.0, dot(dir, world_normal))), 1.0);
}
