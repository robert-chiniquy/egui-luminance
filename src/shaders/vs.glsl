in vec3 position;
in vec3 normal;
in vec3 color;

out vec3 v_normal;
out vec3 v_color;

uniform mat4 projection;
uniform mat4 view;

void main() {
  v_normal = normal;
  v_color = vec3(color);
  gl_Position = projection * view * vec4(position, 1.);
}