use crate::brain::BrainID;
use crate::id::ID;
use crate::Scalar;
use serde::{Deserialize, Serialize};

pub type NeuronID = ID<Neuron>;

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Impulse {
    pub potential: Scalar,
    pub timeout: Scalar,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Synapse {
    pub source: NeuronID,
    pub target: NeuronID,
    pub distance: Scalar,
    pub receptors: Scalar,
    pub impulses: Vec<Impulse>,
    pub inactivity: Scalar,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Position {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl Position {
    #[inline]
    pub fn magnitude_sqr(&self) -> Scalar {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn magnitude(&self) -> Scalar {
        self.magnitude_sqr().sqrt()
    }

    pub fn distance_sqr(&self, other: Self) -> Scalar {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    #[inline]
    pub fn distance(&self, other: Self) -> Scalar {
        self.distance_sqr(other).sqrt()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Neuron {
    id: NeuronID,
    owner_id: BrainID,
    position: Position,
    potential: Scalar,
}

impl Neuron {
    pub(crate) fn new(owner_id: BrainID, position: Position) -> Self {
        Self {
            id: Default::default(),
            owner_id,
            position,
            potential: 0.0,
        }
    }

    pub(crate) fn with_id(id: NeuronID, owner_id: BrainID, position: Position) -> Self {
        Self {
            id,
            owner_id,
            position,
            potential: 0.0,
        }
    }

    #[inline]
    pub fn id(&self) -> NeuronID {
        self.id
    }

    #[inline]
    pub fn owner_id(&self) -> BrainID {
        self.owner_id
    }

    #[inline]
    pub fn position(&self) -> Position {
        self.position
    }

    #[inline]
    pub fn potential(&self) -> Scalar {
        self.potential
    }

    #[inline]
    pub(crate) fn push_potential(&mut self, value: Scalar) {
        self.potential += value;
    }

    #[inline]
    pub(crate) fn process_potential(&mut self, delta_time_times_decay: Scalar) {
        if self.potential < -delta_time_times_decay {
            self.potential = (self.potential + delta_time_times_decay).min(0.0);
        } else if self.potential > delta_time_times_decay {
            self.potential = (self.potential - delta_time_times_decay).max(0.0);
        } else {
            self.potential = 0.0;
        }
    }

    #[inline]
    pub(crate) fn fire(&mut self) {
        self.potential = 0.0;
    }
}
