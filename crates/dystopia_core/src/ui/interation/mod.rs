use bevy::{
    app::{App, Plugin, Update},
    prelude::{in_state, IntoSystemConfigs},
};

use crate::schedule::state::GameState;

pub mod close_button;
pub mod scrollable_list;

pub struct UiInterationPlugin;

impl Plugin for UiInterationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                scrollable_list::init_structure,
                scrollable_list::handle_scroll,
                close_button::handle_button_close_click,
            )
                .run_if(in_state(GameState::Simulate)),
        );
    }
}
