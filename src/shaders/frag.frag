#version 460 core

out vec4 fragColor;

in float texture_id;
in vec2 texture_coords;

uniform sampler2D textures[32]; // 32개 텍스처 지원

// 여기에서 color를 처리한다.
// e.g. 흑백 처리, 금속성 광택 처리

void main() {
    //fragColor = outColor;
    int texture_id_int = int(texture_id);
    fragColor = texture(textures[texture_id_int], texture_coords);
}