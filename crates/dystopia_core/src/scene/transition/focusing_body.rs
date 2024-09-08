//! Transition between cosmos view to body view
//! - cosmos view: Interface where you will see bodies rotate around.
//! - body view: The tilemap of a specific body.

use bevy::prelude::{Commands, Entity, NextState, Query, Res, ResMut, Transform, Visibility, With};

use crate::{
    body::FocusingOn,
    cosmos::celestial::{BodyIndex, BodyTilemap},
    impl_transition_plugin,
    input::{event::condition::keyboard_event_activating, event::OPEN_COSMOS_VIEW},
    map::tilemap::TilemapStorage,
    schedule::state::SceneState,
};

impl_transition_plugin!(
    FocusingBodyTransitionPlugin,
    SceneState::FocusingBody,
    show_tilemap,
    handle_cosmos_view_entering.run_if(keyboard_event_activating(OPEN_COSMOS_VIEW)),
    defocus_body
);

fn show_tilemap(
    mut commands: Commands,
    bodies_query: Query<(Entity, &Transform, &BodyIndex, Option<&BodyTilemap>)>,
) {
    for (entity, transform, index, maybe_tilemap) in &bodies_query {
        if let Some(tilemap) = maybe_tilemap {
            commands.entity(**tilemap).insert(Visibility::Inherited);
            continue;
        }

        // TODO load tilemap or generate
    }
}

fn defocus_body(
    mut commands: Commands,
    mut tilemaps_query: Query<&mut Visibility, With<TilemapStorage>>,
    focusing_on: Res<FocusingOn>,
) {
    *tilemaps_query.get_mut(*focusing_on.tilemap).unwrap() = Visibility::Hidden;
    commands.remove_resource::<FocusingOn>();
}

fn handle_cosmos_view_entering(mut scene_state: ResMut<NextState<SceneState>>) {
    scene_state.set(SceneState::CosmosView);
}
