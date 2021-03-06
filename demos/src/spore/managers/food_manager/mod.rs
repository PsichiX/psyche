pub mod food;

use crate::managers::items_manager::{ItemsManager, Named};
use crate::managers::physics_manager::body::BodyID;
use crate::managers::physics_manager::PhysicsManager;
use crate::managers::renderables_manager::RenderablesManager;
use food::*;

#[derive(Debug, Clone, Default)]
pub struct FoodManager {
    food: Vec<Food>,
}

impl FoodManager {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn item_by_body(&self, id: BodyID) -> Option<&Food> {
        self.food.iter().find(|f| {
            if let Some(f) = f.inner() {
                f.body == id
            } else {
                false
            }
        })
    }

    pub fn refresh(&self, physics: &PhysicsManager, renderables: &mut RenderablesManager) {
        for food in &self.food {
            if let Some(inner) = food.inner() {
                if let Some(renderable) = renderables.item_mut(inner.renderable) {
                    if let Some(body) = physics.item(inner.body) {
                        let position = body.cached_state().position;
                        renderable.transform.position = [position.x, position.y].into();
                    }
                }
            }
        }
    }
}

impl ItemsManager<Food> for FoodManager {
    #[inline]
    fn items(&self) -> &[Food] {
        &self.food
    }

    fn add(&mut self, item: Food) -> FoodID {
        let id = item.id();
        self.food.push(item);
        id
    }

    fn create(&mut self) -> FoodID {
        self.add(Food::new())
    }

    fn create_with<F>(&mut self, mut with: F) -> FoodID
    where
        F: FnMut(&mut Food, &mut Self),
    {
        let mut food = Food::new();
        with(&mut food, self);
        self.add(food)
    }

    /// WARNING: Consider using `World::annihilate_food()`
    fn destroy(&mut self, id: FoodID) -> bool {
        if let Some(index) = self.food.iter().position(|r| r.id() == id) {
            self.food.swap_remove(index);
            true
        } else {
            false
        }
    }

    fn with<F, R>(&mut self, id: FoodID, mut with: F) -> Option<R>
    where
        F: FnMut(&mut Food, &mut Self) -> R,
    {
        if let Some(index) = self.food.iter().position(|r| r.id() == id) {
            let mut food = self.food.swap_remove(index);
            let result = with(&mut food, self);
            self.food.push(food);
            Some(result)
        } else {
            None
        }
    }

    #[inline]
    fn item(&self, id: FoodID) -> Option<&Food> {
        self.food.iter().find(|r| r.id() == id)
    }

    #[inline]
    fn item_mut(&mut self, id: FoodID) -> Option<&mut Food> {
        self.food.iter_mut().find(|r| r.id() == id)
    }
}
