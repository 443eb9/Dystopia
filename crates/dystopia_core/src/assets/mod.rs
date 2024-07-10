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
use uuid::Uuid;

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

/// A unique identifier for objects like items, structures etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<T> {
    id: Uuid,
    _markder: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn from_str(s: &str) -> Self {
        // Algorithm adopted from <https://cp-algorithms.com/string/string-hashing.html>

        const P: u128 = 31;
        const M: u128 = 1000000009;
        let mut hash_value = 0;
        let mut p_pow = 1;

        s.bytes().for_each(|c| {
            hash_value = (hash_value + (c as u128 + 1) * p_pow) % M;
            p_pow = (p_pow * P) % M;
        });

        Self {
            id: Uuid::from_u128(hash_value),
            _markder: PhantomData::default(),
        }
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
}
