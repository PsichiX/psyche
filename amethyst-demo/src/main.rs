extern crate amethyst;
extern crate psyche;
extern crate psyche_amethyst;

mod components;
mod data;
mod states;
mod systems;

use crate::{
    states::loading::LoadingState,
    systems::{environment::EnvironmentSystem, shiba::ShibaSystem},
};
use amethyst::{
    core::{
        bundle::SystemBundle, frame_limiter::FrameRateLimitStrategy, transform::TransformBundle,
        Error,
    },
    ecs::prelude::DispatcherBuilder,
    // input::InputBundle,
    prelude::*,
    renderer::{DisplayConfig, DrawFlat2D, Pipeline, RenderBundle, Stage},
    ui::{DrawUi, UiBundle},
    utils::application_root_dir,
};
use psyche_amethyst::BrainBundle;
use std::path::PathBuf;
use std::time::Duration;

pub type Vector = (f32, f32, f32);

struct GameBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GameBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(EnvironmentSystem, "environment_system", &[]);
        builder.add(ShibaSystem, "shiba_system", &["environment_system"]);
        Ok(())
    }
}

fn main() -> amethyst::Result<()> {
    // amethyst::start_logger(Default::default());

    let app_root: PathBuf = application_root_dir().into();
    let display_config_path = app_root.join("assets/display.ron");
    // let key_bindings_path = app_root.join("resources/input.ron");
    let assets_dir = app_root.join("assets/");

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
    let mut game = Application::build(assets_dir, LoadingState)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;

    game.run();

    Ok(())
}
