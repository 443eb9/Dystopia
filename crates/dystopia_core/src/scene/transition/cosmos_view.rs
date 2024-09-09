use bevy::prelude::{
    Commands, Entity, EventWriter, MouseButton, NextState, Query, Res, ResMut, Transform,
    Visibility, With, Without,
};

use crate::{
    body::FocusingOn,
    cosmos::celestial::{BodyIndex, BodyTilemap},
    impl_transition_plugin,
    input::{MouseClickCounter, MouseInput},
    scene::transition::CameraRecoverTransform,
    schedule::state::SceneState,
    sim::{MainCamera, ViewScale},
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
    camera_query: Query<&Transform, (With<MainCamera>, Without<BodyIndex>)>,
    view_scale: Res<ViewScale>,
    mut target_change: EventWriter<PanelTargetChange<BodyDataPanel>>,
) {
    bodies_query
        .par_iter_mut()
        .for_each(|mut vis| *vis = Visibility::Hidden);

    target_change.send(PanelTargetChange::none());

    commands.insert_resource(CameraRecoverTransform::new(
        &camera_query.single(),
        &view_scale,
    ));
}

fn enter_cosmos_view(
    mut bodies_query: Query<&mut Visibility, With<BodyIndex>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<BodyIndex>)>,
    mut view_scale: ResMut<ViewScale>,
    recover_transl: Res<CameraRecoverTransform>,
) {
    bodies_query
        .par_iter_mut()
        .for_each(|mut vis| *vis = Visibility::Inherited);

    recover_transl.recover(&mut camera_query.single_mut(), &mut view_scale);
}

pub fn handle_body_focusing(
    mut commands: Commands,
    mut scene_state: ResMut<NextState<SceneState>>,
    double_clicked_query: Query<(
        Entity,
        &BodyIndex,
        &BodyTilemap,
        &MouseInput,
        &MouseClickCounter,
    )>,
) {
    for (entity, body, tilemap, input, counter) in &double_clicked_query {
        if input.button != MouseButton::Left || **counter != 2 {
            continue;
        }

        commands.insert_resource(FocusingOn {
            entity,
            body: *body,
            tilemap: *tilemap,
        });
        scene_state.set(SceneState::FocusingBody);
    }
}
