#![allow(dead_code)]

use crate::managers::items_manager::{ItemsManager, Named};
use crate::managers::physics_manager::body::{Body, BodyID};
use crate::managers::physics_manager::PhysicsManager;
use crate::managers::renderables_manager::renderable::{Graphics, RenderableID};
use crate::managers::renderables_manager::RenderablesManager;
use psyche::core::id::ID;
use psyche::core::Scalar;
use std::f64::consts::PI;

pub type FoodID = ID<Food>;

#[derive(Debug, Clone, Default)]
pub struct FoodInner {
    pub body: BodyID,
    pub renderable: RenderableID,
}

#[derive(Debug, Clone, Default)]
pub struct Food {
    id: FoodID,
    calories: Scalar,
    inner: Option<FoodInner>,
}

impl Named<Self> for Food {
    #[inline]
    fn id(&self) -> FoodID {
        self.id
    }
}

impl Food {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn calories(&self) -> Scalar {
        self.calories
    }

    #[inline]
    pub fn inner(&self) -> Option<&FoodInner> {
        if let Some(ref inner) = self.inner {
            Some(inner)
        } else {
            None
        }
    }

    pub fn born(
        &mut self,
        calories: Scalar,
        position: [Scalar; 2],
        physics: &mut PhysicsManager,
        renderables: &mut RenderablesManager,
    ) {
        if self.inner.is_some() {
            return;
        }

        let radius = (calories / PI).sqrt();
        let body = Body::new(physics, true);
        body.setup(physics, Some(position.into()), Some(0.0), Some(radius));
        let body = physics.add(body);
        let renderable = renderables.create_with(|renderable, _| {
            renderable.transform.position = position.into();
            renderable.graphics = Graphics::Circle([0.0, 1.0, 0.0, 0.5], radius);
        });
        if let Some(food) = renderables.hierarchy_mut("food") {
            food.children.push(renderable.into());
        }

        self.calories = calories;
        self.inner = Some(FoodInner { body, renderable });
    }

    pub fn annihilate(
        &mut self,
        physics: &mut PhysicsManager,
        renderables: &mut RenderablesManager,
    ) {
        if let Some(ref inner) = self.inner {
            physics.destroy(inner.body);
            renderables.destroy(inner.renderable);
            self.inner = None;
        }
    }
}
