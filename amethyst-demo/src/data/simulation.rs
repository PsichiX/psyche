use psyche::core::{
    brain::Brain, brain_builder::BrainBuilder, config::Config as BrainConfig,
    offspring_builder::OffspringBuilder,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationData {
    pub brain_scored: (Brain, f32),
    pub last_scored: Option<(Brain, f32)>,
}

impl Default for SimulationData {
    fn default() -> Self {
        let mut config = BrainConfig::default();
        config.propagation_speed = 1000.0;
        config.synapse_reconnection_range = Some(15.0);
        config.neuron_potential_decay = 0.1;
        config.synapse_propagation_decay = 0.01;
        config.synapse_new_connection_receptors = Some(2.0);
        let brain_builder = BrainBuilder::new()
            .config(config)
            .neurons(50)
            .connections(200)
            .min_neurogenesis_range(5.0)
            .max_neurogenesis_range(15.0)
            .radius(30.0)
            .sensors(4)
            .effectors(2);

        Self {
            brain_scored: (brain_builder.build(), 0.0),
            last_scored: None,
        }
    }
}

impl SimulationData {
    pub fn mutate(&mut self, score: f32) -> bool {
        let offspring_builder = OffspringBuilder::new()
            .new_neurons(2)
            .new_connections(8)
            .radius(30.0)
            .min_neurogenesis_range(5.0)
            .max_neurogenesis_range(15.0)
            .new_sensors(0)
            .new_effectors(0);

        if score > self.brain_scored.1 || self.last_scored.is_none() {
            println!("score = {}", score);
            println!("curr score = {}", self.brain_scored.1);
            println!(
                "score > self.brain_scored.1 = {}",
                score > self.brain_scored.1
            );
            println!(
                "self.last_scored.is_none() = {}",
                self.last_scored.is_none()
            );
            self.last_scored = Some(self.brain_scored.clone());
            self.brain_scored = (offspring_builder.build_mutated(&self.brain_scored.0), score);
            true
        } else {
            if let Some(last_scored) = &self.last_scored {
                self.brain_scored = (
                    offspring_builder.build_mutated(&last_scored.0),
                    last_scored.1,
                );
            }
            false
        }
    }
}
