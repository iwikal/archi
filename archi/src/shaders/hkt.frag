#pragma include "complex.glsl"

#define TAU 6.283185307179586476925286766559

in vec2 uv;

uniform sampler2D h0k_texture;

out vec2 displacement_x;
out vec2 displacement_y;
out vec2 displacement_z;

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

  float cosine = cos(w * time);
  float sine   = sin(w * time);

  // euler formula
  vec2 exp_iwt = vec2(cosine, sine);
  vec2 exp_iwt_inv = vec2(cosine, -sine);

  // dy
  displacement_y = cmul(fou_amp, exp_iwt) + cmul(fou_amp_conj, exp_iwt_inv);

  // dx
  vec2 dx = vec2(0.0, -k.x / magnitude);
  displacement_x = cmul(dx, displacement_y);

  // dz
  vec2 dz = vec2(0.0, -k.y / magnitude);
  displacement_z = cmul(dz, displacement_y);
}
