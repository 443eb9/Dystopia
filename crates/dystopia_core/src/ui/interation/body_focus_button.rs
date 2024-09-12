use bevy::prelude::{
    Commands, Component, EventWriter, NextState, Query, Res, ResMut, State, Transform, With, Without
};

use crate::{
    cosmos::celestial::{BodyIndex, Cosmos},
    input::MouseInput,
    schedule::state::SceneState,
    sim::MainCamera,
    ui::{panel::{body_data::BodyDataPanel, PanelTargetChange}, update::{AsOriginalComponent, AsUpdatableData, DataUpdatableUi}},
};

#[derive(Component, Default)]
pub struct BodyFocusButton {
    pub target: Option<BodyIndex>,
    /// Switch to CosmosView scene if not in?
    pub forced_scene_switch: bool,
}

impl AsOriginalComponent for Option<BodyIndex> {
    type OriginalComponent = BodyFocusButton;
}

impl AsUpdatableData for Option<BodyIndex> {
    type UpdatableData = Self;
}

impl DataUpdatableUi<Option<BodyIndex>> for BodyFocusButton {
    fn update_data(&mut self, data: &Option<BodyIndex>, _commands: &mut Commands) {
        self.target = *data;
    }
}

impl AsOriginalComponent for BodyIndex {
    type OriginalComponent = BodyFocusButton;
}

impl AsUpdatableData for BodyIndex {
    type UpdatableData = Self;
}

impl DataUpdatableUi<BodyIndex> for BodyFocusButton {
    fn update_data(&mut self, data: &BodyIndex, _commands: &mut Commands) {
        self.target = Some(*data);
    }
}

pub fn handle_body_focus_click(
    mut main_camera_query: Query<&mut Transform, (With<MainCamera>, Without<BodyIndex>)>,
    bodies_query: Query<&Transform, With<BodyIndex>>,
    btn_query: Query<(&BodyFocusButton, &MouseInput)>,
    mut target_change: EventWriter<PanelTargetChange<BodyDataPanel>>,
    cosmos: Res<Cosmos>,
    scene_state: Res<State<SceneState>>,
    mut next_scene_state: ResMut<NextState<SceneState>>,
) {
    for (btn, input) in &btn_query {
        if !input.is_left_click() {
            continue;
        }

        if let Some(target) = btn.target {
            if *scene_state.get() != SceneState::CosmosView {
                if btn.forced_scene_switch {
                    next_scene_state.set(SceneState::CosmosView);
                } else {
                    continue;
                }
            }

            let entity = cosmos.entities[*target];
            target_change.send(PanelTargetChange::some(entity));
            main_camera_query.single_mut().translation =
                bodies_query.get(entity).unwrap().translation;
        }
    }
}
