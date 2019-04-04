use psyche::core::{brain::Brain, brain_builder::BrainBuilder, config::Config as BrainConfig};

#[derive(Debug)]
pub struct SimulationData {
    pub brain: Brain,
}

impl Default for SimulationData {
    fn default() -> Self {
        let mut config = BrainConfig::default();
        config.propagation_speed = 50.0;
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
            brain: brain_builder.build(),
        }
    }
}
