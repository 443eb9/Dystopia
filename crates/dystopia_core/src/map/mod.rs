use bevy::app::{App, Plugin};

use crate::map::{render::TilemapRenderPlugin, serde::TilemapSerdePlugin};

pub mod bundle;
pub mod render;
pub mod serde;
pub mod shape;
pub mod tilemap;

pub struct DystopiaMapPlugin;

impl Plugin for DystopiaMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TilemapRenderPlugin, TilemapSerdePlugin));
    }
}
