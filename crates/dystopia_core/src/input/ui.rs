use bevy::{
    input::{mouse::MouseButtonInput, ButtonInput, ButtonState},
    log::error,
    math::Vec2,
    prelude::{
        Commands, Component, Entity, EventReader, EventWriter, GlobalTransform, KeyCode,
        MouseButton, ParallelCommands, Query, Res, ResMut, ViewVisibility, With, Without,
    },
    ui::{Node, Style, Val},
};
use thiserror::Error;

use crate::{
    input::{
        Dragable, MouseHovering, MouseInput, RayTransparent, SceneCursorPosition, SceneMouseClick,
    },
    math::{Axis, Direction},
    simulation::{CursorPosition, WindowSize},
};

pub fn ui_mouse_event_reset(
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

pub fn ui_mouse_hover_filterer(
    mut commands: Commands,
    cursor_pos: Res<CursorPosition>,
    nodes_query: Query<(Entity, &Node, &GlobalTransform, &ViewVisibility), Without<RayTransparent>>,
    mut scene_cursor_pos: ResMut<SceneCursorPosition>,
) {
    let Some(cursor_pos) = **cursor_pos else {
        return;
    };

    let mut blocked = false;

    for (entity, node, transform, vis) in &nodes_query {
        if vis.get() && node.logical_rect(transform).contains(cursor_pos) {
            blocked = true;
            commands.entity(entity).insert(MouseHovering);
        }
    }

    if !blocked {
        **scene_cursor_pos = Some(cursor_pos);
    }
}

pub fn ui_mouse_input_filterer(
    mut commands: Commands,
    cursor_pos: Res<CursorPosition>,
    nodes_query: Query<Entity, (With<MouseHovering>, With<Node>)>,
    mut mouse: EventReader<MouseButtonInput>,
    mut event: EventWriter<SceneMouseClick>,
) {
    let Some(cursor_pos) = **cursor_pos else {
        return;
    };

    for ev in mouse.read() {
        if nodes_query.is_empty() {
            event.send(SceneMouseClick {
                cursor_pos,
                button: ev.button,
                state: ev.state,
            });
        } else {
            nodes_query.iter().for_each(|entity| {
                commands.entity(entity).insert(MouseInput {
                    button: ev.button,
                    state: ev.state,
                });
            });
        }
    }
}

#[derive(Error, Debug)]
pub enum UiPosCreationError {
    #[error("Style value conflict on {0:?}.")]
    ValueConflict(Axis),
    #[error("Style value on {0:?} is not supported.")]
    ValueNotSupported(Direction),
}

pub struct UiPos {
    /// Position converted to x: left, y: top
    pub converted_pos: Vec2,
    /// Original position that depends on `original_param`
    pub original_pos: Vec2,
    pub original_param: [Direction; 2],
}

impl UiPos {
    pub fn new(
        style: &Style,
        window_size: Vec2,
        elem_size: Vec2,
    ) -> Result<Self, UiPosCreationError> {
        let mut desc = [Direction::Up; 2];

        let (original_x, x) = {
            match style.left {
                Val::Auto => match style.right {
                    Val::Auto => {
                        desc[0] = Direction::Left;
                        (0., 0.)
                    }
                    Val::Px(px) => {
                        desc[0] = Direction::Right;
                        (px, window_size.x - px - elem_size.x)
                    }
                    _ => return Err(UiPosCreationError::ValueNotSupported(Direction::Right)),
                },
                Val::Px(px) => {
                    if !matches!(style.right, Val::Auto) {
                        return Err(UiPosCreationError::ValueConflict(Axis::X));
                    }

                    desc[0] = Direction::Left;
                    (px, px)
                }
                _ => return Err(UiPosCreationError::ValueNotSupported(Direction::Left)),
            }
        };

        let (original_y, y) = {
            match style.top {
                Val::Auto => match style.bottom {
                    Val::Auto => {
                        desc[1] = Direction::Up;
                        (0., 0.)
                    }
                    Val::Px(px) => {
                        desc[1] = Direction::Down;
                        (px, window_size.y - px - elem_size.y)
                    }
                    _ => return Err(UiPosCreationError::ValueNotSupported(Direction::Down)),
                },
                Val::Px(px) => {
                    if !matches!(style.bottom, Val::Auto) {
                        return Err(UiPosCreationError::ValueConflict(Axis::Y));
                    }

                    desc[1] = Direction::Up;
                    (px, px)
                }
                _ => return Err(UiPosCreationError::ValueNotSupported(Direction::Up)),
            }
        };

        Ok(Self {
            converted_pos: Vec2 { x, y },
            original_pos: Vec2 {
                x: original_x,
                y: original_y,
            },
            original_param: desc,
        })
    }
}

#[derive(Component)]
pub struct UiOnDrag {
    /// `x: style.top`, `y: style.left`
    ///
    /// in [`Val::Px`](bevy::ui::Val::Px)
    pub initial_elem_pos: UiPos,
    pub initial_cursor_pos: Vec2,
}

pub fn ui_drag_marker(
    commands: ParallelCommands,
    nodes_query: Query<(Entity, &Node, &Style, &MouseInput), With<Dragable>>,
    cursor_pos: Res<CursorPosition>,
    window_size: Res<WindowSize>,
) {
    let Some(cursor_pos) = **cursor_pos else {
        return;
    };

    nodes_query
        .par_iter()
        .for_each(|(entity, node, style, input)| {
            if input.button != MouseButton::Left {
                return;
            }

            match input.state {
                ButtonState::Pressed => {
                    commands.command_scope(|mut c| {
                        c.entity(entity).insert(UiOnDrag {
                            initial_elem_pos: match UiPos::new(style, **window_size, node.size()) {
                                Ok(ok) => ok,
                                Err(err) => {
                                    error!("{:?}", err);
                                    return;
                                }
                            },
                            initial_cursor_pos: cursor_pos,
                        });
                    });
                }
                ButtonState::Released => {
                    commands.command_scope(|mut c| {
                        c.entity(entity).remove::<UiOnDrag>();
                    });
                }
            }
        });
}

pub fn ui_drag_canceller(
    commands: ParallelCommands,
    mut nodes_query: Query<(Entity, &mut Style, &UiOnDrag), (With<Dragable>, With<Node>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    nodes_query
        .par_iter_mut()
        .for_each(|(entity, mut style, on_drag)| {
            match on_drag.initial_elem_pos.original_param[0] {
                Direction::Left => style.left = Val::Px(on_drag.initial_elem_pos.original_pos.x),
                Direction::Right => style.right = Val::Px(on_drag.initial_elem_pos.original_pos.x),
                _ => unreachable!(),
            }

            match on_drag.initial_elem_pos.original_param[1] {
                Direction::Up => style.top = Val::Px(on_drag.initial_elem_pos.original_pos.y),
                Direction::Down => style.bottom = Val::Px(on_drag.initial_elem_pos.original_pos.y),
                _ => unreachable!(),
            }

            commands.command_scope(|mut c| {
                c.entity(entity).remove::<UiOnDrag>();
            });
        });
}

pub fn ui_drag_handler(
    mut commands: Commands,
    mut nodes_query: Query<(Entity, &mut Style, &UiOnDrag, &Dragable, &ViewVisibility)>,
    cursor_pos: Res<CursorPosition>,
) {
    let Some(cursor_pos) = **cursor_pos else {
        return;
    };

    for (entity, mut style, on_drag, dragable, vis) in &mut nodes_query {
        if !vis.get() {
            commands.entity(entity).remove::<UiOnDrag>();
            return;
        }

        let offset = (cursor_pos - on_drag.initial_cursor_pos) * dragable.constraint;

        match on_drag.initial_elem_pos.original_param[0] {
            Direction::Left => {
                style.left = Val::Px(on_drag.initial_elem_pos.original_pos.x + offset.x)
            }
            Direction::Right => {
                style.right = Val::Px(on_drag.initial_elem_pos.original_pos.x - offset.x)
            }
            _ => unreachable!(),
        }

        match on_drag.initial_elem_pos.original_param[1] {
            Direction::Up => {
                style.top = Val::Px(on_drag.initial_elem_pos.original_pos.y + offset.y)
            }
            Direction::Down => {
                style.bottom = Val::Px(on_drag.initial_elem_pos.original_pos.y - offset.y)
            }
            _ => unreachable!(),
        }
    }
}
