// TODO: add disabling synapse from propagation when receptors meet upper treshold (overdose effect).

use crate::config::Config;
use crate::effector::{Effector, EffectorID};
use crate::error::*;
use crate::id::ID;
use crate::neuron::{Impulse, Neuron, NeuronID, Position, Synapse};
use crate::sensor::{Sensor, SensorID};
use crate::Scalar;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::slice::Iter;

pub type BrainID = ID<Brain>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct BrainActivityMap {
    pub connections: Vec<(Position, Position)>,
    pub impulses: Vec<(Position, Position, Scalar)>,
    pub sensors: Vec<Position>,
    pub effectors: Vec<Position>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Brain {
    id: BrainID,
    neurons: Vec<Neuron>,
    synapses: Vec<Synapse>,
    sensors: Vec<Sensor>,
    effectors: Vec<Effector>,
    config: Config,
}

impl Brain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn duplicate(&self) -> Self {
        let id = Default::default();
        let neuron_indices = self.neurons.iter().map(|n| n.id()).collect::<Vec<_>>();
        let neurons = self
            .neurons
            .iter()
            .map(|n| Neuron::new(id, n.position()))
            .collect::<Vec<_>>();
        let synapses = self
            .synapses
            .iter()
            .map(|s| Synapse {
                source: neurons[neuron_indices.iter().position(|n| *n == s.source).unwrap()].id(),
                target: neurons[neuron_indices.iter().position(|n| *n == s.target).unwrap()].id(),
                distance: s.distance,
                receptors: s.receptors,
                impulses: vec![],
                inactivity: 0.0,
            })
            .collect::<Vec<_>>();
        let sensors = self
            .sensors
            .iter()
            .map(|s| Sensor {
                id: s.id,
                target: neurons[neuron_indices.iter().position(|n| *n == s.target).unwrap()].id(),
            })
            .collect::<Vec<_>>();
        let effectors = self
            .effectors
            .iter()
            .map(|e| Effector {
                id: e.id,
                source: neurons[neuron_indices.iter().position(|n| *n == e.source).unwrap()].id(),
                potential: 0.0,
            })
            .collect::<Vec<_>>();
        Self {
            id,
            neurons,
            synapses,
            sensors,
            effectors,
            config: self.config.clone(),
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut rng = thread_rng();
        let id = Default::default();
        let brain_a = self.duplicate();
        let brain_b = other.duplicate();
        let neurons_count = (brain_a.neurons.len() + brain_b.neurons.len()) / 2;
        let synapses_count = (brain_a.synapses.len() + brain_b.synapses.len()) / 2;
        let sensors_count = (brain_a.sensors.len() + brain_b.sensors.len()) / 2;
        let effectors_count = (brain_a.effectors.len() + brain_b.effectors.len()) / 2;
        let neurons = brain_a
            .neurons
            .iter()
            .chain(brain_b.neurons.iter())
            .map(|n| Neuron::with_id(n.id(), id, n.position()))
            .collect();
        let synapses = brain_a
            .synapses
            .iter()
            .chain(brain_b.synapses.iter())
            .cloned()
            .collect();
        let sensors = brain_a
            .sensors
            .iter()
            .chain(brain_b.sensors.iter())
            .cloned()
            .collect();
        let effectors = brain_a
            .effectors
            .iter()
            .chain(brain_b.effectors.iter())
            .cloned()
            .collect();
        let mut brain = Self {
            id,
            neurons,
            synapses,
            sensors,
            effectors,
            config: brain_a.config().merge(brain_b.config()),
        };
        while brain.neurons.len() > neurons_count {
            if brain
                .kill_neuron(
                    brain.neurons[rng.gen_range(0, brain.neurons.len()) % brain.neurons.len()].id(),
                )
                .is_err()
            {
                break;
            }
        }
        while brain.sensors.len() > sensors_count {
            let id = brain.sensors[rng.gen_range(0, brain.sensors.len()) % brain.sensors.len()].id;
            if brain.kill_sensor(id).is_err() {
                break;
            }
        }
        while brain.effectors.len() > effectors_count {
            let id =
                brain.effectors[rng.gen_range(0, brain.effectors.len()) % brain.effectors.len()].id;
            if brain.kill_effector(id).is_err() {
                break;
            }
        }
        while brain.synapses.len() > synapses_count {
            let (from, to) = {
                let index = rng.gen_range(0, brain.synapses.len()) % brain.synapses.len();
                let synapse = &brain.synapses[index];
                (synapse.source, synapse.target)
            };
            if brain.unbind_neurons(from, to).is_err() {
                break;
            }
        }
        brain
    }

    #[inline]
    pub fn id(&self) -> BrainID {
        self.id
    }

    #[inline]
    pub fn get_neurons(&self) -> Vec<NeuronID> {
        self.neurons.iter().map(|n| n.id()).collect()
    }

    #[inline]
    pub fn get_sensors(&self) -> Vec<SensorID> {
        self.sensors.iter().map(|s| s.id).collect()
    }

    #[inline]
    pub fn get_effectors(&self) -> Vec<EffectorID> {
        self.effectors.iter().map(|e| e.id).collect()
    }

    #[inline]
    pub fn synapses_count(&self) -> usize {
        self.synapses.len()
    }

    pub fn clear(&mut self) {
        self.neurons.clear();
        self.synapses.clear();
        self.sensors.clear();
        self.effectors.clear();
    }

    #[inline]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[inline]
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    #[inline]
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    #[inline]
    pub fn neuron(&self, id: NeuronID) -> Option<&Neuron> {
        self.neurons.iter().find(|n| n.id() == id)
    }

    #[inline]
    pub fn neuron_mut(&mut self, id: NeuronID) -> Option<&mut Neuron> {
        self.neurons.iter_mut().find(|n| n.id() == id)
    }

    #[inline]
    pub fn neurons(&self) -> Iter<Neuron> {
        self.neurons.iter()
    }

    #[inline]
    pub fn are_neurons_connected(&self, from: NeuronID, to: NeuronID) -> bool {
        self.synapses
            .iter()
            .any(|s| s.source == from && s.target == to)
    }

    pub fn create_sensor(&mut self, target: NeuronID) -> Result<SensorID> {
        if let Some(sensor) = self.sensors.iter().find(|s| s.target == target) {
            return Err(Error::NeuronIsAlreadyConnectedToSensor(target, sensor.id));
        }
        if let Some(effector) = self.effectors.iter().find(|e| e.source == target) {
            return Err(Error::NeuronIsAlreadyConnectedToEffector(
                target,
                effector.id,
            ));
        }
        let sensor = Sensor {
            id: Default::default(),
            target,
        };
        let id = sensor.id;
        self.sensors.push(sensor);
        Ok(id)
    }

    pub fn kill_sensor(&mut self, id: SensorID) -> Result<()> {
        if let Some(index) = self.sensors.iter().position(|s| s.id == id) {
            self.sensors.swap_remove(index);
            Ok(())
        } else {
            Err(Error::SensorDoesNotExists(id))
        }
    }

    pub fn sensor_trigger_impulse(&mut self, id: SensorID, potential: Scalar) -> Result<()> {
        if let Some(sensor) = self.sensors.iter().find(|s| s.id == id) {
            if let Some(neuron) = self.neurons.iter_mut().find(|n| n.id() == sensor.target) {
                neuron.push_impulse(Impulse {
                    value: potential,
                    timeout: self.config.neuron_impulse_decay,
                });
                Ok(())
            } else {
                Err(Error::NeuronDoesNotExists(sensor.target))
            }
        } else {
            Err(Error::SensorDoesNotExists(id))
        }
    }

    pub fn create_effector(&mut self, source: NeuronID) -> Result<EffectorID> {
        if let Some(sensor) = self.sensors.iter().find(|s| s.target == source) {
            return Err(Error::NeuronIsAlreadyConnectedToSensor(source, sensor.id));
        }
        if let Some(effector) = self.effectors.iter().find(|e| e.source == source) {
            return Err(Error::NeuronIsAlreadyConnectedToEffector(
                source,
                effector.id,
            ));
        }
        let effector = Effector {
            id: Default::default(),
            source,
            potential: 0.0,
        };
        let id = effector.id;
        self.effectors.push(effector);
        Ok(id)
    }

    pub fn kill_effector(&mut self, id: EffectorID) -> Result<()> {
        if let Some(index) = self.effectors.iter().position(|e| e.id == id) {
            self.effectors.swap_remove(index);
            Ok(())
        } else {
            Err(Error::EffectorDoesNotExists(id))
        }
    }

    pub fn effector_potential_release(&mut self, id: EffectorID) -> Result<Scalar> {
        if let Some(effector) = self.effectors.iter_mut().find(|e| e.id == id) {
            let potential = effector.potential;
            effector.potential = 0.0;
            Ok(potential)
        } else {
            Err(Error::EffectorDoesNotExists(id))
        }
    }

    pub fn create_neuron(&mut self, position: Position) -> NeuronID {
        let neuron = Neuron::new(self.id, position);
        let id = neuron.id();
        self.neurons.push(neuron);
        id
    }

    pub fn kill_neuron(&mut self, id: NeuronID) -> Result<()> {
        if let Some(index) = self.neurons.iter().position(|n| n.id() == id) {
            self.neurons.swap_remove(index);
            while let Some(index) = self
                .synapses
                .iter()
                .position(|s| s.source == id || s.target == id)
            {
                self.synapses.swap_remove(index);
            }
            while let Some(index) = self.sensors.iter().position(|s| s.target == id) {
                self.sensors.swap_remove(index);
            }
            while let Some(index) = self.effectors.iter().position(|e| e.source == id) {
                self.effectors.swap_remove(index);
            }
            Ok(())
        } else {
            Err(Error::NeuronDoesNotExists(id))
        }
    }

    pub fn bind_neurons(&mut self, from: NeuronID, to: NeuronID) -> Result<bool> {
        if from == to {
            return Err(Error::BindingNeuronToItSelf(from));
        }
        if let Some(source) = self.neuron(from) {
            if let Some(target) = self.neuron(to) {
                if !self.config.allow_sensors_both_way_connections {
                    if let Some(sensor) = self.sensors.iter().find(|s| s.target == to) {
                        return Err(Error::BindingNeuronToSensor(to, sensor.id));
                    }
                }
                if !self.config.allow_effectors_both_way_connections {
                    if let Some(effector) = self.effectors.iter().find(|e| e.source == from) {
                        return Err(Error::BindingEffectorToNeuron(effector.id, from));
                    }
                }
                if self.are_neurons_connected(from, to) {
                    return Ok(false);
                }
                let distance = source.position().distance(target.position());
                self.synapses.push(Synapse {
                    source: from,
                    target: to,
                    distance,
                    receptors: thread_rng().gen_range(
                        self.config.default_receptors.0,
                        self.config.default_receptors.1,
                    ),
                    impulses: vec![],
                    inactivity: 0.0,
                });
                Ok(true)
            } else {
                Err(Error::NeuronDoesNotExists(to))
            }
        } else {
            Err(Error::NeuronDoesNotExists(from))
        }
    }

    pub fn unbind_neurons(&mut self, from: NeuronID, to: NeuronID) -> Result<bool> {
        if from == to {
            return Err(Error::UnbindingNeuronFromItSelf(from));
        }
        if self.neurons.iter().any(|n| n.id() == from) {
            if self.neurons.iter().any(|n| n.id() == to) {
                if let Some(index) = self
                    .synapses
                    .iter()
                    .position(|s| s.source == from && s.target == to)
                {
                    self.synapses.swap_remove(index);
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Err(Error::NeuronDoesNotExists(to))
            }
        } else {
            Err(Error::NeuronDoesNotExists(from))
        }
    }

    pub fn process(&mut self, delta_time: Scalar) -> Result<()> {
        let mut rng = thread_rng();

        // neuron input processing phase.
        {
            for neuron in &mut self.neurons {
                let impulse_sum = neuron.input_impulses().iter().fold(0.0, |a, i| a + i.value);
                if impulse_sum >= self.config.action_potential_treshold {
                    for s in self.synapses.iter_mut().filter(|s| s.source == neuron.id()) {
                        if s.inactivity <= 0.0 {
                            s.impulses.push(Impulse {
                                value: self.config.default_action_potential,
                                timeout: s.distance,
                            });
                            s.inactivity = self.config.synapse_inactivity_time;
                        }
                    }
                    neuron.clear_input_impulses();
                }
                neuron.set_accumulated_impulse(impulse_sum);
                neuron.process_input_impulses(delta_time, self.config.propagation_speed);
            }
        }

        // synapse propagation phase.
        {
            let mut neurons_to_trigger = vec![];
            let s = delta_time * self.config.propagation_speed;
            let r = self.config.receptors_excitation * delta_time;
            for synapse in &mut self.synapses {
                let mut excitation = 0;
                synapse.impulses = synapse
                    .impulses
                    .iter()
                    .filter_map(|impulse| {
                        let mut impulse = *impulse;
                        impulse.timeout -= s;
                        if impulse.timeout > 0.0 {
                            Some(impulse)
                        } else {
                            neurons_to_trigger.push(synapse.target);
                            excitation += 1;
                            None
                        }
                    })
                    .collect();
                synapse.receptors += Scalar::from(excitation) * r;
                synapse.inactivity = (synapse.inactivity - delta_time).max(0.0);
            }
            for id in neurons_to_trigger {
                if let Some(neuron) = self.neurons.iter_mut().find(|n| n.id() == id) {
                    neuron.push_impulse(Impulse {
                        value: self.config.default_action_potential,
                        timeout: self.config.neuron_impulse_decay,
                    })
                }
            }
        }

        // inhibition and reconnection phase.
        {
            let r = self.config.receptors_inhibition * delta_time;
            for synapse in &mut self.synapses {
                synapse.receptors -= r;
            }
            let mut neurons_to_reconnect = vec![];
            let synapses_to_remove = self
                .synapses
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    if s.receptors <= 0.0 {
                        if let Some(neuron) = self.neurons.iter().find(|n| n.id() == s.source) {
                            if let Some(id) = self.select_active_neuron(neuron.position(), &mut rng)
                            {
                                if s.source != id
                                    && (!self.config.synapse_reconnection_no_loop
                                        || (!self.are_neurons_connected(s.source, id)
                                            && !self.are_neurons_connected(id, s.source)))
                                {
                                    neurons_to_reconnect.push((s.source, id));
                                }
                            }
                        }
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            for index in synapses_to_remove.into_iter().rev() {
                self.synapses.swap_remove(index);
            }
            for (from, to) in neurons_to_reconnect {
                self.bind_neurons(from, to)?;
            }
        }

        // removing dead neurons phase.
        {
            let neurons_to_remove = self
                .neurons
                .iter()
                .enumerate()
                .filter_map(|(i, n)| {
                    let id = n.id();
                    if !self
                        .synapses
                        .iter()
                        .any(|s| s.source == id || s.target == id)
                    {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            for index in neurons_to_remove.into_iter().rev() {
                let id = self.neurons.swap_remove(index).id();
                while let Some(index) = self
                    .synapses
                    .iter()
                    .position(|s| s.source == id || s.target == id)
                {
                    self.synapses.swap_remove(index);
                }
                while let Some(index) = self.sensors.iter().position(|s| s.target == id) {
                    self.sensors.swap_remove(index);
                }
                while let Some(index) = self.effectors.iter().position(|e| e.source == id) {
                    self.effectors.swap_remove(index);
                }
            }
        }

        // accumulating effector potentials phase.
        {
            for effector in &mut self.effectors {
                if let Some(neuron) = self.neurons.iter().find(|n| n.id() == effector.source) {
                    effector.potential += neuron.accumulated_impulse();
                }
            }
        }

        Ok(())
    }

    pub fn process_parallel(&mut self, delta_time: Scalar) -> Result<()> {
        let propagation_speed = self.config.propagation_speed;
        let action_potential_treshold = self.config.action_potential_treshold;
        let synapse_inactivity_time = self.config.synapse_inactivity_time;
        let default_action_potential = self.config.default_action_potential;
        let neuron_impulse_decay = self.config.neuron_impulse_decay;

        // neuron input processing phase.
        {
            let neurons_triggering = self
                .neurons
                .par_iter_mut()
                .filter_map(|neuron| {
                    let impulse_sum = neuron.input_impulses().iter().fold(0.0, |a, i| a + i.value);
                    let status = if impulse_sum >= action_potential_treshold {
                        neuron.clear_input_impulses();
                        true
                    } else {
                        false
                    };
                    neuron.set_accumulated_impulse(impulse_sum);
                    neuron.process_input_impulses(delta_time, propagation_speed);
                    if status {
                        Some(neuron.id())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            self.synapses.par_iter_mut().for_each(|s| {
                if s.inactivity <= 0.0 {
                    let sid = s.source;
                    if neurons_triggering.iter().any(|id| sid == *id) {
                        s.impulses.push(Impulse {
                            value: default_action_potential,
                            timeout: s.distance,
                        });
                        s.inactivity = synapse_inactivity_time;
                    }
                }
            });
        }

        // synapse propagation phase.
        {
            let s = delta_time * propagation_speed;
            let r = self.config.receptors_excitation * delta_time;
            let neurons_to_trigger = self
                .synapses
                .par_iter_mut()
                .flat_map(|synapse| {
                    let mut excitation = 0;
                    let mut neurons_to_trigger = Vec::with_capacity(synapse.impulses.len());
                    synapse.impulses = synapse
                        .impulses
                        .iter()
                        .filter_map(|impulse| {
                            let mut impulse = *impulse;
                            impulse.timeout -= s;
                            if impulse.timeout > 0.0 {
                                Some(impulse)
                            } else {
                                neurons_to_trigger.push(synapse.target);
                                excitation += 1;
                                None
                            }
                        })
                        .collect();
                    synapse.receptors += Scalar::from(excitation) * r;
                    synapse.inactivity = (synapse.inactivity - delta_time).max(0.0);
                    neurons_to_trigger
                })
                .collect::<Vec<_>>();
            self.neurons.par_iter_mut().for_each(|neuron| {
                let nid = neuron.id();
                for _ in neurons_to_trigger.iter().filter(|id| nid == **id) {
                    neuron.push_impulse(Impulse {
                        value: default_action_potential,
                        timeout: neuron_impulse_decay,
                    })
                }
            });
        }

        // inhibition and reconnection phase.
        {
            let r = self.config.receptors_inhibition * delta_time;
            let synapses_to_remove = self
                .synapses
                .par_iter_mut()
                .enumerate()
                .filter_map(|(i, synapse)| {
                    synapse.receptors -= r;
                    if synapse.receptors <= 0.0 {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let neurons_to_reconnect = self
                .synapses
                .par_iter()
                .filter_map(|s| {
                    let mut rng = thread_rng();
                    if s.receptors <= 0.0 {
                        if let Some(neuron) = self.neurons.iter().find(|n| n.id() == s.source) {
                            if let Some(id) = self.select_active_neuron(neuron.position(), &mut rng)
                            {
                                if s.source != id
                                    && (!self.config.synapse_reconnection_no_loop
                                        || (!self.are_neurons_connected(s.source, id)
                                            && !self.are_neurons_connected(id, s.source)))
                                {
                                    return Some((s.source, id));
                                }
                            }
                        }
                    }
                    None
                })
                .collect::<Vec<_>>();
            for index in synapses_to_remove.into_iter().rev() {
                self.synapses.swap_remove(index);
            }
            for (from, to) in neurons_to_reconnect {
                self.bind_neurons(from, to)?;
            }
        }

        // removing dead neurons phase.
        {
            let neurons_to_remove = self
                .neurons
                .par_iter()
                .enumerate()
                .filter_map(|(i, n)| {
                    let id = n.id();
                    if !self
                        .synapses
                        .iter()
                        .any(|s| s.source == id || s.target == id)
                    {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            for index in neurons_to_remove.into_iter().rev() {
                let id = self.neurons.swap_remove(index).id();
                while let Some(index) = self
                    .synapses
                    .iter()
                    .position(|s| s.source == id || s.target == id)
                {
                    self.synapses.swap_remove(index);
                }
                while let Some(index) = self.sensors.iter().position(|s| s.target == id) {
                    self.sensors.swap_remove(index);
                }
                while let Some(index) = self.effectors.iter().position(|e| e.source == id) {
                    self.effectors.swap_remove(index);
                }
            }
        }

        // accumulating effector potentials phase.
        {
            self.effectors = self
                .effectors
                .par_iter()
                .map(|effector| {
                    let mut effector = effector.clone();
                    if let Some(neuron) = self.neurons.iter().find(|n| n.id() == effector.source) {
                        effector.potential += neuron.accumulated_impulse();
                    }
                    effector
                })
                .collect::<Vec<_>>();
        }

        Ok(())
    }

    fn select_active_neuron<R>(&self, position: Position, rng: &mut R) -> Option<NeuronID>
    where
        R: Rng,
    {
        let srr = self.config.synapse_reconnection_range;
        let filtered = self
            .neurons
            .iter()
            .filter_map(|neuron| {
                if neuron.is_active()
                    && (srr.is_none() || neuron.position().distance(position) < srr.unwrap())
                {
                    Some(neuron.id())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if filtered.is_empty() {
            None
        } else {
            Some(filtered[rng.gen_range(0, filtered.len()) % filtered.len()])
        }
    }

    pub fn build_activity_map(&self) -> BrainActivityMap {
        let connections = self
            .synapses
            .iter()
            .map(|s| {
                let from = self.neuron(s.source).unwrap().position();
                let to = self.neuron(s.target).unwrap().position();
                (from, to)
            })
            .collect();
        let impulses = self
            .synapses
            .iter()
            .map(|s| {
                let from = self.neuron(s.source).unwrap().position();
                let to = self.neuron(s.target).unwrap().position();
                let distance = from.distance(to);
                s.impulses
                    .iter()
                    .map(|i| {
                        if distance > 0.0 {
                            (from, to, 1.0 - i.timeout.max(0.0).min(distance) / distance)
                        } else {
                            (from, to, 0.0)
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();
        let sensors = self
            .sensors
            .iter()
            .map(|s| self.neuron(s.target).unwrap().position())
            .collect();
        let effectors = self
            .effectors
            .iter()
            .map(|e| self.neuron(e.source).unwrap().position())
            .collect();
        BrainActivityMap {
            connections,
            impulses,
            sensors,
            effectors,
        }
    }

    pub fn build_activity_map_parallel(&self) -> BrainActivityMap {
        let connections = self
            .synapses
            .par_iter()
            .map(|s| {
                let from = self.neuron(s.source).unwrap().position();
                let to = self.neuron(s.target).unwrap().position();
                (from, to)
            })
            .collect();
        let impulses = self
            .synapses
            .par_iter()
            .map(|s| {
                let from = self.neuron(s.source).unwrap().position();
                let to = self.neuron(s.target).unwrap().position();
                let distance = from.distance(to);
                s.impulses
                    .iter()
                    .map(|i| {
                        if distance > 0.0 {
                            (from, to, 1.0 - i.timeout.max(0.0).min(distance) / distance)
                        } else {
                            (from, to, 0.0)
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();
        let sensors = self
            .sensors
            .par_iter()
            .map(|s| self.neuron(s.target).unwrap().position())
            .collect();
        let effectors = self
            .effectors
            .par_iter()
            .map(|e| self.neuron(e.source).unwrap().position())
            .collect();
        BrainActivityMap {
            connections,
            impulses,
            sensors,
            effectors,
        }
    }

    pub fn ignite_random_synapses(&mut self, count: usize) {
        let mut rng = thread_rng();
        for _ in 0..count {
            let index = rng.gen_range(0, self.synapses.len()) % self.synapses.len();
            let synapse = &mut self.synapses[index];
            synapse.impulses.push(Impulse {
                value: self.config.default_action_potential,
                timeout: rng.gen_range(0.0, synapse.distance),
            });
        }
    }
}
