//! The core part of the game.

use bevy::app::{App, Plugin};

mod assets;
mod cosmos;
mod localization;
mod math;
mod physics;
mod schedule;

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
