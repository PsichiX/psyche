![Logo](media/psyche-logo-2-light.png)
# Psyche AI Toolset

## General idea
This is a research project about General Artificial Intelligence system loosely
based on **Practopoiesis Theory** which stands for neural network that depends
purely on its environment and instead of converting inputs into symbols that are
processed by "machine" to give output, it processes signals as energy potentials
and by evolution of connections and constant change of brain structure, it
produces "consciousness" naturally.

You can read more about Practopoiesis Theory here:
[http://www.danko-nikolic.com/practopoiesis/](http://www.danko-nikolic.com/practopoiesis/)

## Tech used
All toolset modules are written in [**Rust** programming language](https://www.rust-lang.org/)
(a multi-paradigm systems programming language focused on safety, especially
safe concurrency) and is available on [crates.io](https://crates.io/crates/psyche)
as Rust Crate ready to be your project dependency. Moreover, Psyche toolset
provides **FFI** (Foreign Function Interface) libraries for **C** and **C++**
languages, and FFI wrapper for **C#** language. There are plans for
[**Unity 3D** engine](https://unity.com/) plugin but first there is a bug to fix
on the integration side. There are also plans for [**Godot** engine](https://godotengine.org/)
plugin and [**Amethyst** engine](https://www.amethyst.rs/) integration
crate.

## Toolset modules
1. **Core** - defines brain: neurons, connections between them, production of
offsprings by evolution and all processing that makes brain functioning;
1. **Serde** - serialization and deserialization brains into different storing
data formats: Binary, JSON and YAML;
1. **Host** - for now does nothing but it will gives ability to put brain in
host body;
1. **Graphics** - produces Waveform OBJ graphics data that may be used to
visualize brain activity;
1. **Simulator CLI app** - CLI application that simulate brain activity step by
step and for each step it produces brain activity frames data as files ready to
use in external applications such as Houdini for visualizations or any analizer
application.

## Demos

#### [Brain activity visualizer](demos/src/brain-activity)
Every blue line is a connection between two neurons and every white dot is a
signal traveling through neural network.

[![psyche-demo-brain-activity](media/psyche-demo-brain-activity.gif)](media/psyche-demo-brain-activity.mp4)

#### [Spores in fluid environment](demos/src/spore)
Each spore has its own brain connected to body sensors (smell) and motors (legs)
and by that it tries to find and eat food portions left in water. You can also
manipulate environment by producing fluid currents with mouse dragging.

[![psyche-demo-spore](media/psyche-demo-spore.gif)](../media/psyche-demo-spore.mp4)

## Usage
[![Docs.rs](https://docs.rs/psyche/badge.svg)](https://docs.rs/psyche)
[![Crates.io](https://img.shields.io/crates/v/psyche.svg)](https://crates.io/crates/psyche)

Record in `Cargo.toml`:
```toml
[dependencies]
psyche = "0.2"
```

Your crate module:
```rust
extern crate psyche;

use psyche::core::brain_builder::BrainBuilder;
use psyche::core::config::Config;
use psyche::core::Scalar;

// prepare config for brain.
let mut config = Config::default();
config.propagation_speed = 50.0;
config.synapse_reconnection_range = Some(15.0);
config.neuron_potential_decay = 0.1;
config.synapse_propagation_decay = 0.01;
config.synapse_new_connection_receptors = Some(2.0);

// build brain.
let mut brain = BrainBuilder::new()
  .config(config)
  .neurons(100)
  .connections(200)
  .min_neurogenesis_range(5.0)
  .max_neurogenesis_range(15.0)
  .radius(30.0)
  .sensors(10)
  .effectors(10)
  .brain();

loop {
  // trigger sensors.
  for sensor in brain.get_sensors() {
    brain.sensor_trigger_impulse(sensor, 1.0);
  }

  // process brain step.
  brain.process(1.0);

  // read effectors and act based on their stored potential.
  for effector in brain.get_effectors() {
    if let Ok(potential) = brain.effector_potential_release(effector) {
      println!("{:?} = {:?}", effector, potential);
    }
  }
}
```
