#version 430 core

layout (location = 1) uniform vec3 color;
layout (location = 2) uniform int frame_count = 0;
layout (binding = 0) uniform sampler2DArray dither_map;
layout (binding = 1) uniform sampler2D color_buffer;

in vec2 uv;

out vec4 FragColor;

vec3 dither () {
  vec2 dither_coord = gl_FragCoord.xy / textureSize(dither_map, 0).xy;
  return texture(dither_map, vec3(dither_coord, frame_count)).xyz - 0.5;
}

void main () {
  vec3 rgb = texture(color_buffer, uv).xyz * color + dither() / 256;
  FragColor = vec4(rgb, 1);
}
