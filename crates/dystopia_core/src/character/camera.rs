use bevy::{
    app::{App, Plugin, Update},
    input::{
        mouse::{MouseScrollUnit, MouseWheel},
        ButtonState,
    },
    math::{FloatExt, Vec2},
    prelude::{
        Commands, Component, Entity, EventReader, Has, Local, MouseButton, Query, Res, ResMut,
        State, Transform, With,
    },
    time::{Real, Time},
};

use crate::{
    character::{MoveSpeed, MoveSpeedFactor},
    input::{
        event::{
            KeyboardEventCenter, PLAYER_MOVE_DOWN, PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT,
            PLAYER_MOVE_UP, TOGGLE_CAMERA_CONTROL_OVERRIDE,
        },
        scene::EntityOnDrag,
        SceneCursorPosition, SceneMouseInput,
    },
    schedule::state::SceneState,
    sim::{MainCamera, ViewScale},
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (toggle_camera_move, camera_move, camera_zoom));
    }
}

#[derive(Component)]
pub struct CameraBehavior {
    pub zoom_ratio: f32,
    pub zoom_max: f32,
    pub zoom_min: f32,
    pub zoom_smooth: f32,
}

fn toggle_camera_move(
    mut commands: Commands,
    main_camera: Query<Entity, With<MainCamera>>,
    mut mouse_click: EventReader<SceneMouseInput>,
) {
    for click in mouse_click.read() {
        if click.button == MouseButton::Right {
            match click.state {
                ButtonState::Pressed => {
                    commands.entity(main_camera.single()).insert(EntityOnDrag {
                        initial_cursor_pos: Default::default(),
                        initial_elem_world_pos: Default::default(),
                    })
                }
                ButtonState::Released => commands
                    .entity(main_camera.single())
                    .remove::<EntityOnDrag>(),
            };
        }
    }
}

fn camera_move(
    mut main_camera: Query<
        (
            &mut Transform,
            &MoveSpeed,
            &MoveSpeedFactor,
            Has<EntityOnDrag>,
        ),
        With<MainCamera>,
    >,
    cursor_pos: Res<SceneCursorPosition>,
    mut last_pos: Local<Option<Vec2>>,
    current_zoom: Res<ViewScale>,
    event_center: Res<KeyboardEventCenter>,
    time: Res<Time<Real>>,
    scene_state: Res<State<SceneState>>,
) {
    let (mut transform, speed, factor, is_dragging) = main_camera.single_mut();

    let mut vel = Vec2::ZERO;
    if event_center.is_activating(PLAYER_MOVE_UP) {
        vel.y += 1.;
    }
    if event_center.is_activating(PLAYER_MOVE_DOWN) {
        vel.y -= 1.;
    }
    if event_center.is_activating(PLAYER_MOVE_LEFT) {
        vel.x -= 1.;
    }
    if event_center.is_activating(PLAYER_MOVE_RIGHT) {
        vel.x += 1.;
    }

    if (matches!(scene_state.get(), SceneState::CosmosView) && vel != Vec2::ZERO)
        || event_center.is_activating(TOGGLE_CAMERA_CONTROL_OVERRIDE)
    {
        transform.translation +=
            (time.delta_seconds() * **speed * **factor * **current_zoom * vel).extend(0.);
    } else if let Some(cursor_pos) = **cursor_pos {
        if !is_dragging {
            *last_pos = None;
            return;
        }

        let mut delta = (cursor_pos - last_pos.unwrap_or(cursor_pos)) * **current_zoom;
        delta.x *= -1.;
        transform.translation += delta.extend(0.);
        *last_pos = Some(cursor_pos);
    } else {
        *last_pos = None;
        return;
    }
}

fn camera_zoom(
    mut main_camera: Query<&CameraBehavior, With<MainCamera>>,
    mut scroll: EventReader<MouseWheel>,
    mut current_zoom: ResMut<ViewScale>,
    mut maybe_target_zoom: Local<Option<f32>>,
    time: Res<Time<Real>>,
) {
    let behavior = main_camera.single_mut();

    if let Some(target_zoom) = *maybe_target_zoom {
        **current_zoom =
            current_zoom.lerp(target_zoom, behavior.zoom_smooth * time.delta_seconds());

        if (**current_zoom - target_zoom).abs() < 0.001 {
            *maybe_target_zoom = None;
        }
    }

    if scroll.is_empty() {
        return;
    }

    if maybe_target_zoom.is_none() {
        *maybe_target_zoom = Some(**current_zoom);
    }

    let target_zoom = maybe_target_zoom.as_mut().unwrap();

    for scroll in scroll.read() {
        let delta = match scroll.unit {
            MouseScrollUnit::Line => scroll.y * 20.,
            MouseScrollUnit::Pixel => scroll.y,
        };

        *target_zoom *= 1. - delta * behavior.zoom_ratio;
    }

    *target_zoom = target_zoom.clamp(behavior.zoom_min, behavior.zoom_max);
}
