use crate::world::World;
use psyche::core::brain_builder::BrainBuilder;
use psyche::core::Scalar;
use std::ops::Range;

#[derive(Debug)]
pub struct WorldBuilder {
    size: (Scalar, Scalar),
    grid_cols_rows: (usize, usize),
    randomized_fluid: Scalar,
    fluid_diffuse: Scalar,
    fluid_drag: Scalar,
    spores_count: usize,
    spores_brain_builder: BrainBuilder,
    spores_radius: Range<Scalar>,
    food_count: usize,
    food_calories: Range<Scalar>,
}

impl Default for WorldBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            size: (100.0, 100.0),
            grid_cols_rows: (10, 10),
            randomized_fluid: 0.0,
            fluid_diffuse: 0.0,
            fluid_drag: 0.0,
            spores_count: 1,
            spores_brain_builder: BrainBuilder::default(),
            spores_radius: 5.0..10.0,
            food_count: 1,
            food_calories: 10.0..100.0,
        }
    }
}

impl WorldBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn size(mut self, value: (Scalar, Scalar)) -> Self {
        self.size = value;
        self
    }

    #[inline]
    pub fn grid_cols_rows(mut self, value: (usize, usize)) -> Self {
        self.grid_cols_rows = value;
        self
    }

    #[inline]
    pub fn randomized_fluid(mut self, value: Scalar) -> Self {
        self.randomized_fluid = value;
        self
    }

    #[inline]
    pub fn fluid_diffuse(mut self, value: Scalar) -> Self {
        self.fluid_diffuse = value;
        self
    }

    #[inline]
    pub fn fluid_drag(mut self, value: Scalar) -> Self {
        self.fluid_drag = value;
        self
    }

    #[inline]
    pub fn spores_count(mut self, value: usize) -> Self {
        self.spores_count = value;
        self
    }

    #[inline]
    pub fn spores_brain_builder(mut self, value: BrainBuilder) -> Self {
        self.spores_brain_builder = value;
        self
    }

    #[inline]
    pub fn spores_radius(mut self, value: Range<Scalar>) -> Self {
        self.spores_radius = value;
        self
    }

    #[inline]
    pub fn food_count(mut self, value: usize) -> Self {
        self.food_count = value;
        self
    }

    #[inline]
    pub fn food_calories(mut self, value: Range<Scalar>) -> Self {
        self.food_calories = value;
        self
    }

    pub fn build_and_setup<F>(self, mut setup: F) -> World
    where
        F: FnMut(&mut World),
    {
        let mut world = World::new(
            self.size,
            self.grid_cols_rows,
            self.randomized_fluid,
            self.fluid_diffuse,
            self.fluid_drag,
        );
        setup(&mut world);
        for _ in 0..self.food_count {
            world.born_food(self.food_calories.clone());
        }
        for _ in 0..self.spores_count {
            world.born_spore(&self.spores_brain_builder, self.spores_radius.clone());
        }
        world
    }

    pub fn build(self) -> World {
        self.build_and_setup(|_| {})
    }
}
