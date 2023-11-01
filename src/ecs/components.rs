use specs::{Component, VecStorage};

#[derive(Debug)]
pub struct Position(pub f32, pub f32, pub f32);

impl Component for Position {
    type Storage = VecStorage<Self>; // 벡터를 써서 그럼
}

#[derive(Debug)]
pub struct Velocity(pub f32, pub f32);

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}



