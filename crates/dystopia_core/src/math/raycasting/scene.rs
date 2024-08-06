use bevy::{
    input::ButtonState,
    math::Vec2,
    prelude::{Deref, DerefMut, Event, MouseButton, Res, Resource},
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SceneCursorPosition(Option<Vec2>);

#[derive(Event)]
pub struct SceneMouseClick {
    pub cursor_pos: Vec2,
    pub button: MouseButton,
    pub state: ButtonState,
}

pub fn scene_mouse_hover(cursor_pos: Res<SceneCursorPosition>) {}
