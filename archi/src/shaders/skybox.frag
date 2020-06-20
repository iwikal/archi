#pragma include "tonemap.glsl"
#pragma include "atmosphere.glsl"

uniform float exposure;

in vec3 sample_direction;

out vec4 frag;

void main() {
  frag = vec4(1.0);
  vec3 direction = normalize(sample_direction);

  vec3 atmosphere_color = atmosphere(
      direction,                      // normalized ray direction
      vec3(0,6372e3,0),               // ray origin
      vec3(0.0, 0.0, -1.0),            // position of the sun
      22.0,                           // intensity of the sun
      6371e3,                         // radius of the planet in meters
      6471e3,                         // radius of the atmosphere in meters
      vec3(5.5e-6, 13.0e-6, 22.4e-6), // Rayleigh scattering coefficient
      21e-6,                          // Mie scattering coefficient
      8e3,                            // Rayleigh scale height
      1.2e3,                          // Mie scale height
      0.758                           // Mie preferred scattering direction
      );

  frag.rgb = tonemap(atmosphere_color, exposure);
  frag.a = 1.0;
}
