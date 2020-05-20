layout (location = 0) in vec2 uv;
layout (location = 1) in vec3 position;

out vec4 frag;

uniform vec3 camera_pos;

uniform sampler2D heightmap;
uniform sampler2D normalmap;
uniform sampler2D sky_texture;
uniform float exposure;

const vec2 INV_ATAN = vec2(0.1591, 0.3183);

vec2 sample_equirectangular(vec3 v) {
    vec2 uv = vec2(atan(v.z, v.x), -asin(v.y));
    uv *= INV_ATAN;
    uv += 0.5;
    return uv;
}

const float GAMMA = 2.1;

vec3 tonemap(vec3 hdr) {
  vec3 mapped = 1.0 - exp(-hdr * exposure);

  return pow(mapped, vec3(1.0 / GAMMA));
}

vec3 sky(vec3 direction) {
  vec2 uv = sample_equirectangular(direction);
  return tonemap(texture(sky_texture, uv).rgb);
}

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

  normal.y = 1.0 / 8.0;
  normal.x = z0 + 2*z3 + z5 - z2 - 2*z4 - z7;
  normal.z = z0 + 2*z1 + z2 -z5 - 2*z6 - z7;

  return normalize(normal);
}

void main() {
  vec3 world_normal = sobel_normal();
  vec3 look_dir = normalize(camera_pos - position);

  vec3 color = sky(reflect(-look_dir, world_normal));

  frag.rgb = color;
  frag.a = 1.0;
}
