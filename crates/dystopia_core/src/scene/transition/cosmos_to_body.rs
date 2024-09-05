//! Transition between cosmos view to body view
//! - cosmos view: Interface where you will see bodies rotate around.
//! - body view: The tilemap of a specific body.

use std::ops::Range;

use bevy::{
    app::{App, Plugin},
    prelude::{Commands, Entity, Query, Transform},
};

use crate::{cosmos::celestial::BodyTilemap, map::tilemap::TilemapTint};

pub(super) struct CosmosToBodyTransitionPlugin;

impl Plugin for CosmosToBodyTransitionPlugin {
    fn build(&self, app: &mut App) {}
}

const TRANSITION_RANGE: Range<f32> = 0.4..0.6;

fn transition(
    mut commands: Commands,
    bodies_query: Query<(Entity, &Transform, Option<&BodyTilemap>)>,
    tilemaps_query: Query<&mut TilemapTint>,
) {
}
