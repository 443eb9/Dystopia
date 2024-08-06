use bevy::{
    app::{App, Plugin, Update},
    input::ButtonState,
    math::Vec2,
    prelude::{Deref, DerefMut, Event, MouseButton, Resource},
};

use crate::{
    assets::app_ext::DystopiaAssetAppExt,
    input::mapping::{KeyboardEventCenter, RawInputMappingConfig},
};

pub mod camera;
pub mod ext;
pub mod mapping;

pub struct DystopiaInputPlugin;

impl Plugin for DystopiaInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_config::<RawInputMappingConfig>()
            .add_systems(
                Update,
                (
                    camera::toggle_camera_move,
                    camera::camera_move,
                    camera::camera_zoom,
                ),
            )
            .init_resource::<KeyboardEventCenter>();
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
