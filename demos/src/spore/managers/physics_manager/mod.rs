#![allow(dead_code)]

pub mod body;

use crate::managers::items_manager::{ItemsManager, Named};
use body::*;
use ncollide2d::events::ContactEvent;
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::object::{BodyStatus, ColliderDesc, RigidBodyDesc};
use nphysics2d::world::World as PhysicsWorld;
use psyche::core::Scalar;
use std::fmt;

#[derive(Debug, Default, Copy, Clone)]
pub struct TriggeredBodiesPair {
    pub body: BodyID,
    pub sensor: BodyID,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ContactedBodiesPair {
    pub body1: BodyID,
    pub body2: BodyID,
}

pub struct PhysicsManager {
    bodies: Vec<Body>,
    world: PhysicsWorld<Scalar>,
    cache_bodies_triggered: Vec<TriggeredBodiesPair>,
    cache_bodies_contacted: Vec<ContactedBodiesPair>,
}

impl Default for PhysicsManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl PhysicsManager {
    #[inline]
    pub fn new(bounds: Option<(Scalar, Scalar)>) -> Self {
        let mut world = PhysicsWorld::default();
        world.set_gravity([0.0, 0.0].into());

        if let Some(bounds) = bounds {
            let hw = bounds.0 * 0.5;
            let hh = bounds.1 * 0.5;
            let part = RigidBodyDesc::new()
                .status(BodyStatus::Static)
                .build(&mut world)
                .part_handle();
            {
                let shape = ShapeHandle::new(Cuboid::new([hw, 50.0].into()));
                ColliderDesc::new(shape.clone())
                    .density(1.0)
                    .translation([hw, -50.0].into())
                    .build_with_parent(part, &mut world)
                    .unwrap();
                ColliderDesc::new(shape)
                    .density(1.0)
                    .translation([hw, bounds.1 + 50.0].into())
                    .build_with_parent(part, &mut world)
                    .unwrap();
            }
            {
                let shape = ShapeHandle::new(Cuboid::new([50.0, hh].into()));
                ColliderDesc::new(shape.clone())
                    .density(1.0)
                    .translation([-50.0, hh].into())
                    .build_with_parent(part, &mut world)
                    .unwrap();
                ColliderDesc::new(shape)
                    .density(1.0)
                    .translation([bounds.0 + 50.0, hh].into())
                    .build_with_parent(part, &mut world)
                    .unwrap();
            }
        }

        Self {
            bodies: vec![],
            world,
            cache_bodies_triggered: vec![],
            cache_bodies_contacted: vec![],
        }
    }

    #[inline]
    pub fn world(&self) -> &PhysicsWorld<Scalar> {
        &self.world
    }

    #[inline]
    pub fn world_mut(&mut self) -> &mut PhysicsWorld<Scalar> {
        &mut self.world
    }

    #[inline]
    pub fn cache_bodies_triggered(&self) -> &[TriggeredBodiesPair] {
        &self.cache_bodies_triggered
    }

    #[inline]
    pub fn cache_bodies_contacted(&self) -> &[ContactedBodiesPair] {
        &self.cache_bodies_contacted
    }

    pub fn process(&mut self, dt: Scalar) {
        if (self.world.timestep() - dt).abs() < 0.01 {
            self.world.set_timestep(dt);
        }
        self.world.step();

        self.cache_bodies_triggered = self
            .world
            .proximity_events()
            .iter()
            .filter_map(|proximity| {
                if proximity.new_status == Proximity::Intersecting {
                    if let Some(a) = self
                        .bodies
                        .iter()
                        .find(|a| a.collider_handle() == proximity.collider1)
                    {
                        if let Some(b) = self
                            .bodies
                            .iter()
                            .find(|b| b.collider_handle() == proximity.collider2)
                        {
                            let sensor_a = a.is_sensor(self).unwrap();
                            let sensor_b = b.is_sensor(self).unwrap();
                            match (sensor_a, sensor_b) {
                                (false, true) => {
                                    return Some(TriggeredBodiesPair {
                                        body: a.id(),
                                        sensor: b.id(),
                                    });
                                }
                                (true, false) => {
                                    return Some(TriggeredBodiesPair {
                                        body: b.id(),
                                        sensor: a.id(),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }
                None
            })
            .collect();

        self.cache_bodies_contacted = self
            .world
            .contact_events()
            .iter()
            .filter_map(|contact| {
                if let ContactEvent::Started(ca, cb) = contact {
                    if let Some(a) = self.bodies.iter().find(|a| a.collider_handle() == *ca) {
                        if let Some(b) = self.bodies.iter().find(|b| b.collider_handle() == *cb) {
                            return Some(ContactedBodiesPair {
                                body1: a.id(),
                                body2: b.id(),
                            });
                        }
                    }
                }
                None
            })
            .collect();
    }
}

impl ItemsManager<Body> for PhysicsManager {
    #[inline]
    fn items(&self) -> &[Body] {
        &self.bodies
    }

    fn add(&mut self, item: Body) -> BodyID {
        let id = item.id();
        self.bodies.push(item);
        id
    }

    fn create(&mut self) -> BodyID {
        let body = Body::new(self, false);
        self.add(body)
    }

    fn create_with<F>(&mut self, mut with: F) -> BodyID
    where
        F: FnMut(&mut Body, &mut Self),
    {
        let mut body = Body::new(self, false);
        with(&mut body, self);
        self.add(body)
    }

    fn destroy(&mut self, id: BodyID) -> bool {
        if let Some(index) = self.bodies.iter().position(|r| r.id() == id) {
            let body = self.bodies.swap_remove(index);
            body.free(self);
            true
        } else {
            false
        }
    }

    fn with<F, R>(&mut self, id: BodyID, mut with: F) -> Option<R>
    where
        F: FnMut(&mut Body, &mut Self) -> R,
    {
        if let Some(index) = self.bodies.iter().position(|r| r.id() == id) {
            let mut body = self.bodies.swap_remove(index);
            let result = with(&mut body, self);
            self.bodies.push(body);
            Some(result)
        } else {
            None
        }
    }

    #[inline]
    fn item(&self, id: BodyID) -> Option<&Body> {
        self.bodies.iter().find(|r| r.id() == id)
    }

    #[inline]
    fn item_mut(&mut self, id: BodyID) -> Option<&mut Body> {
        self.bodies.iter_mut().find(|r| r.id() == id)
    }
}

impl fmt::Debug for PhysicsManager {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysicsManager")
            .field("bodies", &self.bodies)
            .field("world", &"[...]".to_owned())
            .finish()
    }
}
