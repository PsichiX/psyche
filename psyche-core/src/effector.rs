use crate::id::ID;
use crate::neuron::NeuronID;
use crate::Scalar;
use serde::{Deserialize, Serialize};

pub type EffectorID = ID<Effector>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effector {
    pub(crate) id: EffectorID,
    pub(crate) source: NeuronID,
    pub(crate) potential: Scalar,
}
