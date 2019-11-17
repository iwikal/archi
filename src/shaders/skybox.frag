in vec3 sample_direction;

uniform samplerCube cubemap;

out vec4 frag;

void main() {
  frag = vec4(1.0);
  vec3 stars = texture(cubemap, sample_direction).rgb;
  vec3 direction = normalize(sample_direction);
  vec3 air = vec3(0.07);
  frag.rgb = mix(air, stars, max(direction.y, 0.0));
  frag.a = 1.0;
}
