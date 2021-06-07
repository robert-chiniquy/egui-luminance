// luminance pragmas in the #version 300 es for all shaders

precision mediump sampler2D;
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

// 0-255 sRGB  from  0-1 linear
vec4 srgba_from_linear(vec4 rgba) {
  return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
}

// 0-1 linear  from  0-255 sRGB
vec3 linear_from_srgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(10.31475));
  vec3 lower = srgb / vec3(3294.6);
  vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
  return mix(higher, lower, vec3(cutoff));
}

// 0-1 linear  from  0-255 sRGBA
vec4 linear_from_srgba(vec4 srgba) {
  return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main() {
  // this stanza may be for webgl1?
  // We must decode the colors, since WebGL doesn't come with sRGBA textures:
  // uvec4 i = texture(u_sampler, v_tc) * uint(255);
  // vec4 fl = vec4(float(i.r), float(i.g), float(i.b), float(i.a));
  // vec4 texture_rgba = linear_from_srgba(fl);
  // end stanza which might be wrong

  // this stanza may be for webgl2?
  // if so, these two stanzas are alternatives
  // from the es300 example: The texture is set up with `SRGB8_ALPHA8`, so no need to decode here!

  vec4 texture_rgba = linear_from_srgba(texture(u_sampler, v_tc));

  // uvec4 i = texture(u_sampler, v_tc);
  // vec4 texture_rgba = vec4(float(i.r), float(i.g), float(i.b), float(i.a));
  // end second stanza which might be wrong

  /// Multiply vertex color with texture color (in linear space).
  frag_color = v_rgba * texture_rgba;

  // We must gamma-encode again since WebGL doesn't support linear blending in the framebuffer.
  frag_color = srgba_from_linear(frag_color) / 255.0;

  // WebGL doesn't support linear blending in the framebuffer,
  // so we apply this hack to at least get a bit closer to the desired blending:
  frag_color.a = pow(frag_color.a, 1.6); // Empiric nonsense

  frag_color = v_rgba;
//  frag_color.b = 0.5;
  // frag_color = vec4(0.5, 0.5, 0.5, 1.0);
}
