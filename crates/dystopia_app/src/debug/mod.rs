use bevy::{
    app::{App, Plugin, Startup, Update},
    log::info,
    prelude::{Camera2dBundle, Changed, Commands, Query, ResMut},
    render::camera::OrthographicProjection,
    state::state::{NextState, OnEnter},
};
use bevy_pancam::PanCam;
use dystopia_core::{
    cosmos::gen::CosmosGenerationSettings,
    schedule::state::{AssetState, GameState},
    sci::unit::Length,
    simulation::{MainCamera, ViewScale},
};

pub struct DystopiaDebugPlugin;

impl Plugin for DystopiaDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_sync_scale)
            .add_systems(OnEnter(AssetState::Finish), debug_skip_menu)
            .add_systems(Startup, setup_debug);
    }
}

fn setup_debug(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PanCam::default(), MainCamera));
}

fn debug_sync_scale(
    mut view_scale: ResMut<ViewScale>,
    camera: Query<&OrthographicProjection, Changed<OrthographicProjection>>,
) {
    let Ok(camera) = camera.get_single() else {
        return;
    };
    view_scale.set(camera.scale);
}

fn debug_skip_menu(mut commands: Commands, mut game_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(CosmosGenerationSettings {
        seed: 2,
        galaxy_radius: Length::LightYear(1.),
        // num_stars: 60..69,
        num_stars: 1..2,
    });
    game_state.set(GameState::Initialize);
    info!("Skipped menu");
}
