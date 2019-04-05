use amethyst::ecs::{Component, NullStorage};

#[derive(Default)]
pub struct ObstacleComponent;

impl Component for ObstacleComponent {
    type Storage = NullStorage<Self>;
}
