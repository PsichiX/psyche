extern crate amethyst;
extern crate psyche;
extern crate psyche_amethyst;

mod environment;
mod shiba;

use amethyst::{
    assets::{AssetStorage, Loader},
    core::{
        bundle::SystemBundle,
        frame_limiter::FrameRateLimitStrategy,
        transform::{Transform, TransformBundle},
        Error,
    },
    ecs::{prelude::DispatcherBuilder, prelude::Entity},
    // input::InputBundle,
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawFlat2D, Pipeline, PngFormat, Projection, RenderBundle,
        SpriteRender, SpriteSheet, SpriteSheetFormat, Stage, Texture, TextureMetadata,
    },
    ui::{DrawUi, UiBundle},
    utils::application_root_dir,
};
use environment::*;
use psyche::core::{brain_builder::BrainBuilder, config::Config as BrainConfig};
use psyche_amethyst::{BrainBundle, BrainComponent};
use shiba::*;
use std::path::PathBuf;
use std::time::Duration;

struct Example {
    pub camera_entity: Option<Entity>,
    pub agent_entity: Option<Entity>,
    pub target_entity: Option<Entity>,
    pub danger_entities: Vec<Entity>,
}

impl Default for Example {
    fn default() -> Self {
        Self {
            camera_entity: None,
            agent_entity: None,
            target_entity: None,
            danger_entities: vec![],
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
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
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "sprites.png",
                PngFormat,
                TextureMetadata::srgb_scale(),
                (),
                &texture_storage,
            )
        };
        let spritesheet_handle = {
            let loader = world.read_resource::<Loader>();
            let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
            loader.load(
                "sprites.ron",
                SpriteSheetFormat,
                texture_handle,
                (),
                &sprite_sheet_store,
            )
        };
        self.agent_entity = {
            let mut transform = Transform::default();
            transform.set_x(-96.0);
            let brain = BrainComponent::new(make_brain_builder());
            let shiba = ShibaComponent::new(&brain.brain);
            Some(
                world
                    .create_entity()
                    .with(SpriteRender {
                        sprite_sheet: spritesheet_handle.clone(),
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
                        sprite_sheet: spritesheet_handle.clone(),
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
                    sprite_sheet: spritesheet_handle.clone(),
                    sprite_number: 16,
                })
                .with(transform)
                .with(ObstacleComponent)
                .build()
        }];
    }
}

fn main() -> amethyst::Result<()> {
    // amethyst::start_logger(Default::default());

    let app_root: PathBuf = application_root_dir().into();
    let display_config_path = app_root.join("resources/display_config.ron");
    // let key_bindings_path = app_root.join("resources/input.ron");
    let assets_dir = app_root.join("resources/");

    let config = DisplayConfig::load(&display_config_path);
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::new())
            .with_pass(DrawUi::new()),
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(BrainBundle)?
        // .with_bundle(
        //     InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        // )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(GameBundle)?;
    let mut game = Application::build(assets_dir, Example::default())?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;

    game.run();

    Ok(())
}

fn make_brain_builder() -> BrainBuilder {
    let mut config = BrainConfig::default();
    config.propagation_speed = 50.0;
    config.synapse_reconnection_range = Some(15.0);
    config.neuron_potential_decay = 0.1;
    config.synapse_propagation_decay = 0.01;
    config.synapse_new_connection_receptors = Some(2.0);
    BrainBuilder::new()
        .config(config)
        .neurons(50)
        .connections(200)
        .min_neurogenesis_range(5.0)
        .max_neurogenesis_range(15.0)
        .radius(30.0)
        .sensors(4)
        .effectors(2)
}

struct GameBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GameBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(ShibaSystem, "shiba_system", &[]);
        builder.add(EnvironmentSystem, "environment_system", &[]);
        Ok(())
    }
}
