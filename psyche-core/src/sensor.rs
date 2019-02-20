use crate::id::ID;
use crate::neuron::NeuronID;
use serde::{Deserialize, Serialize};

pub type SensorID = ID<Sensor>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sensor {
    pub(crate) id: SensorID,
    pub(crate) target: NeuronID,
}
