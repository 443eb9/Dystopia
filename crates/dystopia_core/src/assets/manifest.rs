use bevy::{
    log::info,
    prelude::{Res, ResMut, Resource},
    state::state::NextState,
    utils::HashSet,
};

use crate::{
    assets::config::RawConfig,
    schedule::state::{AssetState, ProcessState},
};

#[derive(Resource)]
pub struct Manifest {
    to_load: HashSet<String>,
    total: u32,
}

impl Manifest {
    pub fn new(to_load: HashSet<String>) -> Self {
        Self {
            total: to_load.len() as u32,
            to_load,
        }
    }

    pub fn finish<C: RawConfig>(&mut self) {
        self.to_load.remove(C::NAME);
    }

    pub fn add<C: RawConfig>(&mut self) {
        self.to_load.insert(C::NAME.to_string());
        self.total += 1;
    }
}

pub fn check_if_manifest_finished(
    manifest: Res<Manifest>,
    mut asset_state: ResMut<NextState<AssetState>>,
    mut process_state: ResMut<NextState<ProcessState>>,
) {
    if manifest.to_load.is_empty() {
        asset_state.set(AssetState::Finish);
        process_state.set(ProcessState::InGame);
        info!("All assets successfully loaded.");
    }
}
