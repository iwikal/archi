#pragma include "complex.glsl"

#define TAU 6.283185307179586476925286766559

in vec2 uv;

uniform sampler2D h0k_texture;

out vec2 hkt_dx;
out vec2 hkt_dy;
out vec2 hkt_dz;

uniform int n = 512;
uniform int scale = 1000;
uniform float time;

const float g = 9.81;

void main(void) {
  vec2 xy = gl_FragCoord.xy - 0.5 - float(n) / 2.0;
  vec2 k = TAU * xy / scale;

  float magnitude = max(length(k), 0.00001);

  float w = sqrt(g * magnitude);

  vec4 h0k = texture(h0k_texture, uv);
  vec2 fou_amp = h0k.rg;
  vec2 fou_amp_conj = vec2(h0k.b, -h0k.a);

  float cosinus = cos(w * time);
  float sinus   = sin(w * time);

  // euler formula
  vec2 exp_iwt = vec2(cosinus, sinus);
  vec2 exp_iwt_inv = vec2(cosinus, -sinus);

  // dy
  hkt_dy = cmul(fou_amp, exp_iwt) + cmul(fou_amp_conj, exp_iwt_inv);

  // dx
  vec2 dx = vec2(0.0, -k.x / magnitude);
  hkt_dx = cmul(dx, hkt_dy);

  // dz
  vec2 dz = vec2(0.0, -k.y / magnitude);
  hkt_dz = cmul(dz, hkt_dy);
}
