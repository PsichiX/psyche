use crate::Scalar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Config {
    pub propagation_speed: Scalar,
    pub neuron_potential_decay: Scalar,
    pub action_potential_treshold: Scalar,
    pub receptors_excitation: Scalar,
    pub receptors_inhibition: Scalar,
    pub default_receptors: (Scalar, Scalar),
    pub synapse_inactivity_time: Scalar,
    pub synapse_reconnection_range: Option<Scalar>,
    pub synapse_overdose_receptors: Option<Scalar>,
    pub synapse_propagation_decay: Scalar,
    pub synapse_new_connection_receptors: Option<Scalar>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            propagation_speed: 1.0,
            neuron_potential_decay: 1.0,
            action_potential_treshold: 1.0,
            receptors_excitation: 1.0,
            receptors_inhibition: 0.05,
            default_receptors: (0.5, 1.5),
            synapse_inactivity_time: 0.05,
            synapse_reconnection_range: None,
            synapse_overdose_receptors: None,
            synapse_propagation_decay: 0.0,
            synapse_new_connection_receptors: None,
        }
    }
}

impl Config {
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            propagation_speed: merge_scalar(self.propagation_speed, other.propagation_speed),
            neuron_potential_decay: merge_scalar(
                self.neuron_potential_decay,
                other.neuron_potential_decay,
            ),
            action_potential_treshold: merge_scalar(
                self.action_potential_treshold,
                other.action_potential_treshold,
            ),
            receptors_excitation: merge_scalar(
                self.receptors_excitation,
                other.receptors_excitation,
            ),
            receptors_inhibition: merge_scalar(
                self.receptors_inhibition,
                other.receptors_inhibition,
            ),
            default_receptors: (
                merge_scalar(self.default_receptors.0, other.default_receptors.0),
                merge_scalar(self.default_receptors.1, other.default_receptors.1),
            ),
            synapse_inactivity_time: merge_scalar(
                self.synapse_inactivity_time,
                other.synapse_inactivity_time,
            ),
            synapse_reconnection_range: match (
                self.synapse_reconnection_range,
                other.synapse_reconnection_range,
            ) {
                (Some(a), Some(b)) => Some(merge_scalar(a, b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                _ => None,
            },
            synapse_overdose_receptors: match (
                self.synapse_overdose_receptors,
                other.synapse_overdose_receptors,
            ) {
                (Some(a), Some(b)) => Some(merge_scalar(a, b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                _ => None,
            },
            synapse_propagation_decay: merge_scalar(
                self.synapse_propagation_decay,
                other.synapse_propagation_decay,
            ),
            synapse_new_connection_receptors: match (
                self.synapse_new_connection_receptors,
                other.synapse_new_connection_receptors,
            ) {
                (Some(a), Some(b)) => Some(merge_scalar(a, b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                _ => None,
            },
        }
    }
}

fn merge_scalar(a: Scalar, b: Scalar) -> Scalar {
    (a + b) * 0.5
}
