//! The core part of the game.

use bevy::app::{App, Plugin};

pub mod assets;
pub mod cosmos;
pub mod localization;
pub mod map;
pub mod math;
pub mod schedule;
pub mod sci;
pub mod simulation;
pub mod util;

pub struct DystopiaCorePlugin;

impl Plugin for DystopiaCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            assets::DystopiaAssetsPlugin,
            cosmos::DystopiaCosmosPlugin,
            schedule::DystopiaSchedulePlugin,
            simulation::DystopiaSimulationPlugin,
        ));
    }
}
