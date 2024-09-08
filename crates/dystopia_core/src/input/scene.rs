use std::time::Instant;

use avian2d::prelude::{Collider, Position, Rotation};
use bevy::{
    math::Vec2,
    prelude::{
        Camera, Commands, Component, Entity, EventReader, GlobalTransform, ParallelCommands, Query,
        Res, ViewVisibility, With, Without,
    },
};

use crate::{
    input::{
        MouseClickCounter, MouseHovering, MouseInput, MouseMultiClickCooldown, RayTransparent,
        SceneCursorPosition, SceneMouseInput,
    },
    sim::MainCamera,
};

#[derive(Component)]
pub struct EntityOnDrag {
    pub initial_cursor_pos: Vec2,
    pub initial_elem_world_pos: Vec2,
}

pub fn scene_mouse_hover(
    commands: ParallelCommands,
    cursor_pos: Res<SceneCursorPosition>,
    colliders_query: Query<
        (Entity, &Collider, &Position, &Rotation, &ViewVisibility),
        Without<RayTransparent>,
    >,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, transform) = main_camera.single();
    let Some(cursor_pos) = (**cursor_pos).and_then(|p| camera.viewport_to_world_2d(transform, p))
    else {
        return;
    };

    colliders_query
        .par_iter()
        .for_each(|(entity, collider, position, rotation, vis)| {
            if vis.get() && collider.contains_point(*position, *rotation, cursor_pos) {
                commands.command_scope(|mut c| {
                    c.entity(entity).insert(MouseHovering);
                });
            }
        });
}

pub(super) fn scene_mouse_click(
    mut commands: Commands,
    colliders_query: Query<
        (
            Entity,
            Option<&MouseMultiClickCooldown>,
            Option<&MouseClickCounter>,
        ),
        With<MouseHovering>,
    >,
    mut event: EventReader<SceneMouseInput>,
) {
    for ev in event.read() {
        for (entity, maybe_cooldown, maybe_counter) in &colliders_query {
            let mut entity = commands.entity(entity);
            entity.insert(MouseInput {
                button: ev.button,
                state: ev.state,
            });

            if !ev.state.is_pressed() {
                continue;
            }

            if maybe_cooldown.is_none() || maybe_cooldown.is_some_and(|c| c.button != ev.button) {
                entity.insert(MouseMultiClickCooldown {
                    button: ev.button,
                    latest_click: Instant::now(),
                });
            }

            if maybe_counter.is_none() {
                entity.insert(MouseClickCounter::default());
            }
        }
    }
}
