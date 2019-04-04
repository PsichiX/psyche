use amethyst::ecs::{Component, DenseVecStorage};
use psyche::core::{brain::Brain, effector::EffectorID, sensor::SensorID};

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
            speed: 20.0,
        }
    }
}
