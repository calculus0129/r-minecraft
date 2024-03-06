use crate::{gl_call, shader::ShaderProgram};
use std::{os::raw::c_void, collections::HashMap, cmp::Ordering};
use itertools::Itertools;

// quad: quadrangle
#[derive(Clone, Debug)]
pub struct QuadProps {
    pub position: (f32, f32, f32),
    pub size: (f32, f32), // width, height
    pub texture_id: u32,
    pub texture_coords: (f32, f32, f32, f32),
    // pub texture_coords: (f32, f32) => 0.0, 1.0
    // texture id + coords가 총 3개, 기존 4개에서 1개 줄음.
}

//println!("{:?}"): debug trait 호출

// https://whilescape.tistory.com/entry/OpenGL-%EC%98%A4%ED%94%88%EC%A7%80%EC%97%98-%EB%8D%B0%EC%9D%B4%ED%84%B0-%EA%B4%80%EB%A0%A8-%EA%B0%9C%EB%85%90-%EC%A0%95%EB%A6%AC1
// Implementation: https://t1.daumcdn.net/cfile/tistory/998E70495C7FC78239?original
pub struct Renderer {
    texture_units: u32,
    quads: HashMap<u32, Vec<QuadProps>>,
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

impl Default for Renderer {
    fn default() -> Self {
        Renderer::new(1_000_000)
    }
}

impl Renderer {
    // capacity: vertex array 공간 크기
    pub fn new(capacity: usize) -> Self {
        let mut texture_units: i32 = 0;
        // MAX_TEXTURE_IMAGE_UNITS을 가져온다. i.e. 한 image 안에 몇 개의 texture를 넣을 수 있는지
        gl_call!(gl::GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut texture_units));
        assert!(texture_units > 0);

        let texture_units = texture_units as u32;
        let quads: HashMap<u32, Vec<QuadProps>> = HashMap::new();

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
        let binding_index_pos = 0;
        let binding_index_color = 1;

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
            3, // 크기는 2. 2칸짜리를 만듦. 값이 x, y 2개만 넣을 꺼임.
            gl::FLOAT, // type
            gl::FALSE, // 정규화되어있?
            0 // 상대적인 offset
        ));

        // Binding

        // VAO와 VBO의 index를 연결. binding_index_pos만 binding 시켜줌. Fig 3 or 4 참고.
        gl_call!(gl::VertexArrayAttribBinding(vao, 0, binding_index_pos)); // vertex attribute와 vertex buffer를 연결
        // Buffer 공간을 VAO에 binding시키는 것.
        // 설명 in figure 5
        // 6 -> 5 이유: color->texture_id, 
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
            3, // 크기는 4. 4칸짜리를 만듦. r, g, b, a 4가지를 넣을 꺼임.
            gl::FLOAT, // type
            gl::FALSE, // 정규화되어있?
            (3 * std::mem::size_of::<f32>()) as u32 // 상대적인 offset
        ));

        // Color Binding
        gl_call!(gl::VertexArrayAttribBinding(vao, 1, binding_index_color));
        gl_call!(gl::VertexArrayVertexBuffer(
            vao,
            binding_index_color,
            vbo,
            0,
            (6 * std::mem::size_of::<f32>()) as i32
        ));

        Renderer {
            texture_units,
            quads,
            vertices,
            vbo,
            vao,
        }

    }

    // 데이터를 넣기: batch한다.
    pub fn begin_batch(&mut self) {
        self.quads.clear();
        self.vertices.clear();
    }

    // 1 quad 여러 textures. texture id마다 quad props를 하나하나 넣은 거임.
    pub fn submit_quad(&mut self, quad_props: QuadProps) {
        match self.quads.get_mut(&quad_props.texture_id) {
            Some (quads) => quads ,
            None => {
                self.quads.insert(quad_props.texture_id, Vec::new());
                self.quads.get_mut(&quad_props.texture_id).unwrap()
            }
        }.push(quad_props);
    }

    pub fn end_batch(&mut self, program: &mut ShaderProgram) {
        let mut draw_calls = 0;

        // TODO: Handle quads without textures

        for vec in self.quads.values_mut() {
            vec.sort_by(|a, b| {
                if a.position.2 < b.position.2 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }); // z값에 따라 오름차순 정렬 => 밑에서부터 구성되도록 => 덮어 씌워지게
        }

        // chunk를 HW에서 소화할 수 있는 texture 개수. 그 단위로 쪼갠 것.
        let chunks = &self.quads.keys().chunks(self.texture_units as usize);
        // 이게 1이면 draw call (texture 생성 횟수)가 늘어나서 느려짐. 그게 핵심!

        for chunk in chunks {
            let mut tex_units = Vec::new();
            self.vertices.clear();

            for(tex_unit, &texture_id) in chunk.enumerate() {
                for quad in &self.quads[&texture_id] {
                    let QuadProps { position: (x, y, z), size: (w, h), texture_id: _, texture_coords:  (tex_x_min, tex_y_min, tex_x_max, tex_y_max)} = *quad;

                    let tex_unit = tex_unit as f32;
                    self.vertices.extend_from_slice(&[x, y, z, tex_unit, tex_x_min, tex_y_min]);
                    self.vertices.extend_from_slice(&[x + w, y, z, tex_unit, tex_x_max, tex_y_min]);
                    self.vertices.extend_from_slice(&[x + w, y + h, z, tex_unit, tex_x_max, tex_y_max]);
                    self.vertices.extend_from_slice(&[x + w, y + h, z, tex_unit, tex_x_max, tex_y_max]);
                    self.vertices.extend_from_slice(&[x, y + h, z, tex_unit, tex_x_min, tex_y_max]);
                    self.vertices.extend_from_slice(&[x, y, z, tex_unit, tex_x_min, tex_y_min]);
                }

                gl_call!(gl::BindTextureUnit(tex_unit as u32, texture_id));
                tex_units.push(tex_unit as i32);
            }

            // program: ShaderProgram. 여기에 이런 변수를 넣는다.
            program.set_uniform1iv("textures", tex_units.as_slice()); // Texture id 목록을 uniform 변수 이름에 넣느나.

            // vertices Buffer의 subset임
            // 데이터를 실제로 vertices를 vbo에 넣음.
            gl_call!(gl::NamedBufferSubData(self.vbo, 0 as isize, (self.vertices.len() * std::mem::size_of::<f32>()) as isize, self.vertices.as_ptr() as *mut c_void));

            // vertex array를 Graphic card에 그리라고 보내는 것
            gl_call!(gl::BindVertexArray(self.vao)); // 실제로 bind.
            gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32)); // Triangles를 vertices.len()만큼 형성해주라

            draw_calls += 1; // e.g. 100개인데 texture_unit이 15이면 7번 그려야 하니까 draw_calls는 7번임.
        }
        
        println!("Total draw calls: {draw_calls}");
    }

}
