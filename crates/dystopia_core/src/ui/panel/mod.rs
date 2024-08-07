use bevy::{
    input::ButtonInput,
    prelude::{Commands, KeyCode, Query, Res, ResMut, Visibility, With},
};

use crate::{
    input::Dragable,
    ui::{sync::UiSyncWithCursor, UiStack},
};

pub mod body_data;

pub fn handle_esc_panel_close(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    on_drag: Query<(), (With<UiSyncWithCursor>, With<Dragable>)>,
    mut stack: ResMut<UiStack>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Some(panel) = stack.pop() {
            if !on_drag.contains(panel) {
                commands.entity(panel).insert(Visibility::Hidden);
            }
        }
    }
}
