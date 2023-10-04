#version 460 core // GLS VERSION: 4.6 (openGL: 4 version이 좋음)

// in: 입력
layout (location = 0) in vec3 position; // vao의 0번째 attribute를 position이라는 이름의 vec2 변수에 담는다.
layout (location = 1) in vec3 texture_info; // ㅇㅇ r, g, b, a

//out vec4 outColor; // out: 내보내기 vec4 type의 outColor를 내보내기.
out float texture_id;
out vec2 texture_coords;

// vertex shader에서는 color를 걍 넘겨주는 것밖에 안하니까

void main() {
    gl_Position = vec4(position, 1.0); // gl_Position: 내장변수. 꼭짓점의 좌푯값. 0.0: vec3 (vec4로 되어있다)
    // 1.0: w?: Homogeneous Coordinate여서!
    // Unity 등에서
    // Perspective Projection: 원근 고려 투영
    // Orthogonal Projection: 원근 고려x 투영
    // vec4좌표에 Perspective projection vector를 곱한다.

    // NDC Normalized Device Coordinate. 라는 정규화된 좌표공간에 담기에도 쓰임. 해상도.

    //outColor = color; // color는 그대로 내보낸다.
    texture_id = texture_info.x;
    texture_coords = texture_info.yz;
}