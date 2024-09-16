//! Transition between cosmos view to body view
//! - cosmos view: Interface where you will see bodies rotate around.
//! - body view: The tilemap of a specific body.

use bevy::{
    core::Name,
    prelude::{
        Commands, EventWriter, NextState, Query, Res, ResMut, Transform, Visibility, With, Without,
    },
};

use crate::{
    body::FocusingOn,
    character::player::PlayerAction,
    cosmos::celestial::{BodyColor, BodyIndex, BodyTilemap},
    impl_transition_plugin,
    input::event::{condition::keyboard_event_activating, OPEN_COSMOS_VIEW},
    map::tilemap::TilemapStorage,
    scene::transition::CameraRecoverTransform,
    schedule::state::SceneState,
    sim::{MainCamera, ViewScale},
    ui::panel::{
        scene_title::{LSceneTitle, SceneTitle, SceneTitleChange},
        PanelTargetChange,
    },
};

impl_transition_plugin!(
    FocusingBodyTransitionPlugin,
    SceneState::FocusingBody,
    focus_body,
    handle_cosmos_view_entering.run_if(keyboard_event_activating(OPEN_COSMOS_VIEW)),
    unfocus_body
);

fn focus_body(
    mut commands: Commands,
    bodies_query: Query<(&CameraRecoverTransform, &BodyTilemap, &Name, &BodyColor)>,
    focusing_on: Res<FocusingOn>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<BodyIndex>)>,
    mut view_scale: ResMut<ViewScale>,
    mut target_change: EventWriter<PanelTargetChange<SceneTitle, SceneTitleChange>>,
    mut player_action: EventWriter<PlayerAction>,
) {
    let (recover_transl, tilemap, name, color) = bodies_query.get(focusing_on.entity).unwrap();

    commands.entity(**tilemap).insert(Visibility::Inherited);
    target_change.send(PanelTargetChange::some(SceneTitleChange {
        title: LSceneTitle::FocusingBody,
        name: Some((name.to_string(), **color)),
    }));
    recover_transl.recover(&mut camera_query.single_mut(), &mut view_scale);
    player_action.send(PlayerAction::ChangeVisibility(Visibility::Inherited));
}

fn handle_cosmos_view_entering(mut scene_state: ResMut<NextState<SceneState>>) {
    scene_state.set(SceneState::CosmosView);
}

fn unfocus_body(
    mut commands: Commands,
    mut tilemaps_query: Query<&mut Visibility, With<TilemapStorage>>,
    focusing_on: Res<FocusingOn>,
    mut player_action: EventWriter<PlayerAction>,
) {
    *tilemaps_query.get_mut(*focusing_on.tilemap).unwrap() = Visibility::Hidden;
    commands.remove_resource::<FocusingOn>();
    player_action.send(PlayerAction::ChangeVisibility(Visibility::Hidden));
}
