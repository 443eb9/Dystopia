use bevy::app::{App, Plugin};

pub mod bundle;
pub mod gen;
pub mod quantify;
pub mod render;
pub mod serde;
pub mod shape;
pub mod tilemap;

pub struct DystopiaMapPlugin;

impl Plugin for DystopiaMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            render::TilemapRenderPlugin,
            serde::TilemapSerdePlugin,
            gen::TilemapGenerationPlugin,
        ));
    }
}
