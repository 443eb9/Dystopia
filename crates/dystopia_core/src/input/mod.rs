use bevy::{
    app::{App, Plugin, PreUpdate, Update},
    input::ButtonState,
    math::Vec2,
    prelude::{
        Commands, Component, Deref, DerefMut, Entity, Event, IntoSystemConfigs, MouseButton, Query,
        Resource, With,
    },
};

use crate::{
    assets::app_ext::DystopiaAssetAppExt,
    input::mapping::{KeyboardEventCenter, RawInputMappingConfig},
};

pub mod camera;
pub mod ext;
pub mod mapping;
pub mod scene;
pub mod ui;

pub struct DystopiaInputPlugin;

impl Plugin for DystopiaInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_config::<RawInputMappingConfig>()
            .add_event::<SceneMouseClick>()
            .add_systems(
                PreUpdate,
                (
                    mouse_event_reset,
                    ui::ui_mouse_hover_filterer,
                    (ui::ui_mouse_input_filterer, scene::scene_mouse_hover),
                    scene::scene_mouse_click,
                )
                    .chain(),
            )
            .add_systems(Update, (ui::ui_drag_marker, ui::ui_drag_canceller).chain())
            .add_systems(
                Update,
                (
                    camera::toggle_camera_move,
                    camera::camera_move,
                    camera::camera_zoom,
                ),
            )
            .init_resource::<KeyboardEventCenter>()
            .init_resource::<SceneCursorPosition>();
    }
}

pub fn mouse_event_reset(
    mut commands: Commands,
    hovering_query: Query<Entity, With<MouseHovering>>,
    input_query: Query<Entity, With<MouseInput>>,
) {
    hovering_query.iter().for_each(|entity| {
        commands.entity(entity).remove::<MouseHovering>();
    });

    input_query.iter().for_each(|entity| {
        commands.entity(entity).remove::<MouseInput>();
    });
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

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SceneCursorPosition(Option<Vec2>);

#[derive(Event)]
pub struct SceneMouseClick {
    pub cursor_pos: Vec2,
    pub button: MouseButton,
    pub state: ButtonState,
}
