use bevy::{
    app::{App, Plugin, Startup},
    log::info,
    prelude::{Camera2dBundle, Commands, ResMut},
    state::state::{NextState, OnEnter},
};
use dystopia_core::{
    cosmos::gen::CosmosGenerationSettings,
    schedule::state::{AssetState, GameState},
};

pub struct InfGdnDebugPlugin;

impl Plugin for InfGdnDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetState::Finish), skip_menu)
            .add_systems(Startup, setup_debug);
    }
}

fn setup_debug(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn skip_menu(mut commands: Commands, mut game_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(CosmosGenerationSettings {
        seed: 0,
        num_stars: 60..69,
    });
    game_state.set(GameState::Initialize);
    info!("Skipped menu");
}
