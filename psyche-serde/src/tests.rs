#![cfg(test)]
use crate::bytes::*;
use crate::json::*;
use crate::yaml::*;
use psyche_core::brain::*;
use psyche_core::config::*;
use psyche_core::neuron::*;

#[test]
fn test_brain() {
    let mut brain = Brain::new();
    let n1 = brain.create_neuron(Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let n2 = brain.create_neuron(Position {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    });
    let n3 = brain.create_neuron(Position {
        x: 4.0,
        y: 0.0,
        z: 0.0,
    });
    let s1 = brain.create_sensor(n1);
    brain.bind_neurons(n1, n2).unwrap();
    brain.bind_neurons(n2, n3).unwrap();
    brain.create_effector(n3);
    brain.sensor_trigger_impulse(s1, 10.0, 2.0).unwrap();

    let json = brain_to_json(&brain, true).unwrap();
    let brain_json = brain_from_json(&json).unwrap();
    assert_eq!(brain, brain_json);

    let bytes = brain_to_bytes(&brain).unwrap();
    let brain_bytes = brain_from_bytes(&bytes).unwrap();
    assert_eq!(brain, brain_bytes);

    let yaml = brain_to_yaml(&brain).unwrap();
    let brain_yaml = brain_from_yaml(&yaml).unwrap();
    assert_eq!(brain, brain_yaml);
}

#[test]
fn test_brain_activity_map() {
    let mut brain = Brain::new();
    let n1 = brain.create_neuron(Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let n2 = brain.create_neuron(Position {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    });
    let n3 = brain.create_neuron(Position {
        x: 4.0,
        y: 0.0,
        z: 0.0,
    });
    brain.create_sensor(n1);
    brain.bind_neurons(n1, n2).unwrap();
    brain.bind_neurons(n2, n3).unwrap();
    brain.create_effector(n3);
    brain.ignite_random_synapses(brain.synapses_count());
    let bam = brain.build_activity_map();

    let json = brain_activity_map_to_json(&bam, true).unwrap();
    let bam_json = brain_activity_map_from_json(&json).unwrap();
    assert_eq!(bam, bam_json);

    let bytes = brain_activity_map_to_bytes(&bam).unwrap();
    let bam_bytes = brain_activity_map_from_bytes(&bytes).unwrap();
    assert_eq!(bam, bam_bytes);

    let yaml = brain_activity_map_to_yaml(&bam).unwrap();
    let bam_yaml = brain_activity_map_from_yaml(&yaml).unwrap();
    assert_eq!(bam, bam_yaml);
}

#[test]
fn test_config() {
    let config = Config::default();

    let json = config_to_json(&config, true).unwrap();
    let config_json = config_from_json(&json).unwrap();
    assert_eq!(config, config_json);

    let bytes = config_to_bytes(&config).unwrap();
    let config_bytes = config_from_bytes(&bytes).unwrap();
    assert_eq!(config, config_bytes);

    let yaml = config_to_yaml(&config).unwrap();
    let config_yaml = config_from_yaml(&yaml).unwrap();
    assert_eq!(config, config_yaml);
}
