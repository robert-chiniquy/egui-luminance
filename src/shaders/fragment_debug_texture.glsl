#define attribute in
#define varying out
#define texture2D texture
#define GL2

precision mediump float;
uniform sampler2D u_sampler;

in vec2 v_uv;
out vec4 frag;

// 0-255 sRGB  from  0-1 linear
vec3 srgb_from_linear(vec3 rgb) {
  bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
  vec3 lower = rgb * vec3(3294.6);
  vec3 higher = vec3(269.025) * pow(rgb, vec3(1.0 / 2.4)) - vec3(14.025);
  return mix(higher, lower, vec3(cutoff));
}

// 0-255 sRGBA  from  0-1 linear
vec4 srgba_from_linear(vec4 rgba) {
  return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
}

void main() {
  frag = texture(u_sampler, v_uv);
  frag = srgba_from_linear(frag);
}
