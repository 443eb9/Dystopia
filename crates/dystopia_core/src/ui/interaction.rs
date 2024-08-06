use bevy::{
    input::{ButtonInput, ButtonState},
    log::error,
    math::Vec2,
    prelude::{Component, Entity, KeyCode, MouseButton, ParallelCommands, Query, Res, With},
    ui::{Node, Style, Val},
};
use thiserror::Error;

use crate::{
    math::{raycasting::MouseInput, Axis, UiDirection},
    simulation::{CursorPosition, WindowSize},
};

#[derive(Error, Debug)]
pub enum UiPosCreationError {
    #[error("Style value conflict on {0:?}.")]
    ValueConflict(Axis),
    #[error("Style value on {0:?} is not supported.")]
    ValueNotSupported(UiDirection),
}

pub struct UiPos {
    /// Position converted to x: left, y: top
    pub converted_pos: Vec2,
    /// Original position that depends on `original_param`
    pub original_pos: Vec2,
    pub original_param: [UiDirection; 2],
}

impl UiPos {
    pub fn new(
        style: &Style,
        window_size: Vec2,
        elem_size: Vec2,
    ) -> Result<Self, UiPosCreationError> {
        let mut desc = [UiDirection::Top; 2];

        let (original_x, x) = {
            match style.left {
                Val::Auto => match style.right {
                    Val::Auto => {
                        desc[0] = UiDirection::Left;
                        (0., 0.)
                    }
                    Val::Px(px) => {
                        desc[0] = UiDirection::Right;
                        (px, window_size.x - px - elem_size.x)
                    }
                    _ => return Err(UiPosCreationError::ValueNotSupported(UiDirection::Right)),
                },
                Val::Px(px) => {
                    if !matches!(style.right, Val::Auto) {
                        return Err(UiPosCreationError::ValueConflict(Axis::X));
                    }

                    desc[0] = UiDirection::Left;
                    (px, px)
                }
                _ => return Err(UiPosCreationError::ValueNotSupported(UiDirection::Left)),
            }
        };

        let (original_y, y) = {
            match style.top {
                Val::Auto => match style.bottom {
                    Val::Auto => {
                        desc[1] = UiDirection::Top;
                        (0., 0.)
                    }
                    Val::Px(px) => {
                        desc[1] = UiDirection::Bottom;
                        (px, window_size.y - px - elem_size.y)
                    }
                    _ => return Err(UiPosCreationError::ValueNotSupported(UiDirection::Bottom)),
                },
                Val::Px(px) => {
                    if !matches!(style.bottom, Val::Auto) {
                        return Err(UiPosCreationError::ValueConflict(Axis::Y));
                    }

                    desc[1] = UiDirection::Top;
                    (px, px)
                }
                _ => return Err(UiPosCreationError::ValueNotSupported(UiDirection::Top)),
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

#[derive(Component)]
pub struct OnDrag {
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
    nodes_query
        .par_iter()
        .for_each(|(entity, node, style, input)| {
            if input.button != MouseButton::Left {
                return;
            }

            match input.state {
                ButtonState::Pressed => {
                    commands.command_scope(|mut c| {
                        c.entity(entity).insert(OnDrag {
                            initial_elem_pos: match UiPos::new(style, **window_size, node.size()) {
                                Ok(ok) => ok,
                                Err(err) => {
                                    error!("{:?}", err);
                                    return;
                                }
                            },
                            initial_cursor_pos: **cursor_pos,
                        });
                    });
                }
                ButtonState::Released => {
                    commands.command_scope(|mut c| {
                        c.entity(entity).remove::<OnDrag>();
                    });
                }
            }
        });
}

pub fn ui_drag_canceller(
    commands: ParallelCommands,
    mut nodes_query: Query<(Entity, &mut Style, &OnDrag), (With<Dragable>, With<Node>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    nodes_query
        .par_iter_mut()
        .for_each(|(entity, mut style, on_drag)| {
            match on_drag.initial_elem_pos.original_param[0] {
                UiDirection::Left => style.left = Val::Px(on_drag.initial_elem_pos.original_pos.x),
                UiDirection::Right => {
                    style.right = Val::Px(on_drag.initial_elem_pos.original_pos.x)
                }
                _ => unreachable!(),
            }

            match on_drag.initial_elem_pos.original_param[1] {
                UiDirection::Top => style.top = Val::Px(on_drag.initial_elem_pos.original_pos.y),
                UiDirection::Bottom => {
                    style.bottom = Val::Px(on_drag.initial_elem_pos.original_pos.y)
                }
                _ => unreachable!(),
            }

            commands.command_scope(|mut c| {
                c.entity(entity).remove::<OnDrag>();
            });
        });
}

pub fn ui_drag_handler(
    mut nodes_query: Query<(&mut Style, &OnDrag, &Dragable)>,
    cursor_pos: Res<CursorPosition>,
) {
    nodes_query
        .par_iter_mut()
        .for_each(|(mut style, on_drag, dragable)| {
            let offset = (**cursor_pos - on_drag.initial_cursor_pos) * dragable.constraint;

            match on_drag.initial_elem_pos.original_param[0] {
                UiDirection::Left => {
                    style.left = Val::Px(on_drag.initial_elem_pos.original_pos.x + offset.x)
                }
                UiDirection::Right => {
                    style.right = Val::Px(on_drag.initial_elem_pos.original_pos.x - offset.x)
                }
                _ => unreachable!(),
            }

            match on_drag.initial_elem_pos.original_param[1] {
                UiDirection::Top => {
                    style.top = Val::Px(on_drag.initial_elem_pos.original_pos.y + offset.y)
                }
                UiDirection::Bottom => {
                    style.bottom = Val::Px(on_drag.initial_elem_pos.original_pos.y - offset.y)
                }
                _ => unreachable!(),
            }
        });
}
