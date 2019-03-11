#![allow(dead_code)]

use crate::managers::items_manager::Named;
use crate::managers::physics_manager::PhysicsManager;
use nalgebra::Vector2;
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::algebra::{Force2, ForceType};
use nphysics2d::object::Body as PhysicsBody;
use nphysics2d::object::{BodyHandle, ColliderDesc, ColliderHandle, RigidBodyDesc};
use psyche::core::id::ID;
use psyche::core::Scalar;
use std::fmt;

pub type BodyID = ID<Body>;
pub type Vec2 = Vector2<Scalar>;

#[derive(Debug, Clone)]
pub struct BodyState {
    pub position: Vec2,
    pub rotation: Scalar,
    pub radius: Scalar,
    pub is_sensor: bool,
}

impl Default for BodyState {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            radius: 0.0,
            is_sensor: false,
        }
    }
}

#[derive(Clone)]
pub struct Body {
    id: BodyID,
    shape_handle: ShapeHandle<Scalar>,
    collider_handle: ColliderHandle,
    body_handle: BodyHandle,
    cached_state: BodyState,
}

impl Named<Self> for Body {
    #[inline]
    fn id(&self) -> BodyID {
        self.id
    }
}

impl Body {
    pub fn new(owner: &mut PhysicsManager, is_sensor: bool) -> Self {
        let body = RigidBodyDesc::new().build(owner.world_mut());
        let body_handle = body.handle();
        let shape_handle = ShapeHandle::new(Ball::new(1.0));
        let collider = ColliderDesc::new(shape_handle.clone())
            .density(1.0)
            .sensor(is_sensor)
            .build_with_parent(body.part_handle(), owner.world_mut())
            .unwrap();
        Self {
            id: BodyID::new(),
            shape_handle,
            collider_handle: collider.handle(),
            body_handle,
            cached_state: Default::default(),
        }
    }

    pub fn free(&self, owner: &mut PhysicsManager) {
        owner.world_mut().remove_colliders(&[self.collider_handle]);
        owner.world_mut().remove_bodies(&[self.body_handle]);
    }

    #[inline]
    pub fn cached_state(&self) -> &BodyState {
        &self.cached_state
    }

    #[inline]
    pub(crate) fn cache_state(&mut self, state: BodyState) {
        self.cached_state = state;
    }

    #[inline]
    pub fn body_handle(&self) -> BodyHandle {
        self.body_handle
    }

    #[inline]
    pub fn collider_handle(&self) -> ColliderHandle {
        self.collider_handle
    }

    #[inline]
    pub fn shape_handle(&self) -> &ShapeHandle<Scalar> {
        &self.shape_handle
    }

    pub fn set_radius(&self, value: Scalar) {
        if let Some(ball) = self.shape_handle.as_shape::<Ball<_>>() {
            unsafe {
                *(ball as *const Ball<_> as *mut Ball<_>) = Ball::new(value);
            }
        }
    }

    pub fn apply_force_adv(
        &self,
        owner: &mut PhysicsManager,
        linear: Vec2,
        angular: Scalar,
        force_type: ForceType,
    ) {
        if let Some(body) = owner.world_mut().rigid_body_mut(self.body_handle) {
            body.apply_force(
                0,
                &Force2::from_slice(&[linear.x, linear.y, angular]),
                force_type,
                true,
            );
        }
    }

    pub fn apply_local_force_adv(
        &self,
        owner: &mut PhysicsManager,
        linear: Vec2,
        angular: Scalar,
        force_type: ForceType,
    ) {
        if let Some(body) = owner.world_mut().rigid_body_mut(self.body_handle) {
            body.apply_local_force(
                0,
                &Force2::from_slice(&[linear.x, linear.y, angular]),
                force_type,
                true,
            );
        }
    }

    pub fn apply_force(&self, owner: &mut PhysicsManager, linear: Vec2, angular: Scalar) {
        self.apply_force_adv(owner, linear, angular, ForceType::Force)
    }

    pub fn apply_local_force(&self, owner: &mut PhysicsManager, linear: Vec2, angular: Scalar) {
        self.apply_force_adv(owner, linear, angular, ForceType::Force)
    }

    pub fn apply_impulse(&self, owner: &mut PhysicsManager, linear: Vec2, angular: Scalar) {
        self.apply_force_adv(owner, linear, angular, ForceType::Impulse)
    }

    pub fn apply_local_impulse(&self, owner: &mut PhysicsManager, linear: Vec2, angular: Scalar) {
        self.apply_force_adv(owner, linear, angular, ForceType::Impulse)
    }

    pub fn apply_acceleration_change(
        &self,
        owner: &mut PhysicsManager,
        linear: Vec2,
        angular: Scalar,
    ) {
        self.apply_force_adv(owner, linear, angular, ForceType::AccelerationChange)
    }

    pub fn apply_local_acceleration_change(
        &self,
        owner: &mut PhysicsManager,
        linear: Vec2,
        angular: Scalar,
    ) {
        self.apply_force_adv(owner, linear, angular, ForceType::AccelerationChange)
    }

    pub fn apply_velocity_change(&self, owner: &mut PhysicsManager, linear: Vec2, angular: Scalar) {
        self.apply_force_adv(owner, linear, angular, ForceType::VelocityChange)
    }

    pub fn apply_local_velocity_change(
        &self,
        owner: &mut PhysicsManager,
        linear: Vec2,
        angular: Scalar,
    ) {
        self.apply_force_adv(owner, linear, angular, ForceType::VelocityChange)
    }
}

impl fmt::Debug for Body {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body")
            .field("id", &self.id)
            .field("shape_handle", &"[...]".to_owned())
            .field("collider_handle", &self.collider_handle)
            .field("body_handle", &self.body_handle)
            .finish()
    }
}
