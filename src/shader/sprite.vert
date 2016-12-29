#version 330
uniform vec2 u_Pos;
uniform float u_Aspect;
in vec2 a_Pos;
in vec2 a_UV;
out vec2 v_UV;

void main() {
    vec2 pos = (a_Pos + u_Pos) * vec2(1.0, u_Aspect);

    gl_Position = vec4(pos, 0.0, 1.0);
    v_UV = a_UV;
}
