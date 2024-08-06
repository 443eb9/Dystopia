use bevy::{
    app::{App, Plugin, PreUpdate},
    input::ButtonState,
    prelude::{Component, IntoSystemConfigs, MouseButton},
};

use crate::math::raycasting::scene::{SceneCursorPosition, SceneMouseClick};

pub mod scene;
pub mod ui;

pub struct DystopiaRaycastingPlugin;

impl Plugin for DystopiaRaycastingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SceneMouseClick>()
            .add_systems(
                PreUpdate,
                (
                    ui::ui_mouse_event_reset,
                    (ui::ui_mouse_hover_filterer, ui::ui_mouse_input_filterer),
                )
                    .chain(),
            )
            .init_resource::<SceneCursorPosition>();
    }
}

/// Mark an entity to be invisible to rays.
///
/// This will exclude these entities when performing raycasting.
#[derive(Component)]
pub struct RayTransparent;

#[derive(Component)]
pub struct MouseHovering;

#[derive(Component)]
pub struct MouseInput {
    pub button: MouseButton,
    pub state: ButtonState,
}
