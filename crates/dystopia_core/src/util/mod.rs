use bevy::app::{App, Plugin};

pub mod alpha;
pub mod chunking;
pub mod macros;

pub struct DystopiaUtilPlugin;

impl Plugin for DystopiaUtilPlugin {
    fn build(&self, app: &mut App) {}
}
