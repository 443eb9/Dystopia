use bevy::app::{App, Plugin};
use rand::rngs::StdRng;

pub struct InfGdnSimulationPlugin;

impl Plugin for InfGdnSimulationPlugin {
    fn build(&self, app: &mut App) {}
}

/// The RNG used across the entire game.
/// 
/// For wold that is generated, this RNG will be inserted when generating
/// cosmos, and for those are loaded, this will be loaded from the save.
pub struct GlobalRng(pub StdRng);
