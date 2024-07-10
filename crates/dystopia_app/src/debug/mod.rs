use bevy::{
    app::{App, Plugin, Startup},
    log::info,
    prelude::{Camera2dBundle, Commands, ResMut},
    state::state::{NextState, OnEnter},
};
use bevy_pancam::PanCam;
use dystopia_core::{
    cosmos::gen::CosmosGenerationSettings,
    schedule::state::{AssetState, GameState},
};

pub struct DystopiaDebugPlugin;

impl Plugin for DystopiaDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetState::Finish), skip_menu)
            .add_systems(Startup, setup_debug);
    }
}

fn setup_debug(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}

fn skip_menu(mut commands: Commands, mut game_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(CosmosGenerationSettings {
        seed: 1,
        galaxy_radius: 1_000_000.,
        // num_stars: 60..69,
        num_stars: 1..2,
    });
    game_state.set(GameState::Initialize);
    info!("Skipped menu");
}
