
#version 430 core

in vec4 gl_FragCoord;
in vec2 uv;

layout (location = 1) uniform vec3 light_dir;
layout (location = 2) uniform vec3 light_color;
layout (location = 3) uniform int frame_count;

layout (binding = 0) uniform sampler2DArray dither_map;
layout (binding = 1) uniform sampler2D color_buffer;
layout (binding = 2) uniform sampler2D normal_buffer;

out vec4 FragColor;

vec3 dither () {
  vec2 dither_coord = gl_FragCoord.xy / textureSize(dither_map, 0).xy;
  return texture(dither_map, vec3(dither_coord, frame_count)).xyz - 0.5;
}

void main () {
  vec3 color = texture(color_buffer, uv).xyz;
  vec3 normal = texture(normal_buffer, uv).xyz;
  float diff = max(dot(normal, -light_dir), 0.0);
  FragColor = vec4(color * light_color * diff + dither() / 256, 1);
}
