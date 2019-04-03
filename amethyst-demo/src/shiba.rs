use crate::environment::*;
use amethyst::{
    core::{timing::Time, transform::Transform},
    ecs::{Component, DenseVecStorage, Join, Read, System, WriteStorage},
};
use psyche::core::{brain::Brain, effector::EffectorID, sensor::SensorID};
use psyche_amethyst::BrainComponent;
use std::f32::consts::PI;

const SENSOR_DISTANCE: f32 = 1000.0;
const SIDE_SIGHT: f32 = PI * 0.25;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShibaComponent {
    pub left_obstacle_sensor: Option<SensorID>,
    pub right_obstacle_sensor: Option<SensorID>,
    pub left_target_sensor: Option<SensorID>,
    pub right_target_sensor: Option<SensorID>,
    pub left_turn_effector: Option<EffectorID>,
    pub right_turn_effector: Option<EffectorID>,
    pub direction: f32,
    pub speed: f32,
}

impl Component for ShibaComponent {
    type Storage = DenseVecStorage<Self>;
}

impl ShibaComponent {
    pub fn new(brain: &Brain) -> Self {
        let sensors = brain.get_sensors();
        let effectors = brain.get_effectors();
        Self {
            left_obstacle_sensor: sensors.get(0).map(|v| *v),
            right_obstacle_sensor: sensors.get(1).map(|v| *v),
            left_target_sensor: sensors.get(2).map(|v| *v),
            right_target_sensor: sensors.get(3).map(|v| *v),
            left_turn_effector: effectors.get(0).map(|v| *v),
            right_turn_effector: effectors.get(1).map(|v| *v),
            direction: 0.0,
            speed: 10.0,
        }
    }
}

pub struct ShibaSystem;

impl<'s> System<'s> for ShibaSystem {
    type SystemData = (
        WriteStorage<'s, ShibaComponent>,
        WriteStorage<'s, BrainComponent>,
        WriteStorage<'s, Transform>,
        Read<'s, EnvironmentData>,
        Read<'s, Time>,
    );

    fn run(
        &mut self,
        (mut shibas, mut brains, mut transforms, environment, time): Self::SystemData,
    ) {
        for (shiba, brain, transform) in (&mut shibas, &mut brains, &mut transforms).join() {
            let dt = time.delta_seconds();
            let t = transform.translation();
            if let Some(id) = shiba.left_obstacle_sensor {
                let (y, x) = (shiba.direction + SIDE_SIGHT).sin_cos();
                let potential =
                    environment.sample_obstacles((t.x, t.y, t.z), (x, y, 0.0), SENSOR_DISTANCE);
                if potential > 0.0 {
                    drop(brain.brain.sensor_trigger_impulse(id, potential.into()));
                }
            }
            if let Some(id) = shiba.right_obstacle_sensor {
                let (y, x) = (shiba.direction - SIDE_SIGHT).sin_cos();
                let potential =
                    environment.sample_obstacles((t.x, t.y, t.z), (x, y, 0.0), SENSOR_DISTANCE);
                if potential > 0.0 {
                    drop(brain.brain.sensor_trigger_impulse(id, potential.into()));
                }
            }
            if let Some(id) = shiba.left_target_sensor {
                let (y, x) = (shiba.direction + SIDE_SIGHT).sin_cos();
                let potential =
                    environment.sample_targets((t.x, t.y, t.z), (x, y, 0.0), SENSOR_DISTANCE);
                if potential > 0.0 {
                    drop(brain.brain.sensor_trigger_impulse(id, potential.into()));
                }
            }
            if let Some(id) = shiba.right_target_sensor {
                let (y, x) = (shiba.direction - SIDE_SIGHT).sin_cos();
                let potential =
                    environment.sample_targets((t.x, t.y, t.z), (x, y, 0.0), SENSOR_DISTANCE);
                if potential > 0.0 {
                    drop(brain.brain.sensor_trigger_impulse(id, potential.into()));
                }
            }
            if let Some(id) = shiba.left_turn_effector {
                if let Ok(potential) = brain.brain.effector_potential_release(id) {
                    shiba.direction -= potential as f32 * dt * PI;
                }
            };
            if let Some(id) = shiba.right_turn_effector {
                if let Ok(potential) = brain.brain.effector_potential_release(id) {
                    shiba.direction += potential as f32 * dt * PI;
                }
            };

            {
                let hw = 1024.0 * 0.5 * 0.25;
                let hh = 768.0 * 0.5 * 0.25;
                let (y, x) = shiba.direction.sin_cos();
                let x = (t.x + x * shiba.speed * dt).max(-hw).min(hw);
                let y = (t.y + y * shiba.speed * dt).max(-hh).min(hh);
                transform.set_x(x);
                transform.set_y(y);
            }
        }
    }
}
