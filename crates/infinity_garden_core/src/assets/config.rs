//! Configs about the game, generally some constants including units, some presets etc.

use std::{marker::PhantomData, path::Path};

use bevy::{
    asset::{Asset, AssetServer, Assets, Handle},
    log::info,
    prelude::{Commands, Res, ResMut, Resource, World},
    utils::HashMap,
};
use serde::de::DeserializeOwned;

use crate::assets::{manifest::Manifest, Id};

#[derive(Resource)]
pub struct RawConfigHandle<R: Resource, T: RawConfig<R>> {
    pub handle: Handle<T>,
    _marker: PhantomData<R>,
}

pub trait RawConfig<R: Resource>: Asset + DeserializeOwned + Sized {
    const NAME: &'static str;

    fn load(world: &mut World) {
        info!("Start loading config: {}", Self::NAME);

        let handle = world
            .resource::<AssetServer>()
            .load::<Self>(Path::new("configs").join(Self::NAME));

        world.insert_resource(RawConfigHandle {
            handle,
            _marker: PhantomData::default(),
        });
    }

    fn process(&self) -> R;
}

#[derive(Resource)]
pub struct Config<K, V>(pub HashMap<Id<K>, V>);

#[derive(Resource)]
pub struct ConfigLiteral<K, V>(pub HashMap<K, V>);

pub fn process_raw_config_when_finish_loading<R: Resource, C: RawConfig<R>>(
    mut command: Commands,
    handle: Option<Res<RawConfigHandle<R, C>>>,
    assets: Res<Assets<C>>,
    mut manifest: ResMut<Manifest>,
) {
    if let Some(handle) = handle {
        if let Some(assets) = assets.get(&handle.handle) {
            command.insert_resource(assets.process());
            manifest.finish::<R, C>();
        }
    }
}
