use crate::brain::BrainID;
use crate::id::ID;
use crate::Scalar;
use serde::{Deserialize, Serialize};

pub type NeuronID = ID<Neuron>;

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Impulse {
    pub value: Scalar,
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
pub struct Neuron {
    id: NeuronID,
    owner_id: BrainID,
    position: Position,
    input_impulses: Vec<Impulse>,
    accumulated_impulse: Scalar,
}

impl Neuron {
    pub(crate) fn new(owner_id: BrainID, position: Position) -> Self {
        Self {
            id: Default::default(),
            owner_id,
            position,
            input_impulses: vec![],
            accumulated_impulse: 0.0,
        }
    }

    pub(crate) fn with_id(id: NeuronID, owner_id: BrainID, position: Position) -> Self {
        Self {
            id,
            owner_id,
            position,
            input_impulses: vec![],
            accumulated_impulse: 0.0,
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
    pub fn is_active(&self) -> bool {
        !self.input_impulses.is_empty()
    }

    #[inline]
    pub fn accumulated_impulse(&self) -> Scalar {
        self.accumulated_impulse
    }

    #[inline]
    pub fn input_impulses(&self) -> &[Impulse] {
        &self.input_impulses
    }

    #[inline]
    pub(crate) fn push_impulse(&mut self, impulse: Impulse) {
        self.input_impulses.push(impulse);
    }

    #[inline]
    pub(crate) fn process_input_impulses(&mut self, delta_time: Scalar, propagation_speed: Scalar) {
        let s = delta_time * propagation_speed;
        self.input_impulses = self
            .input_impulses
            .iter()
            .filter_map(|impulse| {
                let mut impulse = impulse.clone();
                impulse.timeout -= s;
                if impulse.timeout > 0.0 {
                    Some(impulse)
                } else {
                    None
                }
            })
            .collect();
    }

    #[inline]
    pub(crate) fn set_accumulated_impulse(&mut self, value: Scalar) {
        self.accumulated_impulse = value;
    }

    #[inline]
    pub(crate) fn clear_input_impulses(&mut self) {
        self.input_impulses.clear();
    }
}
