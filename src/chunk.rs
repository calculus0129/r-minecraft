use crate::{debugging, shapes::unit_cube_array};
use rand::prelude::Distribution;
use rand::distributions::Standard;
use rand::random;
use std::collections::HashMap;
use std::os::raw::c_void;
use crate::{gl_call};

const CHUNK_SIZE: usize = 16;

const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

fn create_vao_vbo() -> (u32, u32) {
    let mut vao = 0;
    gl_call!(gl::CreateVertexArrays(1, &mut vao));

    // Position
    gl_call!(gl::EnableVertexArrayAttrib(vao, 0));
    gl_call!(gl::VertexArrayAttribFormat(vao, 0, 3_i32, gl::FLOAT, gl::FALSE, 0));
    gl_call!(gl::VertexArrayAttribBinding(vao, 0, 0));

    // Texture coords
    gl_call!(gl::EnableVertexArrayAttrib(vao, 1));
    gl_call!(gl::VertexArrayAttribFormat(vao, 1, 2_i32, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as u32));
    gl_call!(gl::VertexArrayAttribBinding(vao, 1, 0));

    let mut vbo = 0;
    gl_call!(gl::CreateBuffers(1, &mut vbo));
    gl_call!(gl::NamedBufferData(vbo, (180 * CHUNK_VOLUME * std::mem::size_of::<f32>()) as isize, std::ptr::null(), gl::DYNAMIC_DRAW));
    
    gl_call!(gl::VertexArrayVertexBuffer(vao, 0, vbo, 0, (5 * std::mem::size_of::<f32>()) as i32));

    (vao, vbo)
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum BlockID {
    AIR,
    DIRT,
    COBBLESTONE,
    OBSIDIAN,
}

impl Distribution<BlockID> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> BlockID {
        match rng.gen_range(0..4) {
            0 => BlockID::AIR,
            1 => BlockID::DIRT,
            2 => BlockID::COBBLESTONE,
            3 => BlockID::OBSIDIAN,
            _ => BlockID::AIR,
        }
    }
}

pub struct Chunk {
    blocks: [BlockID; CHUNK_VOLUME],
    pub vao: u32,
    vbo: u32,
    pub vertices_drawn: u32, // for information.
}

impl Chunk {
    pub fn empty() -> Chunk {
        let (vao, vbo) = create_vao_vbo();
        Chunk {
            blocks: [BlockID::AIR; CHUNK_VOLUME],
            vao,
            vbo,
            vertices_drawn: 0,
        }
    }

    pub fn full_of_block(block: BlockID) -> Chunk {
        let (vao, vbo) = create_vao_vbo();
        Chunk {
            blocks: [block; CHUNK_VOLUME],
            vao,
            vbo,
            vertices_drawn: 0,
        }
    }

    pub fn random() -> Chunk {
        let (vao, vbo) = create_vao_vbo();

        let mut chunk = Chunk {
            blocks: [BlockID::AIR; CHUNK_VOLUME],
            vao,
            vbo,
            vertices_drawn: 0,
        };

        for i in 0..chunk.blocks.len() {
            chunk.blocks[i] = random::<BlockID>();
        }

        chunk
    }

    #[inline]
    fn coords_to_index(x: usize, y: usize, z: usize) -> usize {
        y * (CHUNK_SIZE * CHUNK_SIZE) + z * CHUNK_SIZE + x
    }

    #[inline]
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockID {
        self.blocks[Chunk::coords_to_index(x, y, z)]
    }

    #[inline]
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockID) {
        self.blocks[Chunk::coords_to_index(x, y, z)] = block;
    }

    pub fn regenerate_vbo(&mut self, uv_map: &HashMap<BlockID, ((f32, f32), (f32, f32))>) {
        let mut idx = 0;
        self.vertices_drawn = 0;

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = self.get_block(x, y, z);

                    if block == BlockID::AIR {
                        continue;
                    }

                    let (uv_bl, ub_tr) = uv_map.get(&block).unwrap().clone();
                    let cube_array = unit_cube_array(x as f32, y as f32, z as f32, uv_bl, ub_tr, true, true, true, true, true, true);

                    gl_call!(gl::NamedBufferSubData (
                        self.vbo,
                        (idx * std::mem::size_of::<f32>()) as isize,
                        (cube_array.len() * std::mem::size_of::<f32>()) as isize,
                        cube_array.as_ptr() as *mut c_void,
                    ));

                    self.vertices_drawn += cube_array.len() as u32 / 5;
                    idx += cube_array.len();
                }
            }
        }
    }

}