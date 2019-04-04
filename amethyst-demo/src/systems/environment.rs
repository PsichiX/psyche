use crate::{
    components::{obstacle::ObstacleComponent, target::TargetComponent},
    data::environment::EnvironmentData,
};
use amethyst::{
    core::transform::Transform,
    ecs::{Join, ReadStorage, System, Write},
};

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
