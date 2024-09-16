use bevy::{
    app::{App, Plugin, Update},
    input::{
        mouse::{MouseScrollUnit, MouseWheel},
        ButtonState,
    },
    math::{FloatExt, Vec2},
    prelude::{
        Commands, Component, Entity, EventReader, Local, MouseButton, Query, Res, ResMut,
        Transform, With,
    },
    time::{Real, Time},
};

use crate::{
    input::{scene::EntityOnDrag, SceneCursorPosition, SceneMouseInput},
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
    mut main_camera: Query<&mut Transform, (With<MainCamera>, With<EntityOnDrag>)>,
    cursor_pos: Res<SceneCursorPosition>,
    mut last_pos: Local<Option<Vec2>>,
    current_zoom: Res<ViewScale>,
) {
    let (Some(cursor_pos), Ok(mut transform)) = (**cursor_pos, main_camera.get_single_mut()) else {
        *last_pos = None;
        return;
    };

    let mut delta = (cursor_pos - last_pos.unwrap_or(cursor_pos)) * **current_zoom;
    delta.x *= -1.;
    transform.translation += delta.extend(0.);
    *last_pos = Some(cursor_pos);
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
