use crate::brain::Brain;
use crate::neuron::{NeuronID, Position};
use crate::Scalar;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffspringBuilder {
    new_neurons: usize,
    new_connections: usize,
    radius: Scalar,
    min_neurogenesis_range: Scalar,
    max_neurogenesis_range: Scalar,
    new_sensors: usize,
    new_effectors: usize,
    no_loop_connections: bool,
}

impl Default for OffspringBuilder {
    fn default() -> Self {
        Self {
            new_neurons: 1,
            new_connections: 1,
            radius: 10.0,
            min_neurogenesis_range: 0.1,
            max_neurogenesis_range: 1.0,
            new_sensors: 1,
            new_effectors: 1,
            no_loop_connections: true,
        }
    }
}

impl OffspringBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_neurons(mut self, value: usize) -> Self {
        self.new_neurons = value;
        self
    }

    pub fn new_connections(mut self, value: usize) -> Self {
        self.new_connections = value;
        self
    }

    pub fn radius(mut self, value: Scalar) -> Self {
        self.radius = value;
        self
    }

    pub fn min_neurogenesis_range(mut self, value: Scalar) -> Self {
        self.min_neurogenesis_range = value;
        self
    }

    pub fn max_neurogenesis_range(mut self, value: Scalar) -> Self {
        self.max_neurogenesis_range = value;
        self
    }

    pub fn new_sensors(mut self, value: usize) -> Self {
        self.new_sensors = value;
        self
    }

    pub fn new_effectors(mut self, value: usize) -> Self {
        self.new_effectors = value;
        self
    }

    pub fn no_loop_connections(mut self, value: bool) -> Self {
        self.no_loop_connections = value;
        self
    }

    pub fn build_mutated(mut self, source: &Brain) -> Brain {
        let mut brain = source.duplicate();
        let mut rng = thread_rng();

        let mut neurons = brain.get_neurons();
        for _ in 0..self.new_neurons {
            if let Some(neuron) = self.make_neighbor_neuron(&neurons, &mut brain, &mut rng) {
                neurons.push(neuron);
            }
        }

        let neuron_positions = neurons
            .iter()
            .map(|id| (*id, brain.neuron(*id).unwrap().position()))
            .collect::<Vec<_>>();
        for _ in 0..self.new_connections {
            self.connect_neighbor_neurons(&neuron_positions, &mut brain, &mut rng);
        }
        for _ in 0..self.new_sensors {
            self.make_peripheral_sensor(&neuron_positions, &mut brain, &mut rng);
        }
        for _ in 0..self.new_effectors {
            self.make_peripheral_effector(&neuron_positions, &mut brain, &mut rng);
        }

        brain
    }

    pub fn build_merged(mut self, source_a: &Brain, source_b: &Brain) -> Brain {
        let mut brain = source_a.merge(source_b);
        let mut rng = thread_rng();

        let mut neurons = brain.get_neurons();
        for _ in 0..self.new_neurons {
            if let Some(neuron) = self.make_neighbor_neuron(&neurons, &mut brain, &mut rng) {
                neurons.push(neuron);
            }
        }

        let diff_sensors = (source_a.get_sensors().len() + source_b.get_sensors().len()) / 2
            - brain.get_sensors().len();
        let diff_effectors = (source_a.get_effectors().len() + source_b.get_effectors().len()) / 2
            - brain.get_effectors().len();
        let neuron_positions = neurons
            .iter()
            .map(|id| (*id, brain.neuron(*id).unwrap().position()))
            .collect::<Vec<_>>();
        for _ in 0..self.new_connections {
            self.connect_neighbor_neurons(&neuron_positions, &mut brain, &mut rng);
        }
        for _ in 0..(self.new_sensors + diff_sensors) {
            self.make_peripheral_sensor(&neuron_positions, &mut brain, &mut rng);
        }
        for _ in 0..(self.new_effectors + diff_effectors) {
            self.make_peripheral_effector(&neuron_positions, &mut brain, &mut rng);
        }

        brain
    }

    fn make_peripheral_sensor<R>(
        &self,
        neuron_positions: &[(NeuronID, Position)],
        brain: &mut Brain,
        rng: &mut R,
    ) where
        R: Rng,
    {
        let pos = self.make_new_peripheral_position(rng);
        let index = neuron_positions
            .iter()
            .map(|(_, p)| p.distance_sqr(pos))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;
        brain.create_sensor(neuron_positions[index].0);
    }

    fn make_peripheral_effector<R>(
        &self,
        neuron_positions: &[(NeuronID, Position)],
        brain: &mut Brain,
        rng: &mut R,
    ) where
        R: Rng,
    {
        let pos = self.make_new_peripheral_position(rng);
        let index = neuron_positions
            .iter()
            .map(|(_, p)| p.distance_sqr(pos))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;
        brain.create_effector(neuron_positions[index].0);
    }

    fn make_neighbor_neuron<R>(
        &mut self,
        neurons: &[NeuronID],
        brain: &mut Brain,
        rng: &mut R,
    ) -> Option<NeuronID>
    where
        R: Rng,
    {
        let distance = rng.gen_range(self.min_neurogenesis_range, self.max_neurogenesis_range);
        let origin = neurons[rng.gen_range(0, neurons.len()) % neurons.len()];
        let origin_pos = brain.neuron(origin).unwrap().position();
        let new_position = self.make_new_position(origin_pos, distance, rng);
        let neuron = brain.create_neuron(new_position);
        if brain.bind_neurons(origin, neuron).is_err() {
            return None;
        }
        Some(neuron)
    }

    fn connect_neighbor_neurons<R>(
        &mut self,
        neuron_positions: &[(NeuronID, Position)],
        brain: &mut Brain,
        rng: &mut R,
    ) where
        R: Rng,
    {
        let origin =
            neuron_positions[rng.gen_range(0, neuron_positions.len()) % neuron_positions.len()];
        let filtered = neuron_positions
            .iter()
            .filter_map(|(id, p)| {
                if p.distance(origin.1) <= self.max_neurogenesis_range {
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let target = *filtered[rng.gen_range(0, filtered.len()) % filtered.len()];
        if origin.0 != target
            && (!self.no_loop_connections
                || (!brain.are_neurons_connected(origin.0, target)
                    && !brain.are_neurons_connected(target, origin.0)))
        {
            drop(brain.bind_neurons(origin.0, target));
        }
    }

    fn make_new_position<R>(&self, pos: Position, scale: Scalar, rng: &mut R) -> Position
    where
        R: Rng,
    {
        let phi = rng.gen_range(0.0, PI * 2.0);
        let theta = rng.gen_range(-PI, PI);
        let pos = Position {
            x: pos.x + theta.cos() * phi.cos() * scale,
            y: pos.y + theta.cos() * phi.sin() * scale,
            z: pos.z + theta.sin() * scale,
        };
        let magnitude = pos.magnitude();
        if magnitude > self.radius {
            Position {
                x: self.radius * pos.x / magnitude,
                y: self.radius * pos.y / magnitude,
                z: self.radius * pos.z / magnitude,
            }
        } else {
            pos
        }
    }

    fn make_new_peripheral_position<R>(&self, rng: &mut R) -> Position
    where
        R: Rng,
    {
        let phi = rng.gen_range(0.0, PI * 2.0);
        let theta = rng.gen_range(-PI, PI);
        Position {
            x: theta.cos() * phi.cos() * self.radius,
            y: theta.cos() * phi.sin() * self.radius,
            z: theta.sin() * self.radius,
        }
    }
}
