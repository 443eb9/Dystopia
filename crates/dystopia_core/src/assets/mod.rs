//! Load static assets, like configurations, images etc.

use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Update},
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
    prelude::IntoSystemConfigs,
    state::condition::in_state,
};
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::schedule::state::AssetState;

pub mod app_ext;
pub mod config;
pub mod manifest;

pub struct DystopiaAssetsPlugin;

impl Plugin for DystopiaAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            manifest::check_if_manifest_finished.run_if(in_state(AssetState::Load)),
        );
    }
}

#[derive(Error, Debug)]
pub enum JsonLoaderError {
    #[error("Failed to load json {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to load json {0}")]
    Json(#[from] serde_json::Error),
}

pub struct JsonLoader<A: Asset + DeserializeOwned>(PhantomData<A>);

impl<A: Asset + DeserializeOwned> Default for JsonLoader<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<A: Asset + DeserializeOwned> AssetLoader for JsonLoader<A> {
    type Asset = A;

    type Settings = ();

    type Error = JsonLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf).map_err(Into::into)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
