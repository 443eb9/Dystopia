//! The core part of the game.

use bevy::app::{App, Plugin};

pub mod assets;
pub mod cosmos;
pub mod localization;
pub mod math;
pub mod schedule;
pub mod simulation;

pub struct InfGdnCorePlugin;

impl Plugin for InfGdnCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            assets::InfGdnAssetsPlugin,
            cosmos::InfGdnCosmosPlugin,
            schedule::InfGdnSchedulePlugin,
        ));
    }
}
