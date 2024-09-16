use bevy::{
    asset::Asset,
    input::ButtonInput,
    prelude::{Deref, KeyCode, Res, ResMut, Resource},
    reflect::TypePath,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::assets::config::RawConfig;

pub use code::*;

pub const MAX_EVENT_COUNT: usize = 512;

#[derive(Asset, TypePath, Clone, Serialize, Deserialize)]
pub struct RawInputMappingConfig {
    readable_mapping: HashMap<String, usize>,
    keyboard_mapping: HashMap<KeyCode, Vec<String>>,
}

impl RawConfig for RawInputMappingConfig {
    type Processed = InputMappingConfig;

    const PATH: &'static str = "configs/user/input_mapping.json";
}

#[derive(Resource, Deref)]
pub struct InputMappingConfig(HashMap<KeyCode, Vec<usize>>);

impl From<RawInputMappingConfig> for InputMappingConfig {
    fn from(value: RawInputMappingConfig) -> Self {
        Self(
            value
                .keyboard_mapping
                .into_iter()
                .map(|(key, event)| {
                    (
                        key,
                        event
                            .into_iter()
                            .map(|readable| value.readable_mapping[&readable])
                            .collect(),
                    )
                })
                .collect(),
        )
    }
}

#[derive(Resource)]
pub struct KeyboardEventCenter([u8; MAX_EVENT_COUNT / 8]);

impl Default for KeyboardEventCenter {
    fn default() -> Self {
        Self([0; MAX_EVENT_COUNT / 8])
    }
}

impl KeyboardEventCenter {
    #[inline]
    pub fn activate(&mut self, event: usize) {
        self.0[event / 8] |= 1 << (event % 8)
    }

    #[inline]
    pub fn deactivate(&mut self, event: usize) {
        self.0[event / 8] &= !(1 << (event % 8))
    }

    #[inline]
    pub fn is_activating(&self, event: usize) -> bool {
        self.0[event / 8] & (1 << (event % 8)) != 0
    }
}

pub fn keyboard_input_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<InputMappingConfig>,
    mut center: ResMut<KeyboardEventCenter>,
) {
    keyboard.get_just_pressed().for_each(|key| {
        if let Some(events) = config.get(key) {
            events.iter().for_each(|event| {
                center.activate(*event);
            });
        }
    });

    keyboard.get_just_released().for_each(|key| {
        if let Some(events) = config.get(key) {
            events.iter().for_each(|event| {
                center.deactivate(*event);
            });
        }
    });
}

pub mod condition {
    use bevy::prelude::Res;

    use crate::input::event::KeyboardEventCenter;

    pub fn keyboard_event_activating(event: usize) -> impl FnMut(Res<KeyboardEventCenter>) -> bool {
        move |center| center.is_activating(event)
    }
}

mod code {
    pub const OPEN_COSMOS_VIEW: usize = 0;

    pub const PLAYER_MOVE_UP: usize = 100;
    pub const PLAYER_MOVE_DOWN: usize = 101;
    pub const PLAYER_MOVE_LEFT: usize = 102;
    pub const PLAYER_MOVE_RIGHT: usize = 103;
    pub const TOGGLE_CAMERA_CONTROL_OVERRIDE: usize = 104;
}
