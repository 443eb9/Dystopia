use bevy::{math::Vec2, prelude::Component};

#[derive(Component)]
pub struct EntityDragable;

#[derive(Component)]
pub struct EntityOnDrag {
    pub initial_cursor_pos: Vec2,
    pub initial_elem_world_pos: Vec2,
}
