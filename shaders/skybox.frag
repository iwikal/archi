in vec3 sample_direction;

uniform samplerCube cubemap;

out vec4 frag;

void main() {
  frag = vec4(1.0);
  frag.rgb = texture(cubemap, sample_direction).rgb;
  frag.a = 1.0;
}
