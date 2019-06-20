use crate::{
    data::{
        asset_database::AssetDatabase, environment::EnvironmentData, simulation::SimulationData,
    },
    states::simulation::SimulationState,
};
use amethyst::{
    assets::{AssetStorage, Loader},
    prelude::*,
    renderer::{PngFormat, SpriteSheet, SpriteSheetFormat, Texture, TextureMetadata},
};

pub struct LoadingState;

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        {
            world.add_resource(AssetDatabase::default());
            world.add_resource(EnvironmentData::default());
            world.add_resource(SimulationData::default());
        }

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
        let sprite_sheet_handle = {
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

        {
            let mut asset_base = world.write_resource::<AssetDatabase>();
            asset_base.sprite_sheet = Some(sprite_sheet_handle);
        }
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::Switch(Box::new(SimulationState::default()))
    }
}
