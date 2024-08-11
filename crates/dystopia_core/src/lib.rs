//! The core part of the game.

use bevy::app::{App, Plugin};

pub mod assets;
pub mod cosmos;
pub mod input;
pub mod localization;
pub mod map;
pub mod math;
pub mod schedule;
pub mod sci;
pub mod simulation;
pub mod transition;
pub mod ui;
pub mod util;

pub struct DystopiaCorePlugin;

impl Plugin for DystopiaCorePlugin {
    fn build(&self, app: &mut App) {
        match dotenvy::dotenv() {
            Ok(_) => {}
            Err(err) => {
                panic!("Failed to load .env file: {}", err)
            }
        }

        app.add_plugins((
            assets::DystopiaAssetsPlugin,
            cosmos::DystopiaCosmosPlugin,
            input::DystopiaInputPlugin,
            localization::DystopiaLocalizationPlugin,
            map::DystopiaMapPlugin,
            schedule::DystopiaSchedulePlugin,
            simulation::DystopiaSimulationPlugin,
            transition::DystopiaTransitionPlugin,
            ui::DystopiaUiPlugin,
            util::DystopiaUtilPlugin,
        ));
    }
}
