use bevy::prelude::{Commands, MouseButton, NextState, Query, ResMut, Visibility, With};

use crate::{
    body::FocusingOn,
    cosmos::celestial::{BodyIndex, BodyTilemap},
    impl_transition_plugin,
    input::{MouseClickCounter, MouseInput},
    schedule::state::SceneState,
};

impl_transition_plugin!(
    CosmosViewSceneTransitionPlugin,
    SceneState::CosmosView,
    show_bodies,
    handle_body_focusing,
    hide_bodies
);

fn hide_bodies(mut bodies_query: Query<&mut Visibility, With<BodyIndex>>) {
    dbg!();
    bodies_query
        .par_iter_mut()
        .for_each(|mut vis| *vis = Visibility::Hidden);
}

fn show_bodies(mut bodies_query: Query<&mut Visibility, With<BodyIndex>>) {
    bodies_query
        .par_iter_mut()
        .for_each(|mut vis| *vis = Visibility::Inherited);
}

pub fn handle_body_focusing(
    mut commands: Commands,
    mut scene_state: ResMut<NextState<SceneState>>,
    double_clicked_query: Query<(&BodyIndex, &BodyTilemap, &MouseInput, &MouseClickCounter)>,
) {
    for (body, tilemap, input, counter) in &double_clicked_query {
        if input.button != MouseButton::Left || **counter != 2 {
            continue;
        }

        commands.insert_resource(FocusingOn {
            body: *body,
            tilemap: *tilemap,
        });
        scene_state.set(SceneState::FocusingBody);
    }
}
