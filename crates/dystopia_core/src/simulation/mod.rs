//! Global and core simulation part. For detailed simulation, check `sim.rs`s
//! in corresponding modules.

use bevy::{
    app::{App, FixedUpdate, Plugin, Update},
    log::info,
    prelude::{Component, DetectChanges, IntoSystemConfigs, Query, Res, ResMut, Resource, With},
    render::camera::OrthographicProjection,
    state::{condition::in_state, state::NextState},
};
use rand::rngs::StdRng;

use crate::{
    impl_rw_tuple_struct,
    schedule::{
        signal::InitializationSignal,
        state::{AssetState, GameState},
    },
};

pub struct DystopiaSimulationPlugin;

impl Plugin for DystopiaSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewScale>()
            .add_systems(Update, sync_view_scale)
            .add_systems(
                FixedUpdate,
                global_clock.run_if(in_state(GameState::Simulate)),
            )
            .add_systems(
                Update,
                check_if_initialized
                    .run_if(in_state(AssetState::Finish))
                    .run_if(in_state(GameState::Initialize)),
            );
    }
}

/// Marker struct for main camera. The game only allows one main camera.
#[derive(Component, Default)]
pub struct MainCamera;

/// The only choice in this game if you want to scale the camera. It is
/// not allowed to directly change the `scale` in
/// [`OrthographicProjection`](bevy::render::camera::OrthographicProjection)
/// or `scale` in [`Transform`](bevy::transform::components::Transform) or
/// anything like them.
#[derive(Resource)]
pub struct ViewScale(f32);
impl_rw_tuple_struct!(ViewScale, f32);

impl Default for ViewScale {
    fn default() -> Self {
        Self(1.)
    }
}

// TODO uncomment this after finishing camera management.
fn sync_view_scale(
    view_scale: Res<ViewScale>,
    mut camera: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    // if !view_scale.is_changed() {
    //     return;
    // }

    // let Ok(mut camera) = camera.get_single_mut() else {
    //     panic!("Exactly one main camera can exist at a time.")
    // };

    // camera.scale = *view_scale.get();
}

/// The RNG used across the entire game.
///
/// For wold that is generated, this RNG will be inserted when generating
/// cosmos, and for those are loaded, this will be loaded from the save.
#[derive(Resource)]
pub struct GlobalRng(pub StdRng);

#[derive(Resource)]
pub struct Ticker(pub u64);

pub fn global_clock(mut ticker: ResMut<Ticker>) {
    ticker.0 += 1;
}

fn check_if_initialized(
    signals: Res<InitializationSignal>,
    mut state: ResMut<NextState<GameState>>,
) {
    // TODO: Remove `|| true` when finished world generation & loading.
    if signals.cosmos_initialized && (signals.world_initialized || true) {
        state.set(GameState::Simulate);
        info!("Game initialized! Start Simulating...");
    }
}
