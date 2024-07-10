//! Global and core simulation part. For detailed simulation, check `sim.rs`s
//! in corresponding modules.

use bevy::{
    app::{App, FixedUpdate, Plugin, Update},
    log::info,
    prelude::{IntoSystemConfigs, Res, ResMut, Resource},
    state::{condition::in_state, state::NextState},
};
use rand::rngs::StdRng;

use crate::schedule::{
    signal::InitializationSignal,
    state::{AssetState, GameState},
};

pub struct DystopiaSimulationPlugin;

impl Plugin for DystopiaSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
