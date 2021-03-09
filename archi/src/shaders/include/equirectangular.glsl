const vec2 INV_ATAN = vec2(0.1591, 0.3183);

vec2 equirectangular(vec3 direction) {
    vec3 v = normalize(direction);
    vec2 uv = vec2(atan(v.z, v.x), -asin(v.y));
    uv *= INV_ATAN;
    uv += 0.5;
    return uv;
}
