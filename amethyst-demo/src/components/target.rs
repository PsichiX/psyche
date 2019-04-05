use amethyst::ecs::{Component, NullStorage};

#[derive(Default)]
pub struct TargetComponent;

impl Component for TargetComponent {
    type Storage = NullStorage<Self>;
}
