#pragma include "complex.glsl"

in vec2 uv;

uniform sampler2D twiddle_indices;
uniform sampler2D input_texture;

out vec2 frag;

uniform int stage;
uniform int direction;

vec4 get_pixel(sampler2D sampler, ivec2 uv) {
  vec2 size = textureSize(sampler, 0);
  return texture(sampler, (uv + 0.5) / size);
}

vec2 get_input_pixel(ivec2 uv) {
  if (direction != 0) uv = uv.yx; // Flip coordinates
  return get_pixel(input_texture, uv).rg;
}

void main() {
  vec2 pixel_coord = gl_FragCoord.xy - 0.5;
  if (direction != 0) pixel_coord = pixel_coord.yx; // Flip coordinates

  vec4 twiddle = get_pixel(twiddle_indices, ivec2(stage, pixel_coord.x)).rgba;
  vec2 omega = twiddle.xy;
  vec2 p = get_input_pixel(ivec2(twiddle.z, pixel_coord.y));
  vec2 q = get_input_pixel(ivec2(twiddle.w, pixel_coord.y));

  // Butterfly operation
  vec2 H = p + cmul(omega, q);

  frag = H;
}
