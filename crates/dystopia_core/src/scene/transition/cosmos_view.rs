use bevy::prelude::{
    Commands, EventWriter, MouseButton, NextState, Query, Res, ResMut, Visibility, With,
};

use crate::{
    body::FocusingOn,
    cosmos::celestial::{BodyIndex, BodyTilemap},
    impl_transition_plugin,
    input::{MouseClickCounter, MouseInput},
    schedule::state::SceneState,
    ui::panel::{body_data::BodyDataPanel, PanelTargetChange},
};

impl_transition_plugin!(
    CosmosViewSceneTransitionPlugin,
    SceneState::CosmosView,
    enter_cosmos_view,
    handle_body_focusing,
    exit_cosmos_view
);

fn exit_cosmos_view(
    mut commands: Commands,
    mut bodies_query: Query<&mut Visibility, With<BodyIndex>>,
    body_data_panel: Res<BodyDataPanel>,
) {
    bodies_query
        .par_iter_mut()
        .for_each(|mut vis| *vis = Visibility::Hidden);

    commands
        .entity(**body_data_panel)
        .insert(Visibility::Hidden);
}

fn enter_cosmos_view(mut bodies_query: Query<&mut Visibility, With<BodyIndex>>) {
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
