use bevy::{
    app::{App, Plugin},
    state::app::AppExtStates,
};

use crate::schedule::{
    signal::InitializationSignal,
    state::{AssetState, GameState, ProcessState},
};

pub mod signal;
pub mod state;

pub struct DystopiaSchedulePlugin;

impl Plugin for DystopiaSchedulePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InitializationSignal>()
            .init_state::<ProcessState>()
            .init_state::<AssetState>()
            .init_state::<GameState>();
    }
}
