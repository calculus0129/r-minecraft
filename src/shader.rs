use crate::gl_call;

use gl;
use std::{ffi::{CString, CStr}, cell::RefCell, collections::HashMap};

// CString: A type representing an owned, C-compatible, nul-terminated string with no nul bytes in the middle.
// CStr: Representation of a borrowed C string.


// Vertex Shader
// : 3D Space -> 2D Space (Position)

#[derive(Debug)]
pub struct ShaderPart {
    id: u32,
}

impl ShaderPart {
    // 함수호출
    pub fn from_source(source: &CStr, kind: gl::types::GLenum) -> Result<ShaderPart, String> {
        let id: u32 = shader_from_source(source, kind)?;
        Ok(ShaderPart { id })
    }

    // vertex shader
    pub fn from_vert_source(source: &CStr) -> Result<ShaderPart, String> {
        ShaderPart::from_source(source, gl::VERTEX_SHADER)
    }

    // fragment shader
    pub fn from_frag_source(source: &CStr) -> Result<ShaderPart, String> {
        ShaderPart::from_source (source, gl::FRAGMENT_SHADER)
    }
}

// 이걸 넣어야 실제로 shader가 제거됨.
impl Drop for ShaderPart {
    fn drop(&mut self) {
        gl_call!(gl::DeleteShader (self.id));
    }
}

// 실질적 역할 다함.
fn shader_from_source(source: &CStr, kind: gl::types::GLenum) -> Result<gl::types::GLuint, String> {
    let id: u32 = gl_call!(gl::CreateShader(kind)); // shader 만들기. -> id.
    gl_call!(gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null())); // ㅇㅇ??
    gl_call!(gl::CompileShader(id)); // compile부터 함. "문법이 맞는지 본다."

    let mut success: gl::types::GLint = 1; // 성공 여부
    gl_call!(gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success)); // iv: integer value. gl_COMPILE STATUS ㅇㅇ 그걸 success에 받음.

    // if fail
    if success == 0 {
        let mut len: gl::types::GLint = 0;
        gl_call!(gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len));

        let error = create_whitespace_cstring_with_len(len as usize);

        // 로그 보기
        gl_call!(gl::GetShaderInfoLog (
            id,
            len,
            std::ptr::null_mut(),
            error.as_ptr() as *mut gl::types::GLchar,
        ));

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

// 부가적인거.
fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));

    unsafe { CString::from_vec_unchecked(buffer) }
}

// Fragment Shader
// : 2D Space (position) -> 2D Space (color)

#[derive(Debug, Clone)]
pub struct ShaderProgram {
    id: u32,
    uniform_cache: RefCell<HashMap<String, i32>>,
}

impl ShaderProgram {
    pub fn from_shaders(vertex: ShaderPart, fragment: ShaderPart) -> Result<ShaderProgram, String> {
        let program_id = gl_call!(gl::CreateProgram());

        gl_call!(gl::AttachShader(program_id, vertex.id));
        gl_call!(gl::AttachShader(program_id, fragment.id));
        gl_call!(gl::LinkProgram(program_id));

        let mut success: gl::types::GLint = 1;
        gl_call!(gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success));

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl_call!(gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len));

            let error = create_whitespace_cstring_with_len(len as usize);

            gl_call!(gl::GetProgramInfoLog(
                program_id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar
            ));

            return Err(error.to_string_lossy().into_owned());
        }

        gl_call!(gl::DetachShader(program_id, vertex.id));
        gl_call!(gl::DetachShader(program_id, fragment.id));

        Ok(ShaderProgram { id: program_id, uniform_cache: RefCell::new(HashMap::new()) })
    }

    pub fn use_program(&self) {
        gl_call!(gl::UseProgram(self.id));
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        gl_call!(gl::DeleteProgram(self.id));
    }
}