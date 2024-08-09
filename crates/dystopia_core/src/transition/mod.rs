use bevy::app::{App, Plugin};

use crate::transition::cosmos_to_body::CosmosToBodyTransitionPlugin;

pub mod cosmos_to_body;

pub struct DystopiaTransitionPlugin;

impl Plugin for DystopiaTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CosmosToBodyTransitionPlugin);
    }
}
