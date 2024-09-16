use bevy::{
    app::{App, Plugin},
    math::Vec2,
    prelude::{Component, Deref, DerefMut},
};

use crate::tuple_struct_new;

pub mod camera;
pub mod player;

pub const ISOMETRIC_VEL_FACTOR: Vec2 = Vec2 { x: 1., y: 0.5 };

pub struct DystopiaCharacterPlugin;

impl Plugin for DystopiaCharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((camera::CameraPlugin, player::PlayerPlugin));
    }
}

/// Set the move speed of an entity. This should not be modified once be inserted
/// onto an entity.
///
/// To vary speed, modify [`MoveSpeedFactor`].
#[derive(Component, Debug, Default, Deref)]
pub struct MoveSpeed(f32);
tuple_struct_new!(MoveSpeed, f32);

/// The factor multiplied to the displacement of this entity in each frame.
#[derive(Component, Debug, Deref, DerefMut)]
pub struct MoveSpeedFactor(f32);
tuple_struct_new!(MoveSpeedFactor, f32);

impl Default for MoveSpeedFactor {
    fn default() -> Self {
        Self(1.)
    }
}
