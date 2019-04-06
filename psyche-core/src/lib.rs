extern crate rand;
#[cfg(feature = "parallel")]
extern crate rayon;
extern crate serde;
extern crate uuid;

#[cfg(test)]
pub mod tests;

pub mod brain;
pub mod brain_builder;
pub mod config;
pub mod effector;
pub mod error;
pub mod id;
pub mod neuron;
pub mod offspring_builder;
pub mod sensor;

pub type Scalar = f64;
