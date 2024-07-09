use bevy::{
    app::{App, Plugin},
    prelude::{Camera2dBundle, Commands},
};

pub struct InfGdnDebugPlugin;

impl Plugin for InfGdnDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(bevy::app::Startup, setup_debug);
    }
}

fn setup_debug(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
