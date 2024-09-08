use bevy::{
    app::{App, Plugin, Update},
    prelude::{in_state, IntoSystemConfigs},
};

use crate::schedule::state::SceneState;

pub mod load;
pub mod save;

pub struct DystopiaSerdePlugin;

impl Plugin for DystopiaSerdePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            load::init_tilemap_when_body_clicked.run_if(in_state(SceneState::CosmosView)),
        );
    }
}
