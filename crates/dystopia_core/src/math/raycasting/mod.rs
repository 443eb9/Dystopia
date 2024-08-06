use bevy::{
    app::{App, Plugin, PreUpdate, Update},
    input::ButtonState,
    math::Vec2,
    prelude::{Component, IntoSystemConfigs, MouseButton},
};

use crate::input::{SceneCursorPosition, SceneMouseClick};

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
            .add_systems(
                Update,
                (
                    ui::ui_drag_marker,
                    ui::ui_drag_handler,
                    ui::ui_drag_canceller,
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

#[derive(Component)]
pub struct Dragable {
    pub button: MouseButton,
    pub constraint: Vec2,
}

impl Default for Dragable {
    fn default() -> Self {
        Self {
            button: MouseButton::Left,
            constraint: Vec2::ONE,
        }
    }
}

impl Dragable {
    pub fn left_btn_x_only() -> Self {
        Self {
            button: MouseButton::Left,
            constraint: Vec2::X,
        }
    }

    pub fn left_btn_y_only() -> Self {
        Self {
            button: MouseButton::Left,
            constraint: Vec2::Y,
        }
    }
}
