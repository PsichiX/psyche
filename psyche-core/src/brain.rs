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

pub mod activity {
    pub const NONE: usize = 0;
    pub const CONNECTIONS: usize = 1;
    pub const IMPULSES: usize = 1 << 1;
    pub const SENSORS: usize = 1 << 2;
    pub const EFFECTORS: usize = 1 << 3;
    pub const NEURONS: usize = 1 << 4;
    pub const ALL: usize = 0xFF;
}

pub type BrainID = ID<Brain>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct BrainActivityMap {
    // (point from, point to, receptors)
    pub connections: Vec<(Position, Position, Scalar)>,
    // (point from, point to, factor)
    pub impulses: Vec<(Position, Position, Scalar)>,
    // point
    pub sensors: Vec<Position>,
    // point
    pub effectors: Vec<Position>,
    // point
    pub neurons: Vec<Position>,
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct BrainActivityStats {
    pub neurons_count: usize,
    pub synapses_count: usize,
    pub impulses_count: usize,
    // (current, min, max)
    pub neurons_potential: (Scalar, Scalar, Scalar),
    // (current, min, max)
    pub impulses_potential: (Scalar, Scalar, Scalar),
    // (current, min, max)
    pub all_potential: (Scalar, Scalar, Scalar),
    // (min, max)
    pub incoming_neuron_connections: (usize, usize),
    // (min, max)
    pub outgoing_neuron_connections: (usize, usize),
    // (min, max)
    pub synapses_receptors: (Scalar, Scalar),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Brain {
    id: BrainID,
    neurons: Vec<Neuron>,
    synapses: Vec<Synapse>,
    sensors: Vec<Sensor>,
    effectors: Vec<Effector>,
    config: Config,
    new_connections_accum: Scalar,
}

impl Brain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn duplicate(&self) -> Self {
        let id = Default::default();
        let neuron_indices = self.neurons.par_iter().map(|n| n.id()).collect::<Vec<_>>();
        let neurons = self
            .neurons
            .par_iter()
            .map(|n| Neuron::new(id, n.position()))
            .collect::<Vec<_>>();
        let synapses = self
            .synapses
            .par_iter()
            .map(|s| Synapse {
                source: neurons[neuron_indices
                    .par_iter()
                    .position_any(|n| *n == s.source)
                    .unwrap()]
                .id(),
                target: neurons[neuron_indices
                    .par_iter()
                    .position_any(|n| *n == s.target)
                    .unwrap()]
                .id(),
                distance: s.distance,
                receptors: s.receptors,
                impulses: vec![],
                inactivity: 0.0,
            })
            .collect::<Vec<_>>();
        let sensors = self
            .sensors
            .par_iter()
            .map(|s| Sensor {
                id: s.id,
                target: neurons[neuron_indices
                    .par_iter()
                    .position_any(|n| *n == s.target)
                    .unwrap()]
                .id(),
            })
            .collect::<Vec<_>>();
        let effectors = self
            .effectors
            .par_iter()
            .map(|e| Effector {
                id: e.id,
                source: neurons[neuron_indices
                    .par_iter()
                    .position_any(|n| *n == e.source)
                    .unwrap()]
                .id(),
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
            new_connections_accum: 0.0,
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
            new_connections_accum: 0.0,
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
        self.neurons.par_iter().map(|n| n.id()).collect()
    }

    #[inline]
    pub fn get_sensors(&self) -> Vec<SensorID> {
        self.sensors.par_iter().map(|s| s.id).collect()
    }

    #[inline]
    pub fn get_effectors(&self) -> Vec<EffectorID> {
        self.effectors.par_iter().map(|e| e.id).collect()
    }

    #[inline]
    pub fn synapses_count(&self) -> usize {
        self.synapses.len()
    }

    #[inline]
    pub fn get_impulses_count(&self) -> usize {
        self.synapses.par_iter().map(|s| s.impulses.len()).sum()
    }

    #[inline]
    pub fn get_impulses_potential(&self) -> Scalar {
        self.synapses
            .par_iter()
            .map(|s| s.impulses.par_iter().map(|i| i.potential).sum::<Scalar>())
            .sum::<Scalar>()
    }

    #[inline]
    pub fn get_neurons_potential(&self) -> Scalar {
        self.neurons.par_iter().map(|n| n.potential()).sum()
    }

    #[inline]
    pub fn get_potential(&self) -> Scalar {
        self.get_neurons_potential() + self.get_impulses_potential()
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
        self.neurons.par_iter().find_any(|n| n.id() == id)
    }

    #[inline]
    pub fn neuron_mut(&mut self, id: NeuronID) -> Option<&mut Neuron> {
        self.neurons.par_iter_mut().find_any(|n| n.id() == id)
    }

    #[inline]
    pub fn neurons(&self) -> &[Neuron] {
        &self.neurons
    }

    #[inline]
    pub fn are_neurons_connected(&self, from: NeuronID, to: NeuronID) -> bool {
        self.synapses
            .par_iter()
            .any(|s| s.source == from && s.target == to)
    }

    #[inline]
    pub fn does_neuron_has_connections(&self, id: NeuronID) -> bool {
        self.synapses
            .par_iter()
            .any(|s| s.source == id || s.target == id)
    }

    #[inline]
    pub fn get_neuron_connections_count(&self, id: NeuronID) -> (usize, usize) {
        let incoming = self.synapses.par_iter().filter(|s| s.target == id).count();
        let outgoing = self.synapses.par_iter().filter(|s| s.source == id).count();
        (incoming, outgoing)
    }

    #[inline]
    pub fn get_neuron_connections(&self, id: NeuronID) -> (Vec<NeuronID>, Vec<NeuronID>) {
        let incoming = self
            .synapses
            .par_iter()
            .filter_map(|s| if s.target == id { Some(s.source) } else { None })
            .collect();
        let outgoing = self
            .synapses
            .par_iter()
            .filter_map(|s| if s.source == id { Some(s.target) } else { None })
            .collect();
        (incoming, outgoing)
    }

    pub fn create_sensor(&mut self, target: NeuronID) -> Result<SensorID> {
        if let Some(sensor) = self.sensors.par_iter().find_any(|s| s.target == target) {
            return Err(Error::NeuronIsAlreadyConnectedToSensor(target, sensor.id));
        }
        if let Some(effector) = self.effectors.par_iter().find_any(|e| e.source == target) {
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
        if let Some(index) = self.sensors.par_iter().position_any(|s| s.id == id) {
            self.sensors.swap_remove(index);
            Ok(())
        } else {
            Err(Error::SensorDoesNotExists(id))
        }
    }

    pub fn sensor_trigger_impulse(&mut self, id: SensorID, potential: Scalar) -> Result<()> {
        if let Some(sensor) = self.sensors.par_iter().find_any(|s| s.id == id) {
            if let Some(neuron) = self
                .neurons
                .par_iter_mut()
                .find_any(|n| n.id() == sensor.target)
            {
                neuron.push_potential(potential);
                Ok(())
            } else {
                Err(Error::NeuronDoesNotExists(sensor.target))
            }
        } else {
            Err(Error::SensorDoesNotExists(id))
        }
    }

    pub fn create_effector(&mut self, source: NeuronID) -> Result<EffectorID> {
        if let Some(sensor) = self.sensors.par_iter().find_any(|s| s.target == source) {
            return Err(Error::NeuronIsAlreadyConnectedToSensor(source, sensor.id));
        }
        if let Some(effector) = self.effectors.par_iter().find_any(|e| e.source == source) {
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
        if let Some(index) = self.effectors.par_iter().position_any(|e| e.id == id) {
            self.effectors.swap_remove(index);
            Ok(())
        } else {
            Err(Error::EffectorDoesNotExists(id))
        }
    }

    pub fn effector_potential_release(&mut self, id: EffectorID) -> Result<Scalar> {
        if let Some(effector) = self.effectors.par_iter_mut().find_any(|e| e.id == id) {
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
        if let Some(index) = self.neurons.par_iter().position_any(|n| n.id() == id) {
            self.neurons.swap_remove(index);
            while let Some(index) = self
                .synapses
                .iter()
                .position(|s| s.source == id || s.target == id)
            {
                self.synapses.swap_remove(index);
            }
            while let Some(index) = self.sensors.par_iter().position_any(|s| s.target == id) {
                self.sensors.swap_remove(index);
            }
            while let Some(index) = self.effectors.par_iter().position_any(|e| e.source == id) {
                self.effectors.swap_remove(index);
            }
            Ok(())
        } else {
            Err(Error::NeuronDoesNotExists(id))
        }
    }

    pub fn bind_neurons(&mut self, from: NeuronID, to: NeuronID) -> Result<Option<Scalar>> {
        if from == to {
            return Err(Error::BindingNeuronToItSelf(from));
        }
        if let Some(source) = self.neuron(from) {
            if let Some(target) = self.neuron(to) {
                if self.are_neurons_connected(from, to) {
                    return Ok(None);
                }
                if let Some(sensor) = self.sensors.par_iter().find_any(|s| s.target == to) {
                    return Err(Error::BindingNeuronToSensor(to, sensor.id));
                }
                if let Some(effector) = self.effectors.par_iter().find_any(|e| e.source == from) {
                    return Err(Error::BindingEffectorToNeuron(effector.id, from));
                }
                let distance = source.position().distance(target.position());
                let receptors = thread_rng().gen_range(
                    self.config.default_receptors.0,
                    self.config.default_receptors.1,
                );
                self.synapses.push(Synapse {
                    source: from,
                    target: to,
                    distance,
                    receptors,
                    impulses: vec![],
                    inactivity: 0.0,
                });
                Ok(Some(receptors))
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
        if self.neurons.par_iter().any(|n| n.id() == from) {
            if self.neurons.par_iter().any(|n| n.id() == to) {
                if let Some(index) = self
                    .synapses
                    .par_iter()
                    .position_any(|s| s.source == from && s.target == to)
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
        if self.neurons.is_empty() {
            return Ok(());
        }

        let Config {
            propagation_speed,
            action_potential_treshold,
            synapse_inactivity_time,
            neuron_potential_decay,
            synapse_overdose_receptors,
            receptors_excitation,
            receptors_inhibition,
            synapse_propagation_decay,
            synapse_new_connection_receptors,
            ..
        } = self.config;

        // potential summation phase.
        {
            let dtpd = delta_time * neuron_potential_decay;
            let neurons_triggering = self
                .neurons
                .par_iter_mut()
                .filter_map(|neuron| {
                    let potential = neuron.potential();
                    let status = if potential >= action_potential_treshold {
                        neuron.fire();
                        true
                    } else {
                        false
                    };
                    neuron.process_potential(dtpd);
                    if status {
                        Some((neuron.id(), potential))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            for (id, p) in neurons_triggering {
                let count = self
                    .synapses
                    .par_iter()
                    .filter(|s| s.inactivity <= 0.0 && s.source == id)
                    .count();
                if count > 0 {
                    let p = p / count as Scalar;
                    self.synapses
                        .par_iter_mut()
                        .filter(|s| s.inactivity <= 0.0 && s.source == id)
                        .for_each(|s| {
                            let under = if let Some(o) = synapse_overdose_receptors {
                                s.receptors < o
                            } else {
                                true
                            };
                            if under {
                                s.impulses.push(Impulse {
                                    potential: p,
                                    timeout: s.distance,
                                });
                            }
                            s.inactivity = synapse_inactivity_time;
                        });
                }
            }
        }

        // impulse propagation phase.
        {
            let s = propagation_speed * delta_time;
            let r = receptors_excitation * delta_time;
            let d = synapse_propagation_decay * s;
            let neurons_to_trigger = self
                .synapses
                .par_iter_mut()
                .flat_map(|synapse| {
                    let mut estimated_count = 0;
                    for impulse in &mut synapse.impulses {
                        impulse.potential -= d;
                        impulse.timeout -= s;
                        if impulse.timeout <= 0.0 {
                            estimated_count += 1;
                        }
                    }
                    synapse.receptors += estimated_count as Scalar * r;
                    let mut neurons_to_trigger = Vec::with_capacity(estimated_count);
                    if estimated_count > 0 {
                        synapse.impulses = synapse
                            .impulses
                            .iter()
                            .filter_map(|impulse| {
                                if impulse.potential <= 0.0 {
                                    None
                                } else if impulse.timeout > 0.0 {
                                    Some(*impulse)
                                } else {
                                    neurons_to_trigger.push((synapse.target, impulse.potential));
                                    None
                                }
                            })
                            .collect();
                    }
                    synapse.inactivity = (synapse.inactivity - delta_time).max(0.0);
                    neurons_to_trigger
                })
                .collect::<Vec<_>>();
            self.neurons.par_iter_mut().for_each(|neuron| {
                let nid = neuron.id();
                for (id, potential) in &neurons_to_trigger {
                    if nid == *id {
                        neuron.push_potential(*potential);
                    }
                }
            });
        }

        // inhibition and reconnection phase.
        if receptors_inhibition > 0.0 {
            let r = receptors_inhibition * delta_time;
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
            let neurons_to_reconnect = synapses_to_remove
                .par_iter()
                .filter_map(|index| {
                    let s = &self.synapses[*index];
                    if s.receptors <= 0.0 {
                        if let Some(neuron) =
                            self.neurons.par_iter().find_any(|n| n.id() == s.source)
                        {
                            let mut rng = thread_rng();
                            if let Some(id) = self.select_neuron(neuron.position(), &mut rng) {
                                if s.source != id
                                    && !self.are_neurons_connected(s.source, id)
                                    && !self.are_neurons_connected(id, s.source)
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
                drop(self.bind_neurons(from, to)?);
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
                        .par_iter()
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
                    .par_iter()
                    .position_any(|s| s.source == id || s.target == id)
                {
                    self.synapses.swap_remove(index);
                }
                while let Some(index) = self.sensors.par_iter().position_any(|s| s.target == id) {
                    self.sensors.swap_remove(index);
                }
                while let Some(index) = self.effectors.par_iter().position_any(|e| e.source == id) {
                    self.effectors.swap_remove(index);
                }
            }
        }

        // accumulating effector potentials phase.
        {
            for effector in &mut self.effectors {
                if let Some(neuron) = self
                    .neurons
                    .par_iter()
                    .find_any(|n| n.id() == effector.source)
                {
                    effector.potential = neuron.potential();
                }
            }
        }

        // creating new connections phase.
        if let Some(r) = synapse_new_connection_receptors {
            let synapses_to_connect = self
                .synapses
                .par_iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    if s.receptors > r {
                        if let Some(neuron) = self.neuron(s.source) {
                            let mut rng = thread_rng();
                            if let Some(id) = self.select_neuron(neuron.position(), &mut rng) {
                                if s.source != id
                                    && !self.are_neurons_connected(s.source, id)
                                    && !self.are_neurons_connected(id, s.source)
                                {
                                    return Some((i, s.source, id));
                                }
                            }
                        }
                    }
                    None
                })
                .collect::<Vec<_>>();
            for (index, from, to) in synapses_to_connect.into_iter().rev() {
                if let Some(receptors) = self.bind_neurons(from, to)? {
                    self.synapses[index].receptors -= receptors;
                }
            }
        }

        Ok(())
    }

    fn select_neuron<R>(&self, position: Position, rng: &mut R) -> Option<NeuronID>
    where
        R: Rng,
    {
        let srr = self.config.synapse_reconnection_range;
        if srr.is_none() {
            return if self.neurons.is_empty() {
                None
            } else {
                Some(self.neurons[rng.gen_range(0, self.neurons.len()) % self.neurons.len()].id())
            };
        }
        let filtered = self
            .neurons
            .par_iter()
            .filter_map(|neuron| {
                if neuron.position().distance(position) < srr.unwrap() {
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

    #[inline]
    pub fn build_activity_map_default(&self) -> BrainActivityMap {
        self.build_activity_map(activity::ALL)
    }

    pub fn build_activity_map(&self, flags: usize) -> BrainActivityMap {
        let connections = if flags & activity::CONNECTIONS != 0 {
            self.synapses
                .par_iter()
                .map(|s| {
                    let from = self.neuron(s.source).unwrap().position();
                    let to = self.neuron(s.target).unwrap().position();
                    (from, to, s.receptors)
                })
                .collect()
        } else {
            vec![]
        };
        let impulses = if flags & activity::IMPULSES != 0 {
            self.synapses
                .par_iter()
                .map(|s| {
                    let from = self.neuron(s.source).unwrap().position();
                    let to = self.neuron(s.target).unwrap().position();
                    let distance = from.distance(to);
                    s.impulses
                        .iter()
                        .map(|i| {
                            let factor = if distance > 0.0 {
                                1.0 - i.timeout.max(0.0).min(distance) / distance
                            } else {
                                0.0
                            };
                            (from, to, factor)
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect()
        } else {
            vec![]
        };
        let sensors = if flags & activity::SENSORS != 0 {
            self.sensors
                .par_iter()
                .map(|s| self.neuron(s.target).unwrap().position())
                .collect()
        } else {
            vec![]
        };
        let effectors = if flags & activity::EFFECTORS != 0 {
            self.effectors
                .par_iter()
                .map(|e| self.neuron(e.source).unwrap().position())
                .collect()
        } else {
            vec![]
        };
        let neurons = if flags & activity::NEURONS != 0 {
            self.neurons.par_iter().map(|n| n.position()).collect()
        } else {
            vec![]
        };

        BrainActivityMap {
            connections,
            impulses,
            sensors,
            effectors,
            neurons,
        }
    }

    pub fn build_activity_stats(&self) -> BrainActivityStats {
        let neurons_potential = self.get_neurons_potential();
        let neurons_potential_min = self
            .neurons
            .par_iter()
            .map(|n| n.potential())
            .min_by(|a, b| a.partial_cmp(&b).unwrap())
            .unwrap_or(0.0);
        let neurons_potential_max = self
            .neurons
            .par_iter()
            .map(|n| n.potential())
            .max_by(|a, b| a.partial_cmp(&b).unwrap())
            .unwrap_or(0.0);
        let impulses_potential = self.get_impulses_potential();
        let impulses_potential_min = self
            .synapses
            .par_iter()
            .map(|s| {
                s.impulses
                    .par_iter()
                    .map(|i| i.potential)
                    .min_by(|a, b| a.partial_cmp(&b).unwrap())
                    .unwrap_or(0.0)
            })
            .min_by(|a, b| a.partial_cmp(&b).unwrap())
            .unwrap_or(0.0);
        let impulses_potential_max = self
            .synapses
            .par_iter()
            .map(|s| {
                s.impulses
                    .par_iter()
                    .map(|i| i.potential)
                    .max_by(|a, b| a.partial_cmp(&b).unwrap())
                    .unwrap_or(0.0)
            })
            .max_by(|a, b| a.partial_cmp(&b).unwrap())
            .unwrap_or(0.0);
        let neuron_connections = self
            .neurons
            .par_iter()
            .map(|n| self.get_neuron_connections_count(n.id()))
            .collect::<Vec<_>>();
        let neuron_connections_min = neuron_connections
            .par_iter()
            .cloned()
            .reduce(|| (0, 0), |a, b| (a.0.min(b.0), a.1.min(b.1)));
        let neuron_connections_max = neuron_connections
            .par_iter()
            .cloned()
            .reduce(|| (0, 0), |a, b| (a.0.max(b.0), a.1.max(b.1)));
        let synapses_receptors_min = self
            .synapses
            .par_iter()
            .map(|s| s.receptors)
            .min_by(|a, b| a.partial_cmp(&b).unwrap())
            .unwrap_or(0.0);
        let synapses_receptors_max = self
            .synapses
            .par_iter()
            .map(|s| s.receptors)
            .max_by(|a, b| a.partial_cmp(&b).unwrap())
            .unwrap_or(0.0);

        BrainActivityStats {
            neurons_count: self.neurons.len(),
            synapses_count: self.synapses.len(),
            impulses_count: self.get_impulses_count(),
            neurons_potential: (
                neurons_potential,
                neurons_potential_min,
                neurons_potential_max,
            ),
            impulses_potential: (
                impulses_potential,
                impulses_potential_min,
                impulses_potential_max,
            ),
            all_potential: (
                neurons_potential + impulses_potential,
                neurons_potential_min.min(impulses_potential_min),
                neurons_potential_max.max(impulses_potential_max),
            ),
            incoming_neuron_connections: (neuron_connections_min.0, neuron_connections_max.0),
            outgoing_neuron_connections: (neuron_connections_min.1, neuron_connections_max.1),
            synapses_receptors: (synapses_receptors_min, synapses_receptors_max),
        }
    }

    pub fn ignite_random_synapses(&mut self, count: usize, potential: (Scalar, Scalar)) {
        let mut rng = thread_rng();
        for _ in 0..count {
            let index = rng.gen_range(0, self.synapses.len()) % self.synapses.len();
            let synapse = &mut self.synapses[index];
            synapse.impulses.push(Impulse {
                potential: if potential.1 <= potential.0 {
                    potential.1
                } else {
                    rng.gen_range(potential.0, potential.1)
                },
                timeout: rng.gen_range(0.0, synapse.distance),
            });
        }
    }
}
