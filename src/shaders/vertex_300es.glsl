#define attribute in
#define varying out
#define texture2D texture
#define GL2

precision mediump float;
uniform vec2 u_screen_size;
attribute vec2 a_pos;
attribute vec2 a_tc;
attribute vec4 a_srgba;
varying vec4 v_rgba;
varying vec2 v_tc;

void main() {
  gl_Position = vec4(2.0 * a_pos.x / u_screen_size.x - 1.0, 1.0 - 2.0 * a_pos.y / u_screen_size.y, 0.0, 1.0);
  v_tc = a_tc;

// Luminance normalizing the integers (without gamma correction) is
// already doing the conversion expected in Egui's WebGL2 vertex shader

// Thanks to @zicklag in https://github.com/emilk/egui/discussions/443
  v_rgba = a_srgba;
}
