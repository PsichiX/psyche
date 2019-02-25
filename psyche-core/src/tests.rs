#![cfg(test)]
use crate::brain::*;
use crate::brain_builder::*;
use crate::config::*;
use crate::neuron::*;
use crate::offspring_builder::*;

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
    let s1 = brain.create_sensor(n1).unwrap();
    let e1 = brain.create_effector(n3).unwrap();
    brain.bind_neurons(n1, n2).unwrap();
    brain.bind_neurons(n2, n3).unwrap();
    assert!(brain.bind_neurons(n3, n1).is_err());
    brain.sensor_trigger_impulse(s1, 10.0).unwrap();

    for _ in 0..4 {
        brain.process(1.0).unwrap();
    }
    assert!(brain.effector_potential_release(e1).unwrap() > 0.0);
}

#[test]
fn test_brain_builder() {
    let brain = BrainBuilder::new()
        .config(Config::default())
        .neurons(1000)
        .connections(1000)
        .min_neurogenesis_range(0.1)
        .max_neurogenesis_range(10.0)
        .radius(20.0)
        .sensors(10)
        .effectors(10)
        .build();
    // println!("brain: {:#?}", brain);
    // println!("neurons: {}", brain.get_neurons().len());
    // println!("synapses: {}", brain.synapses_count());
    // println!("sensors: {}", brain.get_sensors().len());
    // println!("effectors: {}", brain.get_effectors().len());
    // println!("brain energy: {}", brain.energy());
    // println!("brain activity: {:#?}", brain.build_activity_map());
}

#[test]
fn test_offspring_builder() {
    let brain_a = BrainBuilder::new()
        .config(Config::default())
        .neurons(1000)
        .connections(1000)
        .min_neurogenesis_range(0.1)
        .max_neurogenesis_range(10.0)
        .radius(20.0)
        .sensors(10)
        .effectors(10)
        .build();
    let brain_b = BrainBuilder::new()
        .config(Config::default())
        .neurons(1000)
        .connections(1000)
        .min_neurogenesis_range(0.1)
        .max_neurogenesis_range(10.0)
        .radius(20.0)
        .sensors(10)
        .effectors(10)
        .build();
    let brain = OffspringBuilder::new()
        .new_neurons(200)
        .new_connections(1000)
        .min_neurogenesis_range(0.1)
        .max_neurogenesis_range(10.0)
        .radius(20.0)
        .new_sensors(0)
        .new_effectors(0)
        .build_merged(&brain_a, &brain_b);
    // println!(
    //     "neurons: {} x {} = {}",
    //     brain_a.get_neurons().len(),
    //     brain_b.get_neurons().len(),
    //     brain.get_neurons().len()
    // );
    // println!(
    //     "synapses: {} x {} = {}",
    //     brain_a.synapses_count(),
    //     brain_b.synapses_count(),
    //     brain.synapses_count()
    // );
    // println!(
    //     "sensors: {} x {} = {}",
    //     brain_a.get_sensors().len(),
    //     brain_b.get_sensors().len(),
    //     brain.get_sensors().len()
    // );
    // println!(
    //     "effectors: {} x {} = {}",
    //     brain_a.get_effectors().len(),
    //     brain_b.get_effectors().len(),
    //     brain.get_effectors().len()
    // );
}
