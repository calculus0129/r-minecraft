use crate::gl_call;
use gl;
use std::os::raw::c_void;
use image::{ColorType, GenericImageView};

pub fn create_texture(path: &str) -> u32 {
    let mut id = 0;
    // 2D texture, 1개, id에 넘김.
    gl_call!(gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id));
    // texture param을 넣어주는 거.
    // MINMAP: Rendering의 속도를 향상시켜주는 technique.
    // 절반씩 크기를 줄여서 만듦. 작은 걸 불러서
    // Zoom-in, Zoom-out => 해상도가 ㅇㅇ
    // 크기에 따라서 작은 걸 불러오면 빠르니까. (즉, 멀리서 볼 때 화면 구성을 하는 '픽셀' 수를 줄인다.)

    // MIN: 축소할 때 어케 할꺼임?
    gl_call!(gl::TextureParameteri(id, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32));
    // nearest? 맨허튼 거리....
    // MAG: 확대할 때 어떻게 적용
    gl_call!(gl::TextureParameteri(id, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));

    let img = image::open(path);
    // 수직으로 뒤집기
    let img = match img {
        Ok(img) => img.flipv(),
        Err(err) => panic!("Filename: {path}, error: {}", err.to_string())
    };

    // color 맞는지 보기.
    match img.color() {
        ColorType::Rgba8 => {},
        _ => panic!("Texture format not supported")
    };
    // texture: 3D 물체 표면의 img를 감싸는 것.
    // texture를 2D 형태로 저장 ㅇㅇ
    // texture id, level: 1
    // level: img의 크기와 비슷. 확대할 때 깨지기 전에 level을 올림.
    // gl texture 종류를 알려줌.
    gl_call!(gl::TextureStorage2D(
        id, 1, gl::RGBA8, img.width() as i32, img.height() as i32
    ));

    // 이미지의 일부만 잘라서 texture로 쓰는 거.
    // 장수를 많이 하는 것 보다 큰거 한 장 => 일부만 불러오는 것도
    // => 

    // 1째줄: 상대적으로 가장 낮은 레벨임.
    // 2째줄: 전체 이미지 범위.
    // 3째줄: format 256개
    // 메모리에 저장된 픽셀 데이터 포인터.
    gl_call!(gl::TextureSubImage2D(
        id, 0,
        0, 0, img.width() as i32, img.height() as i32,
        gl::RGBA, gl::UNSIGNED_BYTE,
        img.as_bytes().as_ptr() as *mut c_void));
    // 이러면 밉맵 만들어짐.
    gl_call!(gl::GenerateTextureMipmap(id));

    id
}