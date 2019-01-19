#version 430 core

in vec4 gl_FragCoord;

layout (binding = 0) uniform sampler2DArray dither_map;
layout (binding = 1) uniform sampler2D color_buffer;
layout (location = 1) uniform int factor = 1;
layout (location = 2) uniform int frame_count = 0;

out vec4 FragColor;

vec3 dither () {
  vec2 dither_coord = gl_FragCoord.xy / textureSize(dither_map, 0).xy;
  return texture(dither_map, vec3(dither_coord, frame_count)).xyz - 0.5;
}

void main () {
  vec2 size = textureSize(color_buffer, 0);
  vec2 low_coord = (gl_FragCoord.xy - 0.5) * factor;
  vec2 high_coord = low_coord + factor;
  vec3 sum_color = vec3(0.0);
  for (float x = low_coord.x; x < high_coord.x; x++) {
    for (float y = low_coord.y; y < high_coord.y; y++) {
      vec2 uv = vec2(x, y) / size;
      sum_color += texture(color_buffer, uv).xyz;
    }
  }
  vec3 average_color = sum_color / (factor * factor);
  int color_depth = 256;
  vec3 rgb = round(average_color * color_depth + dither()) / color_depth;
  FragColor = vec4(rgb, 1.0);
}
