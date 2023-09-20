#version 460 core

in vec4 outColor
out vec4 fragColor;

// 여기에서 color를 처리한다.
// e.g. 흑백 처리, 금속성 광택 처리

void main() {
    //fragColor = outColor;
    fragColor = vec4(1.0, 0.0, 0.0, 1.0);
}