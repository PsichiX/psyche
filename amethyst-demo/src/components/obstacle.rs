use amethyst::ecs::{Component, DenseVecStorage};

pub struct ObstacleComponent;

impl Component for ObstacleComponent {
    type Storage = DenseVecStorage<Self>;
}
