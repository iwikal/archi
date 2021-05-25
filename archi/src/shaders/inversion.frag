in vec2 uv;

uniform sampler2D input_texture;
uniform uint n;

out vec2 frag;

void main() {
  vec2 xy = gl_FragCoord.xy - 0.5;

  // negate every other pixel in a checkerboard-like pattern
  float perm = mod(dot(xy, xy), 2) * -2.0 + 1.0;

  vec2 h = texture(input_texture, uv).xy;
  frag = perm * h / float(n * n);
}
