#version 430 core

in vec4 gl_FragCoord;

layout (location = 2) uniform vec3 center;
layout (location = 3) uniform vec3 light_color;
layout (location = 4) uniform float radius;

layout (binding = 0) uniform sampler2D color_buffer;
layout (binding = 1) uniform sampler2D normal_buffer;
layout (binding = 2) uniform sampler2D position_buffer;
layout (binding = 3) uniform sampler2D depth_buffer;

out vec4 FragColor;

float attenuation (vec3 position, vec3 center) {
  float d = distance(position, center) / radius;
  float linear = 1.0 - d;
  float quadratic = 1.0 / (d * d);
  return min(linear, quadratic);
}

void main () {
  vec2 size = textureSize(color_buffer, 0);
  vec2 uv = gl_FragCoord.xy / size;
  vec3 color = texture(color_buffer, uv).xyz;
  vec3 normal = texture(normal_buffer, uv).xyz;
  vec3 position = texture(position_buffer, uv).xyz;
  vec3 light_dir = normalize(center - position);
  float diff = max(dot(normal, light_dir), 0.0);
  float att = attenuation(position, center);
  FragColor = vec4(color * light_color * att * diff, 0);
}
