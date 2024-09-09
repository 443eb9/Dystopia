use bevy::{
    app::{App, Plugin, Update},
    asset::AssetServer,
    math::{UVec2, Vec2},
    prelude::{
        in_state, Commands, Component, Entity, Has, IntoSystemConfigs, Query, Res, Visibility,
    },
    render::render_resource::FilterMode,
};
use rand::RngCore;

use crate::{
    cosmos::celestial::{BodyIndex, BodyTilemap},
    map::{
        bundle::TilemapBundle,
        shape::rectangle,
        tilemap::{
            Tile, TileAtlasIndex, TileIndex, TileRenderSize, TilemapStorage, TilemapTexture,
            TilemapTextureDescriptor, TilemapTilesets,
        },
    },
    schedule::state::SceneState,
    util::chunking::DEFAULT_CHUNK_SIZE,
};

pub struct TilemapGenerationPlugin;

impl Plugin for TilemapGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            generate_map.run_if(in_state(SceneState::CosmosView)),
        );
    }
}

#[derive(Component)]
pub struct ToGenerateMap;

// TODO real generation algo
pub fn generate_map(
    mut commands: Commands,
    bodies_query: Query<(Entity, &BodyIndex, &ToGenerateMap, Has<BodyTilemap>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, index, _generation_cofig, has_tilemap) in &bodies_query {
        if has_tilemap {
            continue;
        }

        let mut tilemap = TilemapBundle {
            tile_render_size: TileRenderSize(Vec2 { x: 32., y: 16. }),
            storgae: TilemapStorage::default(),
            tilesets: TilemapTilesets::new(
                vec![
                    TilemapTexture {
                        handle: asset_server.load("images/test_tileset_a.png"),
                        desc: TilemapTextureDescriptor {
                            size: UVec2 { x: 32, y: 32 },
                            tile_size: UVec2 { x: 32, y: 16 },
                        },
                    },
                    TilemapTexture {
                        handle: asset_server.load("images/test_tileset_b.png"),
                        desc: TilemapTextureDescriptor {
                            size: UVec2 { x: 32, y: 32 },
                            tile_size: UVec2 { x: 32, y: 16 },
                        },
                    },
                ],
                FilterMode::Nearest,
            ),
            visibility: Visibility::Hidden,
            ..Default::default()
        };

        let mut rng = rand::thread_rng();
        for index in rectangle(10, 10).into_iter() {
            tilemap.storgae.set(Tile {
                index: TileIndex::from_direct(index.as_ivec2(), DEFAULT_CHUNK_SIZE),
                atlas_index: TileAtlasIndex::Static(
                    (rng.next_u32() % 2, rng.next_u32() % 2).into(),
                ),
                ..Default::default()
            });
        }

        let tilemap = commands.spawn(tilemap).id();
        commands
            .entity(entity)
            .insert(BodyTilemap::new(tilemap))
            .remove::<ToGenerateMap>();
    }
}
