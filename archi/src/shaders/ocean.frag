#pragma include "tonemap.glsl"
#pragma include "equirectangular.glsl"

layout (location = 0) in vec2 uv;
layout (location = 1) in vec3 position;

out vec4 frag;

uniform vec3 camera_pos;

uniform sampler2D heightmap;
uniform sampler2D sky_texture;
uniform float exposure;

vec3 sky(vec3 direction) {
  vec2 uv = equirectangular(normalize(direction));
  vec3 atmosphere_color = texture(sky_texture, uv).rgb;

  return tonemap(atmosphere_color, exposure);
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
  normal.z = z0 + 2*z1 + z2 - z5 - 2*z6 - z7;

  return normalize(normal);
}

void main() {
  vec3 world_normal = sobel_normal();
  vec3 look_dir = normalize(camera_pos - position);

  vec3 reflected_dir = reflect(-look_dir, world_normal);
  reflected_dir.y = abs(reflected_dir.y);

  vec3 reflection = sky(reflected_dir);

  float fresnel = dot(look_dir, world_normal);
  vec3 water_color = vec3(0.0, 0.1, 0.05);

  frag.rgb = mix(reflection, water_color, fresnel);
  frag.rgb *= 0.00000001;
  frag.rgb += reflection;

  frag.a = 1.0;
}
