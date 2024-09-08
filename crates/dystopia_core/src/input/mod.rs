use std::time::Instant;

use bevy::{
    app::{App, Plugin, PreUpdate, Update},
    input::ButtonState,
    math::Vec2,
    prelude::{
        in_state, Commands, Component, Deref, DerefMut, Entity, Event, IntoSystemConfigs,
        MouseButton, Query, Resource, With,
    },
};

use crate::{
    assets::app_ext::DystopiaAssetAppExt,
    input::event::{KeyboardEventCenter, RawInputMappingConfig},
    schedule::state::ProcessState,
};

pub mod camera;
pub mod event;
pub mod ext;
pub mod scene;
pub mod ui;

pub struct DystopiaInputPlugin;

impl Plugin for DystopiaInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_config::<RawInputMappingConfig>()
            .add_event::<SceneMouseInput>()
            .add_systems(
                PreUpdate,
                (
                    mouse_event_reset,
                    ui::ui_mouse_hover_filterer,
                    (ui::ui_mouse_input_filterer, scene::scene_mouse_hover),
                    scene::scene_mouse_click,
                    mouse_cooldown_counter_handler,
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
            .add_systems(
                Update,
                event::keyboard_input_handler.run_if(in_state(ProcessState::InGame)),
            )
            .init_resource::<KeyboardEventCenter>()
            .init_resource::<SceneCursorPosition>();
    }
}

fn mouse_event_reset(
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

fn mouse_cooldown_counter_handler(
    mut commands: Commands,
    mut multi_click_query: Query<(
        Entity,
        &MouseMultiClickCooldown,
        &mut MouseClickCounter,
        Option<&MouseInput>,
    )>,
) {
    for (entity, cooldown, mut counter, maybe_input) in &mut multi_click_query {
        if cooldown.latest_click.elapsed().as_secs_f32() > MULTI_CLICK_INTERVAL {
            let mut entity = commands.entity(entity);
            entity.remove::<MouseMultiClickCooldown>();

            if maybe_input.is_none() {
                entity.remove::<MouseClickCounter>();
            }
        }

        if maybe_input.is_some_and(|x| x.button == cooldown.button && x.state.is_pressed()) {
            **counter += 1;
        }
    }
}

pub const MULTI_CLICK_INTERVAL: f32 = 0.3;

/// Mark an entity to be invisible to rays.
///
/// This will exclude these entities when performing raycasting.
#[derive(Component)]
pub struct RayTransparent;

/// Assigned by [`ui_mouse_hover_filterer`](ui::ui_mouse_hover_filterer)
/// and [`scene_mouse_hover`](scene::scene_mouse_hover). Means the cursor
/// is hovering over this entity.
#[derive(Component)]
pub struct MouseHovering;

/// Assigned by [`ui_mouse_input_filterer`](ui::ui_mouse_input_filterer)
/// and [`scene_mouse_click`](scene::scene_mouse_click). Means this entity
/// is clicked by some mouse key.
#[derive(Component)]
pub struct MouseInput {
    pub button: MouseButton,
    pub state: ButtonState,
}

#[derive(Component)]
pub(super) struct MouseMultiClickCooldown {
    pub button: MouseButton,
    pub latest_click: Instant,
}

/// Counts the multi click count. This is valid only if you're detecting pressing
/// buttons. Releasing button won't affect this.
#[derive(Component, Default, Deref, DerefMut)]
pub struct MouseClickCounter(u8);

/// Mark an entity able to be dragged.
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

/// Being different to [`CursorPosition`](crate::simulation::CursorPosition),
/// this value is only [`Some`](Option::Some) when the cursor is not over any
/// UI, or the cursor is over some UI, but the UI is [`RayTransparent`].
#[derive(Resource, Default, Deref, DerefMut)]
pub struct SceneCursorPosition(Option<Vec2>);

/// An event fired by [`ui_mouse_input_filterer`](ui::ui_mouse_input_filterer),
/// which indicates that player clicked on some where not covered by UI, or covered
/// by UI with [`RayTransparent`] component.
#[derive(Event)]
pub struct SceneMouseInput {
    pub cursor_pos: Vec2,
    pub button: MouseButton,
    pub state: ButtonState,
}
