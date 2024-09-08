use bevy::prelude::{Commands, Entity, EventReader, Query, Res};

use crate::{
    cosmos::celestial::{BodyIndex, ToLoadTilemap},
    map::{gen::ToGenerateMap, serde::is_tilemap_exist_in_disk},
    sim::SaveName,
    ui::panel::{body_data::BodyDataPanel, PanelTargetChange},
};

pub fn init_tilemap_when_body_clicked(
    mut commands: Commands,
    bodies_query: Query<(Entity, &BodyIndex)>,
    mut target_change: EventReader<PanelTargetChange<BodyDataPanel>>,
    save_name: Res<SaveName>,
) {
    for change in target_change.read().filter_map(|c| **c) {
        let (entity, body_index) = bodies_query.get(change).unwrap();

        if is_tilemap_exist_in_disk(&save_name, **body_index) {
            commands.entity(entity).insert(ToLoadTilemap);
        } else {
            commands.entity(entity).insert(ToGenerateMap);
        }
    }
}
