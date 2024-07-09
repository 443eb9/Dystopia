//! Generating, simulating and (de)serializing the cosmos.
//!
//! Here, the cosmos does NOT contains data like crops on maps or somewhat detailed. It only
//! contains the basic bodies info like positions, velocities etc.

use bevy::{
    app::{App, Plugin, Update},
    prelude::IntoSystemConfigs,
    state::condition::in_state,
};

use crate::{
    assets::app_ext::InfGdnAssetAppExt,
    cosmos::config::{CosmosStarPropertiesConfig, RawCosmosStarPropertiesConfig},
    schedule::state::{AssetState, GameState},
};

pub mod celestial;
pub mod config;
pub mod gen;
pub mod unit;

pub struct InfGdnCosmosPlugin;

impl Plugin for InfGdnCosmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            gen::generate_cosmos
                .run_if(in_state(AssetState::Finish))
                .run_if(in_state(GameState::Initialize)),
        )
        .add_config::<CosmosStarPropertiesConfig, RawCosmosStarPropertiesConfig>();
    }
}
