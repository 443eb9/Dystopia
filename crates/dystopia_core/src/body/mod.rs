use bevy::{
    app::{App, Plugin},
    prelude::Resource,
};

use crate::cosmos::celestial::{BodyIndex, BodyTilemap};

pub struct DystopiaBodyPlugin;

impl Plugin for DystopiaBodyPlugin {
    fn build(&self, app: &mut App) {}
}

/// The body currently focusing on. Not necessarily exist.
///
/// Inserted by
/// [`handle_body_focusing`](crate::scene::transition::cosmos_view::handle_body_focusing).
#[derive(Resource)]
pub struct FocusingOn {
    pub body: BodyIndex,
    pub tilemap: BodyTilemap,
}
