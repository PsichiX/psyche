use crate::managers::brains_manager::BrainsManager;
use crate::managers::food_manager::FoodManager;
use crate::managers::items_manager::{ItemsManager, Named};
use crate::managers::physics_manager::body::{BodyID, Vec2};
use crate::managers::physics_manager::PhysicsManager;
use crate::managers::renderables_manager::renderable::{angle, Graphics, RenderableID};
use crate::managers::renderables_manager::RenderablesManager;
use psyche::core::brain::BrainID;
use psyche::core::brain_builder::BrainBuilder;
use psyche::core::effector::EffectorID;
use psyche::core::id::ID;
use psyche::core::sensor::SensorID;
use psyche::core::Scalar;
use std::collections::HashMap;
use std::f64::consts::PI;

const POTENTIAL_CALORIES_SCALE: Scalar = 0.01;

pub type SporeID = ID<Spore>;

#[derive(Debug, Clone, Default)]
pub struct LegState {
    pub renderable: RenderableID,
    pub angle: Scalar,
    pub phase: usize,
}

#[derive(Debug, Clone, Default)]
pub struct DetectorState {
    pub renderable: RenderableID,
    pub angle: Scalar,
    pub potential: Scalar,
}

#[derive(Debug, Clone, Default)]
pub struct SporeInner {
    pub body: BodyID,
    pub renderable_body: RenderableID,
    pub legs: HashMap<EffectorID, LegState>,
    pub detectors: HashMap<SensorID, DetectorState>,
    pub brain: BrainID,
}

#[derive(Debug, Clone, Default)]
pub struct Spore {
    id: SporeID,
    calories: Scalar,
    inner: Option<SporeInner>,
}

impl Named<Self> for Spore {
    #[inline]
    fn id(&self) -> SporeID {
        self.id
    }
}

impl Spore {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn calories(&self) -> Scalar {
        self.calories
    }

    #[inline]
    pub fn feed(&mut self, calories: Scalar) {
        self.calories += calories;
    }

    #[inline]
    pub fn inner(&self) -> Option<&SporeInner> {
        if let Some(ref inner) = self.inner {
            Some(inner)
        } else {
            None
        }
    }

    pub fn born(
        &mut self,
        position_rotation_radius: ([Scalar; 2], Scalar, Scalar),
        brain_builder: &BrainBuilder,
        physics: &mut PhysicsManager,
        renderables: &mut RenderablesManager,
        brains: &mut BrainsManager,
    ) {
        if self.inner.is_some() {
            return;
        }

        let (position, rotation, radius) = position_rotation_radius;
        self.calories = radius * radius * PI;
        let body = physics.create_with(|body, owner| {
            owner.setup(body, Some(position.into()), Some(rotation));
            body.set_radius(radius);
        });
        let brain = brain_builder.clone().radius(radius).build();
        let legs = {
            let effectors = brain.get_effectors();
            let count = effectors.len();
            effectors
                .into_iter()
                .enumerate()
                .map(|(i, id)| {
                    let rot = PI * 2.0 * (i as Scalar) / (count as Scalar);
                    let pos = [rot.cos() * radius, rot.sin() * radius];
                    let renderable = renderables.create_with(|renderable, _| {
                        renderable.transform.position = pos.into();
                        renderable.transform.angle = angle(rot);
                        renderable.graphics =
                            Graphics::Line([0.5, 1.0, 0.5, 0.25], radius * 0.3, radius * 0.05);
                    });
                    (
                        id,
                        LegState {
                            renderable,
                            angle: rot,
                            phase: 0,
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        };
        let detectors = {
            let sensors = brain.get_sensors();
            let count = sensors.len();
            sensors
                .into_iter()
                .enumerate()
                .map(|(i, id)| {
                    let rot = PI * (1.0 + 2.0 * (i as Scalar) / (count as Scalar));
                    let pos = [rot.cos() * radius, rot.sin() * radius];
                    let renderable = renderables.create_with(|renderable, _| {
                        renderable.transform.position = pos.into();
                        renderable.transform.angle = angle(rot);
                        renderable.graphics = Graphics::Rectangle(
                            [1.0, 1.0, 0.0, 0.25],
                            [radius * 0.1, radius * 0.1].into(),
                        );
                    });
                    (
                        id,
                        DetectorState {
                            renderable,
                            angle: rot,
                            potential: 0.0,
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        };
        let renderable_body = renderables.create_with(|renderable, _| {
            renderable.transform.position = position.into();
            renderable.transform.angle = angle(rotation);
            renderable.graphics = Graphics::Circle([1.0, 0.5, 0.5, 0.25], radius);
        });
        if let Some(spores) = renderables.hierarchy_mut("spores") {
            let children = legs
                .iter()
                .map(|(_, state)| state.renderable.into())
                .chain(detectors.iter().map(|(_, state)| state.renderable.into()))
                .collect::<Vec<_>>();
            spores.children.push((renderable_body, children).into());
        }
        let brain = brains.add(brain);

        let inner = SporeInner {
            body,
            renderable_body,
            legs,
            detectors,
            brain,
        };
        self.inner = Some(inner);
    }

    pub fn annihilate(
        &mut self,
        physics: &mut PhysicsManager,
        renderables: &mut RenderablesManager,
        brains: &mut BrainsManager,
    ) {
        if let Some(ref inner) = self.inner {
            physics.destroy(inner.body);
            for state in inner.legs.values() {
                renderables.destroy(state.renderable);
            }
            renderables.destroy(inner.renderable_body);
            brains.destroy(inner.brain);
            self.inner = None;
        }
    }

    pub fn process(
        &mut self,
        brains: &mut BrainsManager,
        physics: &mut PhysicsManager,
        food: &FoodManager,
    ) {
        if let Some(ref mut inner) = self.inner {
            if let Some(brain) = brains.item_mut(inner.brain) {
                if let Some(body) = physics.item(inner.body) {
                    let (position, rotation, radius) = {
                        let state = body.cached_state();
                        (state.position, state.rotation, state.radius)
                    };
                    if !inner.detectors.is_empty() {
                        let fov = PI / inner.detectors.len() as Scalar;
                        for (sensor, detector_state) in &mut inner.detectors {
                            let r = rotation + detector_state.angle;
                            let direction = Vec2::new(r.cos(), r.sin());
                            let potential = physics.sample_field_of_view(
                                position,
                                direction,
                                fov,
                                None,
                                |spatial| food.item_by_body(spatial.body).is_some(),
                            );
                            if potential > 0.1 && self.calories > 0.0 {
                                drop(brain.sensor_trigger_impulse(*sensor, potential));
                                detector_state.potential = potential;
                                self.calories -= potential * POTENTIAL_CALORIES_SCALE;
                            }
                        }
                    }
                    for (effector, leg_state) in &mut inner.legs {
                        if let Ok(potential) = brain.effector_potential_release(*effector) {
                            if potential > 0.1 && self.calories > 0.0 {
                                leg_state.phase = (leg_state.phase + 1) % 4;
                                let r = rotation + leg_state.angle;
                                let f = Vec2::new(r.cos(), r.sin()) * radius * -0.1;
                                physics.apply_fluid_force(position, f);
                                self.calories -= potential * POTENTIAL_CALORIES_SCALE;
                            }
                        }
                    }
                }
            }
        }
    }
}
