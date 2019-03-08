#![allow(dead_code)]

pub mod body;

use crate::managers::items_manager::{ItemsManager, Named};
use body::*;
use ncollide2d::events::ContactEvent;
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::algebra::{Force2, ForceType};
use nphysics2d::object::{Body as PhysicsBody, BodyStatus, ColliderDesc, RigidBodyDesc};
use nphysics2d::world::World as PhysicsWorld;
use psyche::core::Scalar;
use psyche::utils::grid::{Grid, GridSampleZeroValue, GridSamplerCluster, GridSamplerDistance};
use psyche::utils::switch::Switch;
use rand::{thread_rng, Rng};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul};

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
    bounds: (Scalar, Scalar),
    bodies: Vec<Body>,
    world: PhysicsWorld<Scalar>,
    cache_bodies_triggered: Vec<TriggeredBodiesPair>,
    cache_bodies_contacted: Vec<ContactedBodiesPair>,
    fluid_grid: Switch<Grid<GridCell>>,
    fluid_diffuse: Scalar,
    fluid_drag: Scalar,
    cache_fluid_forces: Vec<(Vec2, Vec2)>,
}

impl PhysicsManager {
    #[inline]
    pub fn new(
        bounds: (Scalar, Scalar),
        grid_cols_rows: (usize, usize),
        randomized_fluid: Scalar,
        fluid_diffuse: Scalar,
        fluid_drag: Scalar,
    ) -> Self {
        let mut world = PhysicsWorld::default();
        world.set_gravity([0.0, 0.0].into());

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

        let mut fluid_grid = Switch::new(
            2,
            Grid::new(grid_cols_rows.0, grid_cols_rows.1, GridCell::default()),
        );
        if randomized_fluid > 0.0 {
            let mut rng = thread_rng();
            fluid_grid.get_mut().unwrap().fill_with(|_, _| {
                let x = rng.gen_range(-1.0, 1.0);
                let y = rng.gen_range(-1.0, 1.0);
                Some((Vec2::new(x, y).normalize() * randomized_fluid).into())
            });
        }

        Self {
            bounds,
            bodies: vec![],
            world,
            cache_bodies_triggered: vec![],
            cache_bodies_contacted: vec![],
            fluid_grid,
            fluid_diffuse,
            fluid_drag,
            cache_fluid_forces: vec![],
        }
    }

    #[inline]
    pub fn bounds(&self) -> (Scalar, Scalar) {
        self.bounds
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

    #[inline]
    pub fn apply_fluid_force(&mut self, position: Vec2, force: Vec2) {
        self.cache_fluid_forces.push((position, force));
    }

    pub fn process(&mut self, dt: Scalar) {
        if (self.world.timestep() - dt).abs() < 0.01 {
            self.world.set_timestep(dt);
        }
        self.process_fluid_forces();
        self.world.step();
        self.process_cache_bodies_triggered();
        self.process_cache_bodies_contacted();
        self.process_fluid_apply_forces(dt);
        self.process_fluid_propagate_and_diffuse(dt);
    }

    fn process_fluid_forces(&mut self) {
        let grid = self.fluid_grid.get_mut().unwrap();
        let cw = self.bounds.0 / grid.cols() as Scalar;
        let ch = self.bounds.1 / grid.rows() as Scalar;
        let hcw = cw * 0.5;
        let hch = ch * 0.5;
        for y in 0..grid.rows() {
            for x in 0..grid.cols() {
                let cp = Vec2::new(hcw + x as Scalar * cw, hch + y as Scalar * ch);
                let force = self
                    .cache_fluid_forces
                    .iter()
                    .filter_map(|(position, force)| {
                        let dist = (position - cp).magnitude();
                        if dist < 25.0 {
                            let f = 1.0 - dist * 0.04;
                            Some(force * f)
                        } else {
                            None
                        }
                    })
                    .sum::<Vec2>();
                grid[(x, y)] += force.into();
            }
        }
        self.cache_fluid_forces.clear();
    }

    fn process_cache_bodies_triggered(&mut self) {
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
    }

    fn process_cache_bodies_contacted(&mut self) {
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

    fn process_fluid_apply_forces(&mut self, dt: Scalar) {
        let (cw, ch) = {
            let grid = self.fluid_grid.get().unwrap();
            let cw = self.bounds.0 / grid.cols() as Scalar;
            let ch = self.bounds.1 / grid.rows() as Scalar;
            (cw, ch)
        };
        for i in 0..self.bodies.len() {
            let (handle, pos) = {
                let body = &self.bodies[i];
                (body.body_handle(), body.position(&self).unwrap())
            };
            if let Some(body) = self.world.rigid_body_mut(handle) {
                let sampler = GridSamplerDistance::new((pos.x, pos.y), 100.0, (cw, ch));
                let f: Vec2 = self
                    .fluid_grid
                    .get()
                    .unwrap()
                    .sample(sampler)
                    .unwrap()
                    .0
                    .into();
                let vel = body.velocity();
                let mass = body.augmented_mass();
                let dl = vel.linear * mass.linear * -self.fluid_drag;
                let da = vel.angular * mass.angular * -self.fluid_drag;
                let v = f + dl;
                body.apply_force(
                    0,
                    &Force2::from_slice(&[v.x * dt, v.y * dt, da]),
                    ForceType::Impulse,
                    true,
                );
            }
        }
    }

    fn process_fluid_propagate_and_diffuse(&mut self, dt: Scalar) {
        if let Some((prev, next)) = self.fluid_grid.iterate() {
            let cols = prev.cols();
            let rows = prev.rows();
            for y in 0..rows {
                for x in 0..cols {
                    let field_next = &mut next[(x, y)];
                    let sampler = GridSamplerCluster::center_extents((x, y), (1, 1));
                    if let Some((sample, weight)) = prev.sample(sampler) {
                        let sample = sample / weight as Scalar;
                        *field_next = sample;
                    }
                    if self.fluid_diffuse > 0.0 {
                        let factor = 1.0 - self.fluid_diffuse.max(0.0).min(1.0) * dt;
                        *field_next = *field_next * factor;
                    }
                }
            }
        }
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

#[derive(Debug, Copy, Clone, Default)]
struct GridCell(pub Scalar, pub Scalar);

impl Add for GridCell {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        GridCell(self.0 + other.0, self.1 + other.1)
    }
}

impl AddAssign for GridCell {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
        self.1 += other.1;
    }
}

impl Div<Scalar> for GridCell {
    type Output = Self;

    fn div(self, other: Scalar) -> Self {
        GridCell(self.0 / other, self.1 / other)
    }
}

impl From<Vec2> for GridCell {
    fn from(value: Vec2) -> Self {
        GridCell(value.x, value.y)
    }
}

impl Into<Vec2> for GridCell {
    fn into(self) -> Vec2 {
        Vec2::new(self.0, self.1)
    }
}

impl GridSampleZeroValue<Self> for GridCell {
    #[inline]
    fn sample_zero_value() -> Self {
        GridCell(0.0, 0.0)
    }
}

impl Mul<Scalar> for GridCell {
    type Output = Self;

    #[inline]
    fn mul(self, weight: Scalar) -> Self {
        GridCell(self.0 * weight, self.1 * weight)
    }
}
