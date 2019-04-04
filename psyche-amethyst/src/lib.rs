extern crate amethyst;
extern crate psyche;

use amethyst::{
    core::{bundle::SystemBundle, timing::Time, Error},
    ecs::{
        prelude::DispatcherBuilder, Component, DenseVecStorage, Join, Read, System, WriteStorage,
    },
};
use psyche::core::{brain::Brain, brain_builder::BrainBuilder};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BrainComponent {
    pub brain: Brain,
}

impl Component for BrainComponent {
    type Storage = DenseVecStorage<Self>;
}

impl BrainComponent {
    pub fn new(brain: Brain) -> Self {
        Self { brain }
    }

    pub fn with_builder(builder: BrainBuilder) -> Self {
        Self {
            brain: builder.build(),
        }
    }
}

#[derive(Default)]
pub struct BrainSystem;

impl<'s> System<'s> for BrainSystem {
    type SystemData = (WriteStorage<'s, BrainComponent>, Read<'s, Time>);

    fn run(&mut self, (mut brains, time): Self::SystemData) {
        let dt = time.delta_seconds() as f64;
        for brain in (&mut brains).join() {
            if let Err(e) = brain.brain.process(dt) {
                println!("Psyche Brain error: {:#?}", e);
            }
        }
    }
}

pub struct BrainBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for BrainBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(BrainSystem, "brain_system", &[]);
        Ok(())
    }
}
