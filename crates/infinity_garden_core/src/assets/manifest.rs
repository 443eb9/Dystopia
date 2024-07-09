use bevy::{
    log::info,
    prelude::{Res, ResMut, Resource},
    state::state::NextState,
    utils::HashSet,
};

use crate::{assets::config::RawConfig, schedule::state::AssetState};

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

    pub fn finish<R: Resource, C: RawConfig<R>>(&mut self) {
        self.to_load.remove(C::NAME);
    }

    pub fn add<R: Resource, C: RawConfig<R>>(&mut self) {
        self.to_load.insert(C::NAME.to_string());
        self.total += 1;
    }
}

pub fn check_if_manifest_finished(
    manifest: Res<Manifest>,
    mut state: ResMut<NextState<AssetState>>,
) {
    if manifest.to_load.is_empty() {
        state.set(AssetState::Finish);
        info!("All assets successfully loaded.");
    }
}
