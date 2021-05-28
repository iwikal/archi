uniform float choppiness = 1.0;

uniform sampler2D xmap;
uniform sampler2D ymap;
uniform sampler2D zmap;

in vec2 uv;

out vec3 displacementmap;
out vec3 grad_jacobian_map;

void main() {
  displacementmap = vec3(
      textureLod(xmap, uv, 0).r,
      textureLod(ymap, uv, 0).r,
      textureLod(zmap, uv, 0).r);

  vec3 dD_dx = dFdx(displacementmap);
  vec3 dD_dy = dFdy(displacementmap);

  mat2 jacobian = mat2(
      dD_dx.x, dD_dx.y,
      dD_dy.x, dD_dy.y);

  vec2 gradient = vec2(dD_dx.y, dD_dy.y);
  float jacobian_det = determinant(jacobian * choppiness);

  grad_jacobian_map = vec3(gradient, jacobian_det);
}
