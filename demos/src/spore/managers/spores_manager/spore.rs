use crate::managers::brains_manager::BrainsManager;
use crate::managers::items_manager::{ItemsManager, Named};
use crate::managers::physics_manager::body::BodyID;
use crate::managers::physics_manager::PhysicsManager;
use crate::managers::renderables_manager::renderable::{angle, Graphics, RenderableID};
use crate::managers::renderables_manager::RenderablesManager;
use psyche::core::brain::BrainID;
use psyche::core::brain_builder::BrainBuilder;
use psyche::core::effector::EffectorID;
use psyche::core::id::ID;
use psyche::core::Scalar;
use std::collections::HashMap;
use std::f64::consts::PI;

pub type SporeID = ID<Spore>;

#[derive(Debug, Clone, Default)]
pub struct LegState {
    pub renderable: RenderableID,
    pub angle: Scalar,
    pub phase: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SporeInner {
    pub body: BodyID,
    pub renderable_body: RenderableID,
    pub renderables_legs: HashMap<EffectorID, LegState>,
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
        let body = physics.create_with(|body, owner| {
            body.setup(owner, Some(position.into()), Some(rotation), Some(radius));
        });
        let mut brain = brain_builder.clone().radius(radius).build();
        brain.ignite_random_synapses(brain.synapses_count(), 10.0..10.0);
        let effectors = brain.get_effectors();
        let count = effectors.len();
        let renderables_legs = effectors
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
            .collect::<HashMap<_, _>>();
        let renderable_body = renderables.create_with(|renderable, _| {
            renderable.transform.position = position.into();
            renderable.transform.angle = angle(rotation);
            renderable.graphics = Graphics::Circle([1.0, 0.5, 0.5, 0.25], radius);
        });
        if let Some(spores) = renderables.hierarchy_mut("spores") {
            let children = renderables_legs
                .iter()
                .map(|(_, state)| state.renderable.into())
                .collect::<Vec<_>>();
            spores.children.push((renderable_body, children).into());
        }
        let brain = brains.add(brain);

        let inner = SporeInner {
            body,
            renderable_body,
            renderables_legs,
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
            for state in inner.renderables_legs.values() {
                renderables.destroy(state.renderable);
            }
            renderables.destroy(inner.renderable_body);
            brains.destroy(inner.brain);
            self.inner = None;
        }
    }

    pub fn process(&mut self, brains: &mut BrainsManager) {
        if let Some(ref mut inner) = self.inner {
            if let Some(brain) = brains.item_mut(inner.brain) {
                for (effector, state) in &mut inner.renderables_legs {
                    if let Ok(potential) = brain.effector_potential_release(*effector) {
                        if potential > 0.1 {
                            state.phase = (state.phase + 1) % 4;
                        }
                    }
                }
            }
        }
    }
}
