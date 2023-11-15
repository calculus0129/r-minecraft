pub mod debugging;
pub mod renderer;
pub mod shader;

pub mod texture;

pub mod shapes;

pub mod ecs;

use crate::renderer::{QuadProps, Renderer};
use crate::shader::{ShaderPart, ShaderProgram};
use crate::debugging::*;
use crate::texture::create_texture;

use rand::Rng;
use glfw::{Key, CursorMode};
use glfw::ffi::{glfwGetTime, glfwSwapInterval};
use glfw::Context;
use std::ffi::CString;
use std::os::raw::c_void;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap(); // GLFW 초기화
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6)); // OpenGL 4.6으로 띄우기
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(4)); // OpenGL 4
                                                                // 어떤 함수가 오래 걸리는지 등을 분석하는 도구: 프로파일링
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    )); // 전체? or 주요? or 간단한 것만 보여줄지를 결정.
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true)); // debug용 context 띄우기?

    let window_size = (800, 800);
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
    
    // ?
    window.set_raw_mouse_motion(true);
    // 마우스 비활성화. Different from 숨겨짐.
    window.set_cursor_mode(CursorMode::Disabled);
    window.set_cursor_pos(400.0, 400.0);

    // From 재민이's code
    // 기능이 뭐지?? 이거 없으면 glfw not loaded ~ 에러 남
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe { glfwSwapInterval(0) }; // vsync off: 0, on: 1 // 수직 동기 버퍼를 거쳐야 한다.
                                    // frame 처리와 frame 출력을 동기화한다.,
                                    // 고사양 게임에서는 끈다. Why?
                                    // 이것때문에 성능이 저하된다.
                                    // 60frame이면? 예민한 사람들은 input lag이 생긴다고 이야기한다.
                                    // 동기화를 강제로 하다보니까 버퍼를 거쳐 가야하므로 냠냠.
                                    // vsync 옵션이 존재. 제 사양이 더 크면 vsync를 킨다. ㅇㅇ
                                    // 반응속도가 중요한 게임들은 vsync를 끄는 게 많다.
                                    // 많은 frame ㅇㅇ
    
    gl_call!(gl::Enable(gl::DEBUG_OUTPUT));
    gl_call!(gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS));
    // Debug 출력 및 지정 활성화하기.
    gl_call!(gl::DebugMessageCallback(Some(debug_message_callback), 0 as *const c_void));
    gl_call!(gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, 0 as *const u32, gl::TRUE));

    // Backface: 물체의 뒷부분을 배제.
    // Frustrum: 선별해서 뒤에 있는 물체를 제외함. e.g. 그림자는 그려야 함
    // Occlusion: 물건에 의해 가려진 걸 배제.
    gl_call!(gl::Enable(gl::CULL_FACE)); // cull: 안보이는 거 구현 안함
    gl_call!(gl::CullFace(gl::BACK)); // backface만 일단 함. 원리: 광학 등이 쓰임
    gl_call!(gl::Enable(gl::DEPTH_TEST)); // 물체들의 depth를 비교함. => 뭐가 위에 나오는지 판단됨.
    gl_call!(gl::Enable(gl::BLEND));
    gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
    gl_call!(gl::Viewport(0, 0, 800, 800));

    let mut renderer = Renderer::new(100_000); // _: 쉼표 느낌

    let vert = ShaderPart::from_vert_source(&CString::new(include_str!("shaders/vert.vert")).unwrap()).unwrap();
    let frag = ShaderPart::from_frag_source(&CString::new(include_str!("shaders/frag.frag")).unwrap()).unwrap();
    let program = ShaderProgram::from_shaders(vert, frag).unwrap();

    let cobblestone = create_texture("blocks/cobblestone.png");
    gl_call!(gl::ActiveTexture(gl::TEXTURE0));
    gl_call!(gl::BindTexture(gl::TEXTURE_2D, cobblestone));

    let cube = shapes::unit_cube_array();

    let mut cube_vbo = 0;
    gl_call!(gl::CreateBuffers(1, &mut cube_vbo));
    gl_call!(gl::NamedBufferData(cube_vbo, (cube.len() * std::mem::size_of::<f32>()) as isize, cube.as_ptr() as *mut c_void, gl::STATIC_DRAW));

    let mut cube_vao = 0;
    gl_call!(gl::CreateVertexArrays(1, &mut cube_vao));

    gl_call!(gl::EnableVertexArrayAttrib(cube_vao, 0));
    gl_call!(gl::EnableVertexArrayAttrib(cube_vao, 1));

    gl_call!(gl::VertexArrayAttribFormat(cube_vao, 0, 3 as i32, gl::FLOAT, gl::FALSE, 0));
    gl_call!(gl::VertexArrayAttribFormat(cube_vao, 1, 2 as i32, gl::FLOAT, gl::FALSE, 3*std::mem::size_of::<f32>() as u32));
    
    gl_call!(gl::VertexArrayAttribBinding(cube_vao, 0, 0));
    gl_call!(gl::VertexArrayAttribBinding(cube_vao, 1, 0));

    gl_call!(gl::VertexArrayVertexBuffer(cube_vao, 0, cube_vbo, 0, (5 * std::mem::size_of::<f32>()) as i32));

    // 그리기
    gl_call!(gl::BindVertexArray(cube_vao));

    while !window.should_close() {
        glfw.poll_events(); // Event를 당겨오는 거.
                            // first 인자: f64. 프로그램 시작 이후 지난 시간(초)

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                _ => {},
            }
        }

        program.use_program();

        /*for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
        }*/
        gl_call!(gl::ClearColor(0.74, 0.84, 1.0, 1.0));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
// 화면이 front(보여지는거)와 back buffer(갱신한 윈도우)가 있는데 그걸 바꿔치기한다.

        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 36));
        
        window.swap_buffers();

    }
}