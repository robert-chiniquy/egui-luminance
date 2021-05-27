// fragment shader
in vec3 v_normal;

// we will output a single color
out vec3 frag_color;

void main() {
  // KISS
  frag_color = v_normal;
}