const float GAMMA = 2.1;

vec3 tonemap(vec3 hdr, float exposure) {
  vec3 mapped = 1.0 - exp(-hdr * exposure);

  return pow(mapped, vec3(1.0 / GAMMA));
}
