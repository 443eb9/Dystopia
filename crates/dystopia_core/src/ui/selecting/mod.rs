use bevy::{
    app::{App, Plugin, Update},
    prelude::{in_state, IntoSystemConfigs},
};

use crate::schedule::state::GameState;

pub mod body;

pub(super) struct SelectingUiPlugin;

impl Plugin for SelectingUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            body::on_target_change.run_if(in_state(GameState::Simulate)),
        );
    }
}
