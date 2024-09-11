use bevy::{
    app::{App, Plugin, Update},
    prelude::{in_state, IntoSystemConfigs},
};

use crate::{
    cosmos::celestial::BodyIndex,
    schedule::state::GameState,
    ui::{interation::body_focus_button::BodyFocusButton, update::UpdatablePlugin},
};

pub mod body_focus_button;
pub mod close_button;
pub mod scrollable_list;

pub struct UiInterationPlugin;

impl Plugin for UiInterationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UpdatablePlugin::<BodyFocusButton, Option<BodyIndex>>::default())
            .add_systems(
                Update,
                (
                    scrollable_list::init_structure,
                    scrollable_list::handle_scroll,
                    close_button::handle_button_close_click,
                    body_focus_button::handle_body_focus_click,
                )
                    .run_if(in_state(GameState::Simulate)),
            );
    }
}
