use amethyst::ecs::{Component, DenseVecStorage};

pub struct TargetComponent;

impl Component for TargetComponent {
    type Storage = DenseVecStorage<Self>;
}
