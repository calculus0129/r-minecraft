use specs::prelude::*;
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
