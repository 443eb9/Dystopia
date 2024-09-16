//! The core part of the game.

use bevy::app::{App, Plugin};

pub mod assets;
pub mod body;
pub mod character;
pub mod cosmos;
pub mod input;
pub mod localization;
pub mod map;
pub mod math;
pub mod scene;
pub mod schedule;
pub mod sci;
pub mod serde;
pub mod sim;
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
            character::DystopiaCharacterPlugin,
            cosmos::DystopiaCosmosPlugin,
            input::DystopiaInputPlugin,
            localization::DystopiaLocalizationPlugin,
            map::DystopiaMapPlugin,
            serde::DystopiaSerdePlugin,
            scene::DystopiaScenePlugin,
            schedule::DystopiaSchedulePlugin,
            sim::DystopiaSimulationPlugin,
            ui::DystopiaUiPlugin,
            util::DystopiaUtilPlugin,
        ));
    }
}
