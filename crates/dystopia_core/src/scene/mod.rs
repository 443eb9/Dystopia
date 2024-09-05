use bevy::app::{App, Plugin};

use crate::scene::transition::DystopiaTransitionPlugin;

pub mod transition;

pub struct DystopiaScenePlugin;

impl Plugin for DystopiaScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DystopiaTransitionPlugin);
    }
}
