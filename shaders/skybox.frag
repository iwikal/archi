in vec2 v_uv;

uniform sampler2D box_face;

out vec4 frag;

void main() {
  frag = vec4(1.0);
  frag.xyz = texture(box_face, v_uv).xyz;
  frag.w = 1.0;
}
