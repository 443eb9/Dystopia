use bevy::app::{App, Plugin, Update};

pub mod body;

pub(super) struct SelectingUiPlugin;

impl Plugin for SelectingUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, body::handle_target_change);
    }
}
