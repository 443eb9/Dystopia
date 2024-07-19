use bevy::app::{App, Plugin};

pub mod bundle;
pub mod render;
pub mod tilemap;

pub struct DystopiaMapPlugin;

impl Plugin for DystopiaMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(render::DystopiaMapRenderPlugin);
    }
}
