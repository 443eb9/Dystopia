use bevy::{
    app::{App, Update},
    asset::AssetApp,
    prelude::{IntoSystemConfigs, Resource},
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

pub trait InfGdnAssetAppExt {
    fn add_config<R: Resource, C: RawConfig<R>>(&mut self);
}

impl InfGdnAssetAppExt for App {
    fn add_config<R: Resource, C: RawConfig<R>>(&mut self) {
        self.init_asset::<C>()
            .init_asset_loader::<JsonLoader<C>>()
            .add_systems(OnEnter(AssetState::Load), C::load)
            .add_systems(
                Update,
                (config::process_raw_config_when_finish_loading::<R, C>)
                    .run_if(in_state(AssetState::Load)),
            );

        if let Some(mut manifest) = self.world_mut().get_resource_mut::<Manifest>() {
            manifest.add::<R, C>();
        } else {
            self.world_mut()
                .insert_resource(Manifest::new([C::NAME.to_string()].into()))
        }
    }
}
