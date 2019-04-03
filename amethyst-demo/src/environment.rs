use amethyst::{
    core::transform::Transform,
    ecs::{Component, DenseVecStorage, Join, ReadStorage, System, Write},
};

pub struct ObstacleComponent;

impl Component for ObstacleComponent {
    type Storage = DenseVecStorage<Self>;
}

pub struct TargetComponent;

impl Component for TargetComponent {
    type Storage = DenseVecStorage<Self>;
}

pub type Vector = (f32, f32, f32);

#[derive(Debug, Default)]
pub struct EnvironmentData {
    obstacles: Vec<Vector>,
    targets: Vec<Vector>,
}

impl EnvironmentData {
    pub fn set_obstacles(&mut self, items: Vec<Vector>) {
        self.obstacles = items;
    }

    pub fn set_targets(&mut self, items: Vec<Vector>) {
        self.targets = items;
    }

    pub fn sample_obstacles(&self, position: Vector, direction: Vector, distance: f32) -> f32 {
        Self::sample(&self.obstacles, position, direction, distance)
    }

    pub fn sample_targets(&self, position: Vector, direction: Vector, distance: f32) -> f32 {
        Self::sample(&self.targets, position, direction, distance)
    }

    fn sample(data: &[Vector], position: Vector, direction: Vector, distance: f32) -> f32 {
        data.iter()
            .filter_map(|pos| {
                let diff = (pos.0 - position.0, pos.1 - position.1, pos.2 - position.2);
                let len = (diff.0 * diff.0 + diff.1 * diff.1 + diff.2 * diff.2).sqrt();
                if len <= 0.0 {
                    return None;
                }
                let norm = (diff.0 / len, diff.1 / len, diff.2 / len);
                let dot = norm.0 * direction.0 + norm.1 * direction.1 + norm.2 * direction.2;
                Some((dot * (1.0 - len / distance)).max(0.0))
            })
            .sum()
    }
}

pub struct EnvironmentSystem;

impl<'s> System<'s> for EnvironmentSystem {
    type SystemData = (
        ReadStorage<'s, ObstacleComponent>,
        ReadStorage<'s, TargetComponent>,
        ReadStorage<'s, Transform>,
        Write<'s, EnvironmentData>,
    );

    fn run(&mut self, (obstacles, targets, transforms, mut data): Self::SystemData) {
        data.set_obstacles(
            (&obstacles, &transforms)
                .join()
                .map(|(_, transform)| {
                    let t = transform.translation();
                    (t.x, t.y, t.z)
                })
                .collect(),
        );
        data.set_targets(
            (&targets, &transforms)
                .join()
                .map(|(_, transform)| {
                    let t = transform.translation();
                    (t.x, t.y, t.z)
                })
                .collect(),
        );
    }
}
