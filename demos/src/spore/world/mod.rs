#![allow(dead_code)]

pub mod world_builder;

use crate::managers::brains_manager::BrainsManager;
use crate::managers::food_manager::food::{Food, FoodID};
use crate::managers::food_manager::FoodManager;
use crate::managers::items_manager::ItemsManager;
use crate::managers::physics_manager::PhysicsManager;
use crate::managers::renderables_manager::RenderablesManager;
use crate::managers::spores_manager::spore::{Spore, SporeID};
use crate::managers::spores_manager::SporesManager;
use psyche::core::brain_builder::BrainBuilder;
use psyche::core::Scalar;
use rand::{thread_rng, Rng};
use std::f64::consts::PI;
use std::ops::Range;

#[derive(Debug)]
pub struct World {
    size: (Scalar, Scalar),
    renderables: RenderablesManager,
    spores: SporesManager,
    physics: PhysicsManager,
    brains: BrainsManager,
    food: FoodManager,
}

impl World {
    pub fn new(
        size: (Scalar, Scalar),
        grid_cols_rows: (usize, usize),
        randomized_fluid: Scalar,
        fluid_diffuse: Scalar,
        fluid_drag: Scalar,
    ) -> Self {
        Self {
            size,
            renderables: RenderablesManager::new(),
            spores: SporesManager::new(),
            physics: PhysicsManager::new(
                size,
                grid_cols_rows,
                randomized_fluid,
                fluid_diffuse,
                fluid_drag,
            ),
            brains: BrainsManager::new(),
            food: FoodManager::new(),
        }
    }

    pub fn size(&self) -> (Scalar, Scalar) {
        self.size
    }

    pub fn renderables(&self) -> &RenderablesManager {
        &self.renderables
    }

    pub fn renderables_mut(&mut self) -> &mut RenderablesManager {
        &mut self.renderables
    }

    pub fn spores(&self) -> &SporesManager {
        &self.spores
    }

    pub fn spores_mut(&mut self) -> &mut SporesManager {
        &mut self.spores
    }

    pub fn physics(&self) -> &PhysicsManager {
        &self.physics
    }

    pub fn physics_mut(&mut self) -> &mut PhysicsManager {
        &mut self.physics
    }

    pub fn brains(&self) -> &BrainsManager {
        &self.brains
    }

    pub fn brains_mut(&mut self) -> &mut BrainsManager {
        &mut self.brains
    }

    pub fn food(&self) -> &FoodManager {
        &self.food
    }

    pub fn food_mut(&mut self) -> &mut FoodManager {
        &mut self.food
    }

    pub fn process(&mut self, dt: Scalar) {
        self.physics.process(dt);
        self.brains.process(dt);
        self.spores.process(
            &mut self.brains,
            &mut self.physics,
            &mut self.food,
            &mut self.renderables,
        );

        self.spores
            .refresh(&self.physics, &mut self.renderables, &self.brains);
        self.food.refresh(&self.physics, &mut self.renderables);
    }

    pub fn born_spore(&mut self, builder: &BrainBuilder, radius: Range<Scalar>) -> SporeID {
        let mut spore = Spore::default();
        let mut rng = thread_rng();
        let radius = if radius.end > radius.start {
            rng.gen_range(radius.start, radius.end)
        } else {
            radius.end
        };
        let pos = [
            rng.gen_range(radius, self.size.0 - radius),
            rng.gen_range(radius, self.size.1 - radius),
        ];
        let rot = rng.gen_range(0.0, PI * 2.0);
        spore.born(
            (pos, rot, radius),
            builder,
            &mut self.physics,
            &mut self.renderables,
            &mut self.brains,
        );
        self.spores.add(spore)
    }

    pub fn annihilate_spore(&mut self, id: SporeID) {
        if let Some(spore) = self.spores.item_mut(id) {
            spore.annihilate(&mut self.physics, &mut self.renderables, &mut self.brains);
            self.spores.destroy(id);
        }
    }

    pub fn born_food(&mut self, calories: Range<Scalar>) -> FoodID {
        let mut food = Food::default();
        let mut rng = thread_rng();
        let calories = if calories.end > calories.start {
            rng.gen_range(calories.start, calories.end)
        } else {
            calories.end
        };
        let pos = [
            rng.gen_range(0.0, self.size.0),
            rng.gen_range(0.0, self.size.1),
        ];
        food.born(calories, pos, &mut self.physics, &mut self.renderables);
        self.food.add(food)
    }

    pub fn annihilate_food(&mut self, id: FoodID) {
        if let Some(food) = self.food.item_mut(id) {
            food.annihilate(&mut self.physics, &mut self.renderables);
            self.food.destroy(id);
        }
    }
}
