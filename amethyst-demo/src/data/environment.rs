use crate::Vector;

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
