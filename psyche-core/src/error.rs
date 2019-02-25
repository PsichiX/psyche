use crate::effector::EffectorID;
use crate::neuron::NeuronID;
use crate::sensor::SensorID;
use std::io::Error as IoError;
use std::result::Result as StdResult;

#[derive(Debug)]
pub enum Error {
    Simple(SimpleError),
    NeuronDoesNotExists(NeuronID),
    BindingNeuronToItSelf(NeuronID),
    UnbindingNeuronFromItSelf(NeuronID),
    SensorDoesNotExists(SensorID),
    EffectorDoesNotExists(EffectorID),
    BindingNeuronToSensor(NeuronID, SensorID),
    BindingEffectorToNeuron(EffectorID, NeuronID),
    NeuronIsAlreadyConnectedToSensor(NeuronID, SensorID),
    NeuronIsAlreadyConnectedToEffector(NeuronID, EffectorID),
}

impl Error {
    #[inline]
    pub fn simple(message: String) -> Self {
        Error::Simple(SimpleError { message })
    }
}

#[derive(Debug)]
pub struct SimpleError {
    pub message: String,
}

pub type Result<T> = StdResult<T, Error>;

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::simple(format!("{}", error))
    }
}
