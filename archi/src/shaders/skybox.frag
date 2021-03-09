#pragma include "tonemap.glsl"
#pragma include "equirectangular.glsl"

uniform float exposure;
uniform sampler2D sky_texture;

in vec3 sample_direction;

out vec4 frag;

void main() {
  frag = vec4(1.0);
  vec3 direction = normalize(sample_direction);

  vec2 uv = equirectangular(direction);
  vec3 atmosphere_color = texture(sky_texture, uv).rgb;

  frag.rgb = tonemap(atmosphere_color, exposure);
  frag.a = 1.0;
}
