#extension GL_ARB_tessellation_shader : enable

layout(vertices = 4) out;

uniform int tesselation_factor;
uniform float tesselation_slope;
uniform float tesselation_shift;
uniform vec3 camera_position;

float lod_factor(float dist) {
  float level = tesselation_factor / pow(dist, tesselation_slope);
  return max(0.0, level + tesselation_shift);
}

void main() {
  if (gl_InvocationID == 0) {
    gl_TessLevelOuter[0] = 1;
    gl_TessLevelOuter[1] = 1;
    gl_TessLevelOuter[2] = 1;
    gl_TessLevelOuter[3] = 1;

    gl_TessLevelInner[0] = 1;
    gl_TessLevelInner[1] = 1;
  }

  gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;
}
