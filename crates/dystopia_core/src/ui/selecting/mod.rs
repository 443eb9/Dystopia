use bevy::app::{App, Plugin};

pub mod body;

pub(super) struct SelectingUiPlugin;

impl Plugin for SelectingUiPlugin {
    fn build(&self, app: &mut App) {
        app.observe(body::on_target_change);
    }
}
