use bevy::{
    app::{App, Plugin},
    prelude::Resource,
};
use rand::rngs::StdRng;

pub struct DystopiaSimulationPlugin;

impl Plugin for DystopiaSimulationPlugin {
    fn build(&self, app: &mut App) {}
}

/// The RNG used across the entire game.
///
/// For wold that is generated, this RNG will be inserted when generating
/// cosmos, and for those are loaded, this will be loaded from the save.
#[derive(Resource)]
pub struct GlobalRng(pub StdRng);
