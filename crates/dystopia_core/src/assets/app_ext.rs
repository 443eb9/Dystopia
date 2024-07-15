use bevy::{
    app::{App, Update},
    asset::AssetApp,
    prelude::IntoSystemConfigs,
    state::{condition::in_state, state::OnEnter},
};

use crate::{
    assets::{
        config::{self, RawConfig},
        manifest::Manifest,
        JsonLoader,
    },
    schedule::state::AssetState,
};

pub trait DystopiaAssetAppExt {
    /// Add a config into the app. `R` is the processed config while `C` is the raw one, ehich
    /// is directly deserialized from the json file.
    fn add_config<C: RawConfig>(&mut self);
}

impl DystopiaAssetAppExt for App {
    fn add_config<C: RawConfig>(&mut self) {
        self.init_asset::<C>()
            .init_asset_loader::<JsonLoader<C>>()
            .add_systems(OnEnter(AssetState::Load), C::load)
            .add_systems(
                Update,
                (config::process_raw_config_when_finish_loading::<C>)
                    .run_if(in_state(AssetState::Load)),
            );

        if let Some(mut manifest) = self.world_mut().get_resource_mut::<Manifest>() {
            manifest.add::<C>();
        } else {
            self.world_mut()
                .insert_resource(Manifest::new([C::NAME.to_string()].into()))
        }
    }
}
