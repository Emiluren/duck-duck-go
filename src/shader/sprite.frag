#version 330
uniform sampler2D tex;

in vec2 v_UV;
out vec4 Target0;

void main() {
    Target0 = vec4(texture(tex, v_UV));
}
