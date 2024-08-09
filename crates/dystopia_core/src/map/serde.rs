use std::{fs::File, io::Write, path::Path};

use bevy::{
    app::{App, Plugin},
    asset::{io::Reader, Asset, AssetApp, AssetLoader, AssetServer, AsyncReadExt, LoadContext},
    color::ColorToComponents,
    log::{error, info},
    prelude::{Commands, Entity, OnInsert, Query, Res, Trigger},
    reflect::TypePath,
};
use bincode::{config::Configuration, error::DecodeError, Decode, Encode};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use thiserror::Error;

use crate::{
    cosmos::celestial::{BodyIndex, ToSaveTilemap},
    map::tilemap::{
        TileAtlasIndex, TileRenderSize, TilemapAnimations, TilemapStorage, TilemapTilesets,
        TilemapTint,
    },
    simulation::SaveName,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ENCDEC_CONFIG: Configuration = bincode::config::standard();

pub(super) struct TilemapSerdePlugin;

impl Plugin for TilemapSerdePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BinaryTilemap>()
            .init_asset_loader::<BinaryTilemapLoader>()
            .observe(save_tilemap);
    }
}

#[derive(Encode, Decode)]
enum BinaryAtlasIndex {
    Static {
        texture: u32,
        atlas: u32,
        flip: u32,
    },
    Animated {
        start: usize,
        len: usize,
        offset_milisec: u32,
    },
}

#[derive(Encode, Decode)]
struct BinaryTilesets {
    size: [u32; 2],
    filter_mode: u32,
    textures: Vec<(String, [u32; 2], [u32; 2])>,
}

#[derive(Encode, Decode)]
struct BinaryTile {
    atlas: BinaryAtlasIndex,
    tint: [f32; 4],
    visible: bool,
}

// TODO replace `[number; dimension]`s with glam vectors.
#[derive(Encode, Decode, Asset, TypePath)]
pub struct BinaryTilemap {
    version: u32,
    target_body: usize,
    tile_render_size: [f32; 2],
    chunk_size: u32,
    storgae: Vec<([i32; 3], BinaryTile)>,
    tint: [f32; 4],
    tilesets: BinaryTilesets,
    animations: Vec<u32>,
}

#[derive(Error, Debug)]
pub enum TilemapBinaryLoadError {
    #[error("Io error: {0:?}")]
    Io(std::io::Error),
    #[error("Decode error: {0:?}")]
    Decode(DecodeError),
}

#[derive(Default)]
pub struct BinaryTilemapLoader;

impl AssetLoader for BinaryTilemapLoader {
    type Asset = BinaryTilemap;

    type Settings = ();

    type Error = TilemapBinaryLoadError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader
            .read_to_end(&mut buf)
            .await
            .map_err(|e| TilemapBinaryLoadError::Io(e))?;
        bincode::decode_from_slice(&buf, ENCDEC_CONFIG)
            .map(|r| r.0)
            .map_err(|e| TilemapBinaryLoadError::Decode(e))
    }

    fn extensions(&self) -> &[&str] {
        &["tmb"]
    }
}

fn save_tilemap(
    trigger: Trigger<OnInsert, ToSaveTilemap>,
    mut commands: Commands,
    to_unload_query: Query<(
        Entity,
        &BodyIndex,
        &TileRenderSize,
        &TilemapStorage,
        &TilemapTint,
        &TilemapTilesets,
        &TilemapAnimations,
        &ToSaveTilemap,
    )>,
    asset_server: Res<AssetServer>,
    save_name: Res<SaveName>,
) {
    let Ok((
        entity,
        body_index,
        tile_render_size,
        storage,
        tint,
        tilesets,
        animations,
        save_options,
    )) = to_unload_query.get(trigger.entity())
    else {
        return;
    };

    commands.entity(entity).remove::<ToSaveTilemap>();

    let binary = BinaryTilemap {
        version: VERSION.split('.').nth(0).unwrap().parse().unwrap(),
        target_body: **body_index,
        tile_render_size: tile_render_size.to_array(),
        chunk_size: storage.chunk_size(),
        storgae: unsafe {
            (*storage.as_unsafe_cell_readonly().internal)
                .par_values()
                .flat_map(|c| {
                    c.par_iter().filter_map(|t| {
                        t.as_ref().map(|t| {
                            (
                                t.index.direct().to_array(),
                                BinaryTile {
                                    atlas: match t.atlas_index {
                                        TileAtlasIndex::Static(s) => BinaryAtlasIndex::Static {
                                            texture: s.texture,
                                            atlas: s.atlas,
                                            flip: s.flip.bits(),
                                        },
                                        TileAtlasIndex::Animated {
                                            anim,
                                            offset_milisec,
                                        } => BinaryAtlasIndex::Animated {
                                            start: anim.start,
                                            len: anim.len,
                                            offset_milisec,
                                        },
                                    },
                                    tint: t.tint.to_linear().to_f32_array(),
                                    visible: t.visible,
                                },
                            )
                        })
                    })
                })
                .collect()
        },
        tint: tint.to_linear().to_f32_array(),
        tilesets: BinaryTilesets {
            size: tilesets.size().to_array(),
            filter_mode: tilesets.filter_mode() as u32,
            textures: tilesets
                .textures()
                .iter()
                .map(|tex| {
                    (
                        asset_server.get_path(&tex.handle).unwrap().to_string(),
                        tex.desc.size.to_array(),
                        tex.desc.tile_size.to_array(),
                    )
                })
                .collect(),
        },
        animations: animations.bytes().clone(),
    };

    match bincode::encode_to_vec(binary, ENCDEC_CONFIG) {
        Ok(data) => {
            let path = Path::new("data")
                .join("saves")
                .join(&**save_name)
                .join("maps")
                .join(format!("{}.tmb", **body_index));
            
            match write_bytes(&data, &path) {
                Ok(len) => {
                    if save_options.remove_after_done {
                        commands.entity(entity).remove::<(
                            TileRenderSize,
                            TilemapStorage,
                            TilemapTint,
                            TilemapTilesets,
                            TilemapAnimations,
                        )>();
                    }

                    info!(
                        "Successfully saved tilemap of body {}. {} bytes are written.",
                        **body_index, len
                    );
                }
                Err(err) => {
                    error!(
                        "Failed to write data into tilemap save of body {}: {}",
                        **body_index, err
                    );
                }
            }
        }
        Err(err) => error!(
            "Failed to encode tilemap data for body {}: {}",
            **body_index, err
        ),
    }
}

fn write_bytes(bytes: &[u8], path: &Path) -> Result<usize, std::io::Error> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut file = File::create(path)?;
    file.write(bytes)
}
