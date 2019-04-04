use amethyst::{assets::Handle, renderer::SpriteSheet};

#[derive(Debug, Default)]
pub struct AssetDatabase {
    pub sprite_sheet: Option<Handle<SpriteSheet>>,
}
