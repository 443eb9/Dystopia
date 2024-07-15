//! Configs about the game, generally some constants including units, some presets etc.

use std::path::Path;

use bevy::{
    asset::{Asset, AssetServer, Assets, Handle},
    log::info,
    prelude::{Commands, Res, ResMut, Resource, World},
};
use serde::de::DeserializeOwned;

use crate::assets::manifest::Manifest;

#[derive(Resource)]
pub struct RawConfigHandle<C: RawConfig> {
    pub handle: Handle<C>,
}

pub trait RawConfig: Asset + Clone + DeserializeOwned + Sized {
    type Processed: Resource + From<Self>;

    const NAME: &'static str;

    fn load(world: &mut World) {
        info!("Start loading config: {}", Self::NAME);

        let handle = world
            .resource::<AssetServer>()
            .load::<Self>(Path::new("configs").join(Self::NAME));

        world.insert_resource(RawConfigHandle { handle });
    }

    fn process(&self) -> Self::Processed {
        (*self).clone().into()
    }
}

pub fn process_raw_config_when_finish_loading<C: RawConfig>(
    mut command: Commands,
    handle: Option<Res<RawConfigHandle<C>>>,
    assets: Res<Assets<C>>,
    mut manifest: ResMut<Manifest>,
) {
    if let Some(handle) = handle {
        if let Some(assets) = assets.get(&handle.handle) {
            command.insert_resource(assets.process());
            manifest.finish::<C>();
        }
    }
}
