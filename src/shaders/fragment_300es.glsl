#define attribute in
#define varying out
#define texture2D texture
#define GL2

precision mediump float;
uniform sampler2D u_sampler;
in vec4 v_rgba;
in vec2 v_tc;

out vec4 frag_color;

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

  // The texture is set up with `SRGB8_ALPHA8`, so no need to decode here!
  vec4 texture_rgba = texture2D(u_sampler, v_tc);

  // texture_rgba = 0.9 - texture_rgba; //?

  /// Multiply vertex color with texture color (in linear space).
  frag_color = v_rgba * texture_rgba;

// this is a hack for debugging suggested by @zicklag in https://github.com/emilk/egui/discussions/443
  // frag_color = v_rgba;
  // frag_color = texture_rgba;

  // Unmultiply alpha:
  if(frag_color.a > 0.0) {
    frag_color.rgb /= frag_color.a;
  }

  // Empiric tweak to make e.g. shadows look more like they should:
  frag_color.a *= sqrt(frag_color.a);

  // To gamma:
  frag_color = srgba_from_linear(frag_color) / 255.0;

  // Premultiply alpha, this time in gamma space:
  if(frag_color.a > 0.0) {
    frag_color.rgb *= frag_color.a;
  }

}
