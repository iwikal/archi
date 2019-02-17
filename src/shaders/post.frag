#version 430 core

in vec4 gl_FragCoord;

layout (binding = 0) uniform sampler2DArray dither_map;
layout (binding = 1) uniform sampler2D color_buffer;
layout (location = 1) uniform int factor = 1;
layout (location = 2) uniform int frame_count = 0;
layout (location = 3) uniform int color_depth = 256;

out vec4 FragColor;

float gamma = 2.4;

float to_srgb (float color) {
  return pow(color, 1.0 / gamma);
}

vec3 to_srgb (vec3 color) {
  return vec3(
      to_srgb(color.r),
      to_srgb(color.g),
      to_srgb(color.b)
      );
}

float to_linear (float color) {
  return pow(color, gamma);
}

vec3 to_linear (vec3 color) {
  return vec3(
      to_linear(color.r),
      to_linear(color.g),
      to_linear(color.b)
      );
}

vec3 dither () {
  vec2 dither_coord = gl_FragCoord.xy / textureSize(dither_map, 0).xy;
  vec3 value = texture(dither_map, vec3(dither_coord, frame_count)).xyz;
  return value * 255 / 256;
}

vec3 quantize (vec3 color) {
  return floor(color * color_depth) / (color_depth - 1);
}

vec3 downsample (vec3 color) {
  int depth = color_depth - 1;
  vec3 upscaled = depth * color;
  return floor(upscaled + dither()) / depth;
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
  FragColor.rgb = average_color;
  if (color_depth != 2) {
    FragColor.rgb = to_srgb(FragColor.rgb);
  };
  FragColor.rgb = downsample(FragColor.rgb);
}
