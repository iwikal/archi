// Complex multiplication
vec2 cmul(vec2 c0, vec2 c1) {
  vec2 c;
  c.x = c0.x * c1.x - c0.y * c1.y;
  c.y = c0.x * c1.y + c0.y * c1.x;
  return c;
}
