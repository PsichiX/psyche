use crate::{
    components::{obstacle::ObstacleComponent, shiba::ShibaComponent, target::TargetComponent},
    data::{asset_database::AssetDatabase, settings::SettingsData, simulation::SimulationData},
};
use amethyst::{
    core::{timing::Time, transform::Transform},
    ecs::prelude::Entity,
    input::is_key_down,
    prelude::*,
    renderer::{Camera, Projection, SpriteRender, VirtualKeyCode},
};
use psyche_amethyst::BrainComponent;
use std::{
    fs::{read, write},
    str::from_utf8,
};

const MAX_SESSION_TIME: f32 = 15.0;
const MAX_DISTANCE: f32 = 1000.0 * 0.25;

pub struct SimulationState {
    pub camera_entity: Option<Entity>,
    pub agent_entity: Option<Entity>,
    pub target_entity: Option<Entity>,
    pub danger_entities: Vec<Entity>,
    pub session_time: f32,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            camera_entity: None,
            agent_entity: None,
            target_entity: None,
            danger_entities: vec![],
            session_time: 0.0,
        }
    }
}

impl SimpleState for SimulationState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        {
            let snapshot_path = { world.read_resource::<SettingsData>().snapshot_path.clone() };
            if let Some(snapshot_path) = snapshot_path {
                if let Ok(bytes) = read(&snapshot_path) {
                    if let Ok(json) = from_utf8(&bytes) {
                        if let Ok(data) = serde_json::from_str(json) {
                            *world.write_resource::<SimulationData>() = data;
                        }
                    }
                }
            }
        }

        let sprite_sheet_handle = {
            let asset_base = world.read_resource::<AssetDatabase>();
            asset_base.sprite_sheet.clone().unwrap()
        };
        let brain = {
            world
                .read_resource::<SimulationData>()
                .brain_scored
                .0
                .clone()
        };

        self.camera_entity = {
            let mut transform = Transform::default();
            transform.set_z(1.0);
            let hw = 1024.0 * 0.5 * 0.25;
            let hh = 768.0 * 0.5 * 0.25;
            Some(
                world
                    .create_entity()
                    .with(Camera::from(Projection::orthographic(-hw, hw, -hh, hh)))
                    .with(transform)
                    .build(),
            )
        };
        self.agent_entity = {
            let mut transform = Transform::default();
            transform.set_x(-96.0);
            let brain = BrainComponent::new(brain);
            let shiba = ShibaComponent::new(&brain.brain);
            Some(
                world
                    .create_entity()
                    .with(SpriteRender {
                        sprite_sheet: sprite_sheet_handle.clone(),
                        sprite_number: 0,
                    })
                    .with(transform)
                    .with(brain)
                    .with(shiba)
                    .build(),
            )
        };
        self.target_entity = {
            let mut transform = Transform::default();
            transform.set_x(96.0);
            transform.set_scale(0.5, 0.5, 0.5);
            Some(
                world
                    .create_entity()
                    .with(SpriteRender {
                        sprite_sheet: sprite_sheet_handle.clone(),
                        sprite_number: 19,
                    })
                    .with(transform)
                    .with(TargetComponent)
                    .build(),
            )
        };
        self.danger_entities = vec![{
            let mut transform = Transform::default();
            transform.set_scale(0.5, 0.5, 0.5);
            world
                .create_entity()
                .with(SpriteRender {
                    sprite_sheet: sprite_sheet_handle.clone(),
                    sprite_number: 16,
                })
                .with(transform)
                .with(ObstacleComponent)
                .build()
        }];
        self.session_time = 0.0;
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        if let Some(entity) = self.camera_entity {
            drop(world.delete_entity(entity));
            self.camera_entity = None;
        }
        if let Some(entity) = self.agent_entity {
            drop(world.delete_entity(entity));
            self.agent_entity = None;
        }
        if let Some(entity) = self.target_entity {
            drop(world.delete_entity(entity));
            self.target_entity = None;
        }
        for entity in &self.danger_entities {
            drop(world.delete_entity(*entity));
        }
        self.danger_entities = vec![];
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;
        {
            self.session_time += world.read_resource::<Time>().delta_seconds();
        }

        if self.session_time >= MAX_SESSION_TIME {
            self.done(world);
            return Trans::Switch(Box::new(Self::default()));
        }
        Trans::None
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                let world = data.world;
                {
                    let snapshot_path =
                        { world.read_resource::<SettingsData>().snapshot_path.clone() };
                    if let Some(snapshot_path) = snapshot_path {
                        let data: &SimulationData = &world.read_resource();
                        if let Ok(json) = serde_json::to_string_pretty(data) {
                            drop(write(&snapshot_path, json));
                        }
                    }
                }
                return Trans::Quit;
            }
        }
        Trans::None
    }
}

impl SimulationState {
    fn done(&self, world: &mut World) {
        let score = self.score(world);

        match world.write_resource::<SimulationData>().mutate(score) {
            true => println!("PROGRESSION! score: {}", score),
            false => println!("REGRESSION! score: {}", score),
        }
    }

    fn score(&self, world: &mut World) -> f32 {
        let time_score = MAX_SESSION_TIME - self.session_time;
        let distance_score = MAX_DISTANCE - {
            let agent_pos = {
                *world
                    .read_storage::<Transform>()
                    .get(self.agent_entity.unwrap())
                    .unwrap()
                    .translation()
            };
            let target_pos = {
                *world
                    .read_storage::<Transform>()
                    .get(self.target_entity.unwrap())
                    .unwrap()
                    .translation()
            };
            (target_pos - agent_pos).magnitude()
        };
        time_score + distance_score
    }
}
