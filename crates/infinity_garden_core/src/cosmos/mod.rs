//! Generating, simulating and (de)serializing the cosmos.
//!
//! Here, the cosmos does NOT contains data like crops on maps or somewhat detailed. It only
//! contains the basic bodies info like positions, velocities etc.

use bevy::{
    app::{App, Plugin},
    prelude::IntoSystemConfigs,
    state::{
        app::AppExtStates,
        condition::in_state,
        state::{OnEnter, States},
    },
};

use crate::{
    assets::app_ext::InfGdnAssetAppExt,
    cosmos::config::{CosmosStarPropertiesConfig, RawCosmosStarPropertiesConfig},
    schedule::{state::GameState, system_set::InitializeSet},
};

pub mod celestial;
pub mod config;
pub mod gen;
pub mod unit;

pub struct InfGdnCosmosPlugin;

impl Plugin for InfGdnCosmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Initialize),
            gen::generate_cosmos.in_set(InitializeSet),
        )
        .add_config::<CosmosStarPropertiesConfig, RawCosmosStarPropertiesConfig>();
    }
}
