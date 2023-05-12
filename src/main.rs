use glfw::ffi::glfwSwapInterval;
use glfw::Context;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap(); // GLFW 초기화
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6)); // OpenGL 4.6으로 띄우기
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(4)); // OpenGL 4
    // 어떤 함수가 오래 걸리는지 등을 분석하는 도구: 프로파일링
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    )); // 전체? or 주요? or 간단한 것만 보여줄지를 결정.
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true)); // debug용 context 띄우기?

    let window_size = (500, 500);
    let window_title = "Minecraft";

    let (mut window, events) = glfw
        .create_window(
            window_size.0,
            window_size.1,
            window_title,
            glfw::WindowMode::Windowed, // FullScreen과 Windowed
        )
        .expect("Failed to create GLFW window.");
    
    // Make the window's context current
    // context: 모든 OpenGL 명령을 전달하는 통신 회로, 포트 같은거
    // 이걸 설정해주어야 openGL 명령을 GLFW windows에 보낼 수 있다.
    window.make_current(); // 없으면 'GLFW Error: Cannot set swap interval without a current OpenGL or OpenGL ES context'
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);

    unsafe { glfwSwapInterval(1) };

    while !window.should_close() {
        glfw.poll_events(); // Event를 당겨오는 거.

        for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
        }

        // 화면이 front(보여지는거)와 back buffer(갱신한 윈도우)가 있는데 그걸 바꿔치기한다.
        window.swap_buffers();

        // v_sync:
        // 0: 화면 출력이 다 안되어도 back buffer update시 back buffer를 띄운다.
        // 1: 화면 출력이 다 되면 back buffer를 띄운다.
    }
}
