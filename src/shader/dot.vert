#version 330
uniform vec2 u_Pos;
in vec2 a_Pos;
in vec3 a_Color;
out vec3 v_Color;

void main() {
    gl_Position = vec4(a_Pos + u_Pos, 0.0, 1.0);
    v_Color = a_Color;
}
