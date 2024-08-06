//! Global and core simulation part. For detailed simulation, check `sim.rs`s
//! in corresponding modules.

use bevy::{
    app::{App, FixedUpdate, Plugin, Update},
    log::info,
    math::Vec2,
    prelude::{
        Component, Deref, DerefMut, DetectChanges, IntoSystemConfigs, Query, Res, ResMut, Resource,
        With,
    },
    render::{
        camera::OrthographicProjection,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
    },
    state::{condition::in_state, state::NextState},
    window::Window,
};
use rand::rngs::StdRng;

use crate::schedule::{
    signal::InitializationSignal,
    state::{AssetState, GameState},
};

pub struct DystopiaSimulationPlugin;

impl Plugin for DystopiaSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<MainCamera>::default())
            .add_systems(Update, (update_window_related_data, sync_view_scale))
            .add_systems(
                FixedUpdate,
                global_clock.run_if(in_state(GameState::Simulate)),
            )
            .add_systems(
                Update,
                check_if_initialized
                    .run_if(in_state(AssetState::Finish))
                    .run_if(in_state(GameState::Initialize)),
            )
            .init_resource::<CursorPosition>()
            .init_resource::<ViewScale>()
            .init_resource::<WindowSize>();
    }
}

/// Marker struct for main camera. The game only allows one main camera.
#[derive(Component, ExtractComponent, Default, Clone)]
pub struct MainCamera;

/// The only choice in this game if you want to scale the camera. It is
/// not allowed to directly change the `scale` in
/// [`OrthographicProjection`](bevy::render::camera::OrthographicProjection)
/// or `scale` in [`Transform`](bevy::transform::components::Transform) or
/// anything like them.
#[derive(Resource, Deref, DerefMut)]
pub struct ViewScale(f32);

impl Default for ViewScale {
    fn default() -> Self {
        Self(1.)
    }
}

fn sync_view_scale(
    view_scale: Res<ViewScale>,
    mut camera: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    if !view_scale.is_changed() {
        return;
    }

    let Ok(mut camera) = camera.get_single_mut() else {
        panic!("Exactly one main camera can exist at a time.")
    };

    camera.scale = **view_scale;
}

/// The RNG used across the entire game.
///
/// For wold that is generated, this RNG will be inserted when generating
/// cosmos, and for those are loaded, this will be loaded from the save.
#[derive(Resource, Deref, DerefMut)]
pub struct GlobalRng(StdRng);

impl GlobalRng {
    pub fn new(rng: StdRng) -> Self {
        Self(rng)
    }
}

#[derive(Resource, Default, Deref)]
pub struct Ticker(u64);

pub fn global_clock(mut ticker: ResMut<Ticker>) {
    ticker.0 += 1;
}

#[derive(Resource, Default, Deref)]
pub struct CursorPosition(Option<Vec2>);

#[derive(Resource, Default, Deref)]
pub struct WindowSize(Vec2);

fn update_window_related_data(
    windows_query: Query<&Window>,
    mut pos: ResMut<CursorPosition>,
    mut size: ResMut<WindowSize>,
) {
    let window = windows_query
        .get_single()
        .expect("Multiple windows detected, which is not allowed.");
    pos.0 = window.cursor_position();
    size.0 = window.size();
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
