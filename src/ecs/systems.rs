use specs::prelude::*;
use crate::renderer::Renderer;
use crate::renderer::QuadProps;
use crate::shader::ShaderProgram;

use super::components::*;
use super::resources::*;

pub struct Physics;

// entity마다 조회하고 Position이 없으면 아예 안 돌아가게끔 되는 이런 식으로 되는 거다.
// <'a>는 Read, WriteStorage, ReadStorage
impl<'a> System<'a> for Physics {
    type SystemData = ( // 이대로 argument가 들어온다.
        Read<'a, DeltaTime>, // DeltaTime 값 하나를 읽겠다.
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>); // Velocity라는 배열을 통으로 읽어들이는 느낌
    
    fn run(&mut self, (dt, mut pos, val) : Self::SystemData) {
        let dt = dt.delta.as_micros() as f32 / 1_000_000.0; // us를 100만으로 나눈다.

        for (pos, vel) in (&mut pos, &val).join() { //두 개를 합치는 iterator를 생성한다.
            pos.0 += vel.0 * dt;
            pos.1 += vel.1 * dt;
        }
    }
}

pub struct Render;

impl<'a> System<'a> for Render {
    type SystemData = ( // 이대로 argument가 들어온다.
        Write<'a, Renderer>, // DeltaTime 값 하나를 읽겠다.
        WriteExpect<'a, ShaderProgram>,
        ReadStorage<'a, Position>,
    ); // Velocity라는 배열을 통으로 읽어들이는 느낌
    
    fn run(&mut self, (mut renderer, mut shader, pos) : Self::SystemData) {
        renderer.begin_batch();

        for position in (&pos, ).join() {
            let pos = &*position.0;
            let tuple = (pos.0, pos.1, pos.2);

            renderer.submit_quad(QuadProps{
                position: tuple,
                size: (0.5, 0.5),
                texture_id: 1,
                texture_coords: (0.0, 0.0, 1.0, 1.0),
            })
        }
        shader.use_program();
        renderer.end_batch(&mut shader);
    }
}

pub struct ComputeDeltaTime;

impl<'a> System<'a> for ComputeDeltaTime {
    type SystemData = Write<'a, DeltaTime>;

    fn run(&mut self, mut dt: Self::SystemData) {
        let now = now();

        dt.delta = now - dt.prev;
        dt.prev = now;
    }
}

pub struct Bounce;

impl<'a> System<'a> for Bounce {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, (pos, mut vel): Self::SystemData) {
        for (pos, vel) in (&pos, &mut vel).join() {
            if pos.0 < -1.0 || pos.0 > 1.0 {
                vel.0 *= -1.0;
            }

            if pos.1 < -1.0 || pos.1 > 1.0 {
                vel.1 *= -1.0;
            }
        }
    }
}