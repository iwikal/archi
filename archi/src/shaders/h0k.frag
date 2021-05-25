#define TAU 6.283185307179586476925286766559

in vec2 uv;

uniform sampler2D gauss_noise;

out vec4 frag;

uniform int n;
uniform int scale;
uniform float amplitude;
uniform float intensity; // wind speed
uniform vec2 direction;
uniform float l; // capillary supress factor

const float g = 9.81;

float h0(vec2 k) {
  float L_ = (intensity * intensity) / g;

  float mag = max(length(k), 0.0001);
  float mag_sq = mag * mag;

  float phillips_k = amplitude / (mag_sq * mag_sq) *
    pow(dot(normalize(k), normalize(direction)), 2) *
    exp(-1.0 / (mag_sq * L_ * L_)) *
    exp(-mag_sq * l * l);

  return clamp(sqrt(phillips_k) / sqrt(2.0), -4000.0, 4000.0);
}

void main(void) {
  vec2 xy = gl_FragCoord.xy - float(n) / 2.0;
  vec2 k = TAU * xy / scale;

  vec4 noise = texture(gauss_noise, uv);
  frag.xy = noise.xy * h0(k);
  frag.zw = noise.zw * h0(-k);
}
