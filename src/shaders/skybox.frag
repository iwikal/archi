in vec3 sample_direction;

uniform sampler2D hdri;
uniform float exposure;

out vec4 frag;

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

void main() {
  frag = vec4(1.0);
  vec3 direction = normalize(sample_direction);

  vec2 uv = sample_equirectangular(direction);
  vec3 stars = texture(hdri, uv).rgb;

  vec3 air = vec3(0.07);

  // frag.rgb = mix(air, tonemap(stars), max(direction.y, 0.0));
  frag.rgb = tonemap(stars);
  frag.a = 1.0;
}
