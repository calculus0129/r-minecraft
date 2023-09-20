pub mod debugging;
pub mod renderer;
pub mod shader;

use crate::renderer::{QuadProps, Renderer};
use crate::shader::{ShaderPart, ShaderProgram};

use rand::Rng;
use glfw::{Key};
use glfw::ffi::{glfwGetTime, glfwSwapInterval};
use glfw::Context;
use std::ffi::CString;


// FPS가 잘 되는데도 60 등으로 고정하는 이유? 형평성 문제!
// 참고로 30정도만 되어도 인간들은 걍 움직임을 잘 느낀다.
// 주사율: 화면에서 표시되는 FPS

#[derive(Default)]

pub struct Framerate {
    pub frame_count: u32,
    pub last_frame_time: f64,
}

impl Framerate {
    fn run(&mut self) {
        self.frame_count += 1;

        let current_time = unsafe { glfwGetTime() };
        let delta_time = current_time - self.last_frame_time;

        // we want to print with period 1.0s
        if delta_time >= 1.0 {
            self.last_frame_time = current_time;
            println!("FPS: {}", f64::from(self.frame_count) / delta_time);
            self.frame_count = 0;
        }
    }
}

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
    
    // ?
    window.set_raw_mouse_motion(true);

    unsafe { glfwSwapInterval(0) }; // vsync off: 0, on: 1 // 수직 동기 버퍼를 거쳐야 한다.
                                    // frame 처리와 frame 출력을 동기화한다.,
                                    // 고사양 게임에서는 끈다. Why?
                                    // 이것때문에 성능이 저하된다.
                                    // 60frame이면? 예민한 사람들은 input lag이 생긴다고 이야기한다.
                                    // 동기화를 강제로 하다보니까 버퍼를 거쳐 가야하므로 냠냠.
                                    // vsync 옵션이 존재. 제 사양이 더 크면 vsync를 킨다. ㅇㅇ
                                    // 반응속도가 중요한 게임들은 vsync를 끄는 게 많다.
                                    // 많은 frame ㅇㅇ
    
    let mut renderer = Renderer::new(100_000); // _: 쉼표 느낌

    let vert = ShaderPart::from_vert_source(&CString::new(include_str!("shaders/vert.vert")).unwrap()).unwrap();
    let frag = ShaderPart::from_frag_source(&CString::new(include_str!("shaders/frag.frag")).unwrap()).unwrap();
    let program = ShaderProgram::from_shaders(vert, frag).unwrap();

    let mut framerate = Framerate {
        frame_count: 0,
        last_frame_time: 0.0,
    };

    let mut quads = Vec::new();
    let mut rng = rand::thread_rng();

    while !window.should_close() {
        glfw.poll_events(); // Event를 당겨오는 거.
                            // first 인자: f64. 프로그램 시작 이후 지난 시간(초)

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Space, _, _, _) => {
                    quads.push(QuadProps {
                        position: (
                            (window.get_cursor_pos().0 as f32).to_range(0.0, 500.0, -1.0, 1.0),
                            (window.get_cursor_pos().1 as f32).to_range(0.0, 500.0, 1.0, -1.0),
                        ),
                        size: (0.5, 0.5),
                        color: (
                            rng.gen_range(0.0..=1.0),
                            rng.gen_range(0.0..=1.0),
                            rng.gen_range(0.0..=1.0),
                            1.0
                        ),
                    });
                }
                _ => {},
            }
        }
        /*for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
        }*/
        gl_call!(gl::ClearColor(1.0, 1.0, 1.0, 1.0));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT));

        program.use_program();

        renderer.begin_batch();
        for quad in &quads {
            renderer.submit_quad(quad.clone());
        }

        renderer.end_batch();
        // 화면이 front(보여지는거)와 back buffer(갱신한 윈도우)가 있는데 그걸 바꿔치기한다.
        window.swap_buffers();

        // v_sync:
        // 0: 화면 출력이 다 안되어도 back buffer update시 back buffer를 띄운다.
        // 1: 화면 출력이 다 되면 back buffer를 띄운다.
        framerate.run();
    }
}

trait ToRange {
    fn to_range(&self, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32;
}

impl ToRange for f32 {
    fn to_range(&self, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
        let old_range = old_max - old_min;
        let new_range = new_max - new_min;

        (((self - old_min) * new_range) / old_range) + new_min
    }
}