use bevy::{log::info, prelude::Res};

use crate::cosmos::config::CosmosStarPropertiesConfig;

pub fn generate_cosmos(star_pros: Res<CosmosStarPropertiesConfig>) {
    info!("Start generating cosmos...");
}
