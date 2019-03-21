pub mod spore;

use crate::managers::brains_manager::BrainsManager;
use crate::managers::food_manager::food::Food;
use crate::managers::food_manager::FoodManager;
use crate::managers::items_manager::{ItemsManager, Named};
use crate::managers::physics_manager::body::BodyID;
use crate::managers::physics_manager::PhysicsManager;
use crate::managers::renderables_manager::renderable::{angle, Graphics};
use crate::managers::renderables_manager::RenderablesManager;
use core::f64::consts::PI;
use spore::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct SporesManager {
    spores: Vec<Spore>,
}

impl SporesManager {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn item_by_body_mut(&mut self, id: BodyID) -> Option<&mut Spore> {
        self.spores.iter_mut().find(|s| {
            if let Some(s) = s.inner() {
                s.body == id
            } else {
                false
            }
        })
    }

    pub fn refresh(
        &self,
        physics: &PhysicsManager,
        renderables: &mut RenderablesManager,
        brains: &BrainsManager,
    ) {
        for spore in &self.spores {
            if let Some(inner) = spore.inner() {
                let renderable =
                    if let Some(renderable) = renderables.item_mut(inner.renderable_body) {
                        renderable
                    } else {
                        continue;
                    };
                let body = if let Some(body) = physics.item(inner.body) {
                    body
                } else {
                    continue;
                };
                let brain = if let Some(brain) = brains.item(inner.brain) {
                    brain
                } else {
                    continue;
                };
                let state = body.cached_state();
                renderable.transform.position = [state.position.x, state.position.y].into();
                renderable.transform.angle = angle(state.rotation);
                if let Graphics::Circle(ref mut color, ref mut r) = renderable.graphics {
                    let f = (brain.get_potential() as f32 * 0.1).max(0.0).min(1.0);
                    *color = [f, f * 0.5, f * 0.5, 0.25];
                    *r = state.radius;
                }
                for state in inner.legs.values() {
                    if let Some(renderable) = renderables.item_mut(state.renderable) {
                        let factor = match state.phase {
                            1 => 1.0,
                            3 => -1.0,
                            _ => 0.0,
                        };
                        renderable.transform.angle = angle(state.angle + PI * 0.2 * factor);
                    }
                }
                for state in inner.detectors.values() {
                    if let Some(renderable) = renderables.item_mut(state.renderable) {
                        if let Graphics::Rectangle(ref mut color, _) = renderable.graphics {
                            let f = (state.potential as f32 * 1.0).max(0.0).min(1.0);
                            *color = [f, f, 0.0, 0.25];
                        }
                    }
                }
            }
        }
    }

    pub fn process(
        &mut self,
        brains: &mut BrainsManager,
        physics: &mut PhysicsManager,
        foods: &mut FoodManager,
        renderables: &mut RenderablesManager,
    ) {
        for spore in &mut self.spores {
            spore.process(brains, physics, foods);
        }

        let food_to_destroy = physics
            .cache_bodies_triggered()
            .iter()
            .filter_map(|trigger| {
                let (food_id, calories) = if let Some(food) = foods.item_by_body(trigger.sensor) {
                    (food.id(), food.calories())
                } else {
                    return None;
                };
                if let Some(spore) = self.item_by_body_mut(trigger.body) {
                    spore.feed(calories);
                    Some(food_id)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        for id in food_to_destroy {
            foods.item_mut(id).unwrap().annihilate(physics, renderables);
            foods.destroy(id);
        }

        // TODO: produce offspring if compatible DNA or eat smaller spore.
        // for contact in physics.cache_bodies_contacted() {}

        let spores_to_destroy = self
            .spores
            .iter()
            .filter_map(|spore| {
                if spore.calories() <= 0.0 {
                    Some(spore.id())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for id in spores_to_destroy {
            if let Some(spore) = self.item_mut(id) {
                if let Some(inner) = spore.inner() {
                    if let Some(body) = physics.item(inner.body) {
                        let state = body.cached_state();
                        let position = state.position;
                        let radius = state.radius;
                        let calories = radius * radius * PI;
                        let mut food = Food::default();
                        food.born(calories, position.into(), physics, renderables);
                        foods.add(food);
                    }
                }
                spore.annihilate(physics, renderables, brains);
            }
            self.destroy(id);
        }
    }
}

impl ItemsManager<Spore> for SporesManager {
    #[inline]
    fn items(&self) -> &[Spore] {
        &self.spores
    }

    fn add(&mut self, item: Spore) -> SporeID {
        let id = item.id();
        self.spores.push(item);
        id
    }

    fn create(&mut self) -> SporeID {
        self.add(Spore::new())
    }

    fn create_with<F>(&mut self, mut with: F) -> SporeID
    where
        F: FnMut(&mut Spore, &mut Self),
    {
        let mut spore = Spore::new();
        with(&mut spore, self);
        self.add(spore)
    }

    /// WARNING: Consider using `World::annihilate_spore()`
    fn destroy(&mut self, id: SporeID) -> bool {
        if let Some(index) = self.spores.iter().position(|r| r.id() == id) {
            self.spores.swap_remove(index);
            true
        } else {
            false
        }
    }

    fn with<F, R>(&mut self, id: SporeID, mut with: F) -> Option<R>
    where
        F: FnMut(&mut Spore, &mut Self) -> R,
    {
        if let Some(index) = self.spores.iter().position(|r| r.id() == id) {
            let mut spore = self.spores.swap_remove(index);
            let result = with(&mut spore, self);
            self.spores.push(spore);
            Some(result)
        } else {
            None
        }
    }

    #[inline]
    fn item(&self, id: SporeID) -> Option<&Spore> {
        self.spores.iter().find(|r| r.id() == id)
    }

    #[inline]
    fn item_mut(&mut self, id: SporeID) -> Option<&mut Spore> {
        self.spores.iter_mut().find(|r| r.id() == id)
    }
}
