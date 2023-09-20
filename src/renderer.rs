use crate::gl_call;
use std::os::raw::c_void;

// quad: quadrangle
#[derive(Clone)]
pub struct QuadProps {
    pub position: (f32, f32),
    pub size: (f32, f32), // width, height
    pub color: (f32, f32, f32, f32),
}

// https://whilescape.tistory.com/entry/OpenGL-%EC%98%A4%ED%94%88%EC%A7%80%EC%97%98-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EA%B4%80%EB%A0%A8-%EA%B0%9C%EB%85%90-%EC%A0%95%EB%A6%AC1
// Implementation: https://t1.daumcdn.net/cfile/tistory/998E70495C7FC78239?original
pub struct Renderer {
    vertices: Vec<f32>,
    // OpenGL에 이런 게 있다.
    vbo: u32, // vbo: vertex buffer object: GPU에 있는 memory 버퍼. 무엇을 담고 있나? 도형같은 게 있죠? 도형을 그리는데 필요한 정보
    // e.g. 정육면체 그리려면 vbo 2개: 위치 정보 하나 + 색상을 담는 vbo
    // vbo 2개
    vao: u32, // vao: vertex array object: vbo를 담는 객체
    // vbo를 담는데 사용
    // e.g. 이 두 가지 vbo를 담은 거 1개.
    // 계산된(rendering된) object 정보를 담는 역할.
}

// 1. Shader(GPU) Attribute Var에 binding
// 2. Vertex Array(CPU) -> Vertex Array가 있으면 얘를 VBO(GPU)에 binding
// 3. 그 data를 가지고 이어준다. General Vertex Attributes (CPU)

impl Renderer {
    // capacity: vertex array 공간 크기
    pub fn new(capacity: usize) -> Self {
        let mut vertices = Vec::new();
        vertices.reserve(capacity);

        // Setup VBO
        let mut vbo = 0;
        // "buffer 좀 만들지~ 공간 내놔"
        gl_call!(gl::CreateBuffers(1, &mut vbo)); // vbo: index값

        // : Buffer object에 이름을 지정하기.
        gl_call!(gl::NamedBufferData(
            vbo,
            (capacity * std::mem::size_of::<f32>()) as isize, // size. OpenGL이 저렇게 되어 있음 ㅇㅇ
            std::ptr::null(), // 데이터를 가리키는 포인터. 지금 데이터가 없어서 null.
            gl::DYNAMIC_DRAW // 목적: 자주 바뀌는 애들. 도형 정점이 바뀌는 애들같은 것들
        ));

        // Setup VAO
        let mut vao = 0;
        let mut binding_index_pos = 0;
        let mut binding_index_color = 1;

        // Position
        // 배열 만듦
        gl_call!(gl::CreateVertexArrays(1, &mut vao));

        gl_call!(gl::EnableVertexArrayAttrib(vao, 0)); // VAO의 0번째 attrib를 활성화
        // format 지정.

        // Quad: Quadrangle

        // (x, y, r, g, b, a) * 6
        // triangle * 2가 사각형 1개. => 필요한 위치 정보는 꼭짓점 3 * 2 == 6, 색상 정보는 색깔 * 2.

        gl_call!(gl::VertexArrayAttribFormat (
            vao,
            0, // 0번 attrib
            2, // 크기는 2. 2칸짜리를 만듦. 값이 x, y 2개만 넣을 꺼임.
            gl::FLOAT, // type
            gl::FALSE, // 정규화되어있?
            0 // 상대적인 offset
        ));

        // Binding

        // VAO와 VBO의 index를 연결. binding_index_pos만 binding 시켜줌. Fig 3 or 4 참고.
        gl_call!(gl::VertexArrayAttribBinding(vao, 0, binding_index_pos)); // vertex attribute와 vertex buffer를 연결
        // Buffer 공간을 VAO에 binding시키는 것.
        // 설명 in figure 5
        // 
        gl_call!(gl::VertexArrayVertexBuffer(
            vao,
            binding_index_pos,
            vbo,
            0,
            (6 * std::mem::size_of::<f32>()) as i32
        ));

        // Color
        gl_call!(gl::EnableVertexArrayAttrib(vao, 1)); // 1번째 attrib를 활성화
        gl_call!(gl::VertexArrayAttribFormat (
            vao,
            1, // 1번 attrib
            4, // 크기는 4. 4칸짜리를 만듦. r, g, b, a 4가지를 넣을 꺼임.
            gl::FLOAT, // type
            gl::FALSE, // 정규화되어있?
            (2 * std::mem::size_of::<f32>()) as u32 // 상대적인 offset
        ));

        // Color Binding
        gl_call!(gl::VertexArrayAttribBinding(vao, 1, binding_index_color));
        gl_call!(gl::VertexArrayVertexBuffer(
            vao,
            binding_index_color,
            vbo,
            0,
            (6 * std::mem::size_of::<f32>() as isize) as i32
        ));

        Renderer {
            vertices,
            vbo,
            vao,
        }

    }

    // 데이터를 넣기: batch한다.
    pub fn begin_batch(&mut self) {
        self.vertices.clear();
    }

    pub fn submit_quad(&mut self, quad_props: QuadProps) {
        let QuadProps { position: (x, y), size: (w, h), color: (r, g, b, a) } = quad_props;

        // ccw.
        self.vertices.extend_from_slice(&[x, y, r, g, b, a]);
        self.vertices.extend_from_slice(&[x + w, y, r, g, b, a]);
        self.vertices.extend_from_slice(&[x + w, y + h, r, g, b, a]);
        self.vertices.extend_from_slice(&[x + w, y + h, r, g, b, a]);
        self.vertices.extend_from_slice(&[x, y + h, r, g, b, a]);
        self.vertices.extend_from_slice(&[x, y, r, g, b, a]);
    }

    pub fn end_batch(&mut self) {
        // vertices Buffer의 subset임
        // 데이터를 실제로 vertices를 vbo에 넣음.
        gl_call!(gl::NamedBufferSubData(self.vbo, 0 as isize, (self.vertices.len() * std::mem::size_of::<f32>()) as isize, self.vertices.as_ptr() as *mut c_void));

        // vertex array를 Graphic card에 그리라고 보내는 것
        gl_call!(gl::BindVertexArray(self.vao)); // 실제로 bind.
        gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32)); // Triangles를 vertices.len()만큼 형성해주라
    }

}
