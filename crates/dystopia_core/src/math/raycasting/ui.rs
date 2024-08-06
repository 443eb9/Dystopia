use std::sync::atomic::{AtomicBool, Ordering};

use bevy::{
    input::mouse::MouseButtonInput,
    prelude::{
        Entity, EventReader, EventWriter, GlobalTransform, ParallelCommands, Query, Res, ResMut,
        ViewVisibility, With, Without,
    },
    ui::Node,
};

use crate::{
    math::raycasting::{
        scene::{SceneCursorPosition, SceneMouseClick},
        MouseHovering, MouseInput, RayTransparent,
    },
    simulation::CursorPosition,
};

pub fn ui_mouse_event_reset(
    commands: ParallelCommands,
    hovering_query: Query<Entity, With<MouseHovering>>,
    input_query: Query<Entity, With<MouseInput>>,
) {
    hovering_query.par_iter().for_each(|e| {
        commands.command_scope(|mut c| {
            c.entity(e).remove::<MouseHovering>();
        });
    });

    input_query.par_iter().for_each(|e| {
        commands.command_scope(|mut c| {
            c.entity(e).remove::<MouseInput>();
        });
    });
}

pub fn ui_mouse_hover_filterer(
    commands: ParallelCommands,
    cursor_pos: Res<CursorPosition>,
    nodes_query: Query<(Entity, &Node, &GlobalTransform, &ViewVisibility), Without<RayTransparent>>,
    mut scene_cursor_pos: ResMut<SceneCursorPosition>,
) {
    let blocked = AtomicBool::default();

    nodes_query
        .par_iter()
        .for_each(|(entity, node, transform, vis)| {
            if vis.get() && node.logical_rect(transform).contains(**cursor_pos) {
                blocked.store(true, Ordering::Relaxed);

                commands.command_scope(|mut c| {
                    c.entity(entity).insert(MouseHovering);
                });
            }
        });

    if !blocked.load(Ordering::Relaxed) {
        **scene_cursor_pos = Some(**cursor_pos);
    }
}

pub fn ui_mouse_input_filterer(
    commands: ParallelCommands,
    cursor_pos: Res<CursorPosition>,
    nodes_query: Query<(Entity, &Node, &GlobalTransform, &ViewVisibility), Without<RayTransparent>>,
    mut mouse: EventReader<MouseButtonInput>,
    mut event: EventWriter<SceneMouseClick>,
) {
    for ev in mouse.read() {
        let blocked = AtomicBool::default();

        nodes_query
            .par_iter()
            .for_each(|(entity, node, transform, vis)| {
                if vis.get() && node.logical_rect(transform).contains(**cursor_pos) {
                    blocked.store(true, Ordering::Relaxed);

                    commands.command_scope(|mut c| {
                        c.entity(entity).insert(MouseInput {
                            button: ev.button,
                            state: ev.state,
                        });
                    });
                }
            });

        if !blocked.load(Ordering::Relaxed) {
            event.send(SceneMouseClick {
                cursor_pos: **cursor_pos,
                button: ev.button,
                state: ev.state,
            });
        }
    }
}
