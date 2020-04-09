uniform sampler2D heightmap;

in vec2 uv;
out vec4 frag;

void main() {
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

  // Sobel Filter
  normal.z = 1.0 / 8.0;
  normal.x = z0 + 2*z3 + z5 - z2 - 2*z4 - z7;
  normal.y = z0 + 2*z1 + z2 -z5 - 2*z6 - z7;

  frag.xyz = normalize(normal);
}
