#[macro_use]
pub mod debugging;
pub mod renderer;
pub mod shader;
pub mod util;
pub mod chunk;
pub mod chunk_manager;
pub mod raycast;

pub mod texture;

pub mod shapes;

pub mod ecs;

use crate::chunk::{BlockID};
use crate::chunk_manager::ChunkManager;
use crate::renderer::{QuadProps, Renderer};
use crate::shader::{ShaderPart, ShaderProgram};
use crate::debugging::*;
use crate::texture::create_texture;
use crate::util::forward;

use rand::Rng;
use image::ColorType;
use glfw::{Key, CursorMode, Action, MouseButton};
use glfw::ffi::{glfwGetTime, glfwSwapInterval};
use glfw::Context;
use nalgebra_glm::{vec3, pi, Vec2, vec1_to_vec2, vec2, IVec3};
use nalgebra::{Vector3, clamp};

use std::ffi::CString;
use std::os::raw::c_void;
use std::borrow::Borrow;
use std::collections::HashMap;

// "Why needed?"
// input segregation is kkalkkums
pub struct InputCache {
    pub last_cursor_pos: Vec2,
    pub cursor_rel_pos: Vec2,
    pub key_states: HashMap<Key, Action>,
}

impl Default for InputCache {
    fn default() -> Self {
        InputCache {
            last_cursor_pos: vec2(0.0, 0.0),
            cursor_rel_pos: vec2(0.0, 0.0),
            key_states: HashMap::new(),
        }
    }
}

impl InputCache {
    pub fn is_key_pressed(&self, key: Key) -> bool {
        match self.key_states.get(&key) {
            Some(action) => *action == Action::Press || *action == Action::Repeat,
            None => false,
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
    // mouse button information getting
    window.set_mouse_button_polling(true); // self position informing
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

    let mut camera_position = vec3(3.0f32, 160.0, 4.0); // Temporary Value
    let mut camera_rotation = vec3(0.0f32, 0.0, 0.0);


    let mut renderer = Renderer::new(100_000); // _: 쉼표 느낌

    let vert = ShaderPart::from_vert_source(&CString::new(include_str!("shaders/diffuse.vert")).unwrap()).unwrap();
    let frag = ShaderPart::from_frag_source(&CString::new(include_str!("shaders/diffuse.frag")).unwrap()).unwrap();
    let mut program = ShaderProgram::from_shaders(vert, frag).unwrap();
    // Generate texture atlas
    let mut texture_map: HashMap<BlockID, &str> = HashMap::new();
    texture_map.insert(BlockID::DIRT, "blocks/dirt.png");
    texture_map.insert(BlockID::COBBLESTONE, "blocks/cobblestone.png");
    texture_map.insert(BlockID::OBSIDIAN, "blocks/obsidian.png");

    let mut atlas = 0;
    gl_call!(gl::CreateTextures(gl::TEXTURE_2D, 1, &mut atlas));
    gl_call!(gl::TextureParameteri(
        atlas,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST_MIPMAP_NEAREST as i32
    ));
    gl_call!(gl::TextureParameteri(
        atlas,
        gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as i32
    ));
    gl_call!(gl::TextureStorage2D(atlas, 1, gl::RGBA8, 1024, 1024,));

    let mut uv_map = HashMap::<BlockID, ((f32, f32), (f32, f32))>::new();
    let mut x = 0;
    let mut y = 0;

    for (block, texture_path) in texture_map {
        let img = image::open(texture_path);
        let img = match img {
            Ok(img) => img.flipv(),
            Err(err) => panic!("Filename: {texture_path}, error: {}", err.to_string()),
        };

        match img.color() {
            ColorType::Rgba8 => {}
            _ => panic!("Texture format not supported"),
        };

        gl_call!(gl::TextureSubImage2D(
            atlas,
            0,
            x,
            y,
            img.width() as i32,
            img.height() as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img.as_bytes().as_ptr() as *mut c_void
        ));

        uv_map.insert(
            block,
            (
                (x as f32 / 1024.0, y as f32 / 1024.0),
                ((x as f32 + 16.0) / 1024.0, (y as f32 + 16.0) / 1024.0),
            ),
        );

        x += 16;

        if x >= 1024 {
            x = 0;
            y += 16;
        }
    }

    gl_call!(gl::ActiveTexture(gl::TEXTURE0 + 0));
    gl_call!(gl::BindTexture(gl::TEXTURE_2D, atlas));

    let mut chunk_manager = ChunkManager::new();
    chunk_manager.preload_some_chunks();

    let mut input_cache = InputCache::default();
    let mut prev_cursor_pos = (0.0, 0.0);


    while !window.should_close() {
        glfw.poll_events(); // Event를 당겨오는 거.
                            // first 인자: f64. 프로그램 시작 이후 지난 시간(초)

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::CursorPos(x, y) => {
                    let rel_x = x - prev_cursor_pos.0;
                    let rel_y = y - prev_cursor_pos.1;

                    camera_rotation.y += rel_x as f32 / 100.0;
                    camera_rotation.x += rel_y as f32 / 100.0;

                    camera_rotation.x = clamp(camera_rotation.x, -pi::<f32>() / 2.0 + 0.0001, pi::<f32>() / 2.0 - 0.0001,);
                    
                    prev_cursor_pos = (x, y);
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(key, _, action, _) => {
                    input_cache.key_states.insert(key, action);
                }
                glfw::WindowEvent::MouseButton(button, Action::Press, _) => {
                    let forward = forward(&camera_rotation);
                    let get_voxel = |x: i32, y: i32, z: i32| {
                        chunk_manager.get(x, y, z).filter(|&block| block!= BlockID::AIR).and_then(|_| Some((x, y, z)))
                    };

                    let hit =
                        raycast::raycast(&get_voxel, &camera_position, &forward.normalize(), 400.0);

                    if let Some(((x, y, z), normal)) = hit {
                        if button == MouseButton::Button1 {
                            chunk_manager.set(x, y, z, BlockID::AIR)
                        } else if button == MouseButton::Button2 {
                            let near = IVec3::new(x, y, z) + normal;
                            chunk_manager.set(near.x, near.y, near.z, BlockID::DIRT)
                        }

                        println!("HIT {} {} {}", x, y, z);
                        dbg!(forward);
                    } else {
                        println!("No hit");
                    }
                }
                _ => {},
            }
        }

        let multiplier = 0.2f32;

//action == Action::Press || action == Action::Repeat
// 이따 ㄱㄱ
        if input_cache.is_key_pressed(Key::W) {
            camera_position += forward(&camera_rotation).scale(multiplier);
        }
        if input_cache.is_key_pressed(Key::S) {
            camera_position -= forward(&camera_rotation).scale(multiplier);
        }
        if input_cache.is_key_pressed(Key::A) {
            camera_position -= forward(&camera_rotation)
                .cross(&Vector3::y())
                .scale(multiplier);
        }
        if input_cache.is_key_pressed(Key::D) {
            camera_position += forward(&camera_rotation)
                .cross(&Vector3::y())
                .scale(multiplier);
        }
        if input_cache.is_key_pressed(Key::Q) {
            camera_position.y += multiplier;
        }
        if input_cache.is_key_pressed(Key::Z) {
            camera_position.y -= multiplier;
        }


        let direction = forward(&camera_rotation);

        let view_matrix = nalgebra_glm::look_at(&camera_position, &(camera_position + direction), &Vector3::y());
        let projection_matrix = nalgebra_glm::perspective(1.0, pi::<f32>() / 2.0, 0.1, 1000.0);

        chunk_manager.rebuild_dirty_chunks(&uv_map);

        program.use_program();
        program.set_uniform_matrix4fv("view", view_matrix.as_ptr());
        program.set_uniform_matrix4fv("projection", projection_matrix.as_ptr());
        program.set_uniform1i("tex", 0);

        /*for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
        }*/
        gl_call!(gl::ClearColor(0.74, 0.84, 1.0, 1.0));
        gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
// 화면이 front(보여지는거)와 back buffer(갱신한 윈도우)가 있는데 그걸 바꿔치기한다.

        chunk_manager.render_loaded_chunks(&mut program);

        window.swap_buffers();

    }
}