//! Generating, simulating and (de)serializing the cosmos.
//!
//! Here, the cosmos does NOT contains data like crops on maps or somewhat detailed. It only
//! contains the basic bodies info like positions, velocities etc.

use bevy::{
    app::{App, FixedUpdate, Plugin, Update},
    asset::AssetApp,
    prelude::IntoSystemConfigs,
    sprite::Material2dPlugin,
    state::condition::in_state,
};

use crate::{
    assets::app_ext::DystopiaAssetAppExt,
    cosmos::{
        celestial::ShowOrbits,
        config::{CosmosStarNamesConfig, RawCosmosStarPropertiesConfig},
        mesh::{GiantBodyMaterial, OrbitMaterial, RockyBodyMaterial, StarMaterial},
    },
    schedule::state::{AssetState, GameState},
};

pub mod bundle;
pub mod celestial;
pub mod config;
pub mod gen;
pub mod mesh;
pub mod sim;

pub const ORBIT_MESH_SCALE: f32 = 1.5;
pub const ORBIT_WIDTH: f32 = 1.;

pub struct DystopiaCosmosPlugin;

impl Plugin for DystopiaCosmosPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StarMaterial>()
            .init_asset::<RockyBodyMaterial>()
            .init_asset::<GiantBodyMaterial>()
            .init_resource::<ShowOrbits>()
            .add_plugins(Material2dPlugin::<StarMaterial>::default())
            .add_plugins(Material2dPlugin::<RockyBodyMaterial>::default())
            .add_plugins(Material2dPlugin::<GiantBodyMaterial>::default())
            .add_plugins(Material2dPlugin::<OrbitMaterial>::default())
            .add_systems(
                Update,
                gen::generate_cosmos
                    .run_if(in_state(AssetState::Finish))
                    .run_if(in_state(GameState::Initialize)),
            )
            .add_systems(
                Update,
                sim::manage_orbit_visibility.run_if(in_state(GameState::Simulate)),
            )
            .add_systems(
                FixedUpdate,
                sim::update_cosmos.run_if(in_state(GameState::Simulate)),
            )
            .add_systems(
                FixedUpdate,
                (sim::sync_bodies, sim::sync_orbits)
                    .run_if(in_state(GameState::Simulate))
                    .after(sim::update_cosmos),
            )
            .add_config::<RawCosmosStarPropertiesConfig>()
            .add_config::<CosmosStarNamesConfig>();
    }
}
