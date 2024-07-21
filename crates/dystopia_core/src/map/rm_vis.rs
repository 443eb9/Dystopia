//! Removal and visibility change handlers.

use bevy::{
    app::{App, First, Plugin, Update},
    math::IVec3,
    prelude::{
        Commands, Component, Entity, IntoSystemConfigs, ParallelCommands, Query, Res, ResMut,
        Resource, With,
    },
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        Render, RenderApp, RenderSet,
    },
    state::condition::in_state,
};

use crate::{
    map::{
        render::{
            mesh::TilemapMeshStorage,
            resource::{TilemapBindGroups, TilemapBuffers},
            texture::TilemapTextureStorage,
        },
        tilemap::{TileBindedTilemap, TileIndex, TilemapStorage},
    },
    schedule::state::GameState,
};

/// Handler of removal and visibility change for tilemaps
/// and tiles.
pub struct DystopiaMapRmVisPlugin;

impl Plugin for DystopiaMapRmVisPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<RemovedTilemapsRecorder>::default(),
            ExtractResourcePlugin::<RemovedTilesRecorder>::default(),
            ExtractResourcePlugin::<RemovedTilemapChunksRecorder>::default(),
        ))
        .add_systems(First, reset_removal_recorders)
        .add_systems(
            Update,
            (despawn_tilemaps, despawn_tiles, remove_chunks).run_if(in_state(GameState::Simulate)),
        )
        .init_resource::<RemovedTilemapsRecorder>()
        .init_resource::<RemovedTilesRecorder>()
        .init_resource::<RemovedTilemapChunksRecorder>();

        let render_app = app.sub_app_mut(RenderApp);

        render_app.add_systems(
            Render,
            (
                handle_render_tilemap_removal,
                handle_render_tile_removal,
                handle_render_chunks_removal,
            )
                .in_set(RenderSet::PrepareResources),
        );
    }
}

/// Mark a tilemap/tile to be despawned.
///
/// As they require some clean up in render world, we can't
/// despawn them directly.
#[derive(Component)]
pub struct DespawnMe;

/// Mark a chunk in a tilemap to be removed.
///
/// As they require some clean up in render world, we can't
/// despawn them directly.
#[derive(Component, Clone)]
pub struct RemoveTilemapChunk {
    pub tilemap: Entity,
    pub index: IVec3,
}

#[derive(Resource, ExtractResource, Default, Clone)]
pub struct RemovedTilemapsRecorder(Vec<Entity>);

#[derive(Resource, ExtractResource, Default, Clone)]
pub struct RemovedTilesRecorder(Vec<(TileBindedTilemap, TileIndex)>);

#[derive(Resource, ExtractResource, Default, Clone)]
pub struct RemovedTilemapChunksRecorder(Vec<RemoveTilemapChunk>);

pub fn reset_removal_recorders(
    mut despawned_tilemaps: ResMut<RemovedTilemapsRecorder>,
    mut despawned_tiles: ResMut<RemovedTilesRecorder>,
    mut removed_tilemap_chunks: ResMut<RemovedTilemapChunksRecorder>,
) {
    despawned_tilemaps.0.clear();
    despawned_tiles.0.clear();
    removed_tilemap_chunks.0.clear();
}

pub fn despawn_tilemaps(
    mut commands: Commands,
    tilemaps_query: Query<Entity, (With<DespawnMe>, With<TilemapStorage>)>,
    mut despawned_tilemaps: ResMut<RemovedTilemapsRecorder>,
) {
    despawned_tilemaps.0.extend(&tilemaps_query);

    tilemaps_query
        .iter()
        .for_each(|e| commands.entity(e).despawn());
}

pub fn despawn_tiles(
    commands: ParallelCommands,
    tiles_query: Query<(Entity, &TileBindedTilemap, &TileIndex), With<DespawnMe>>,
    mut despawned_tiles: ResMut<RemovedTilesRecorder>,
) {
    despawned_tiles
        .0
        .extend(tiles_query.iter().map(|(_, b, i)| (*b, *i)));

    tiles_query.par_iter().for_each(|(e, ..)| {
        commands.command_scope(|mut c| c.entity(e).despawn());
    });
}

pub fn remove_chunks(
    commands: ParallelCommands,
    chunks_query: Query<(Entity, &RemoveTilemapChunk)>,
    mut removed_tilemap_chunks: ResMut<RemovedTilemapChunksRecorder>,
) {
    removed_tilemap_chunks
        .0
        .extend(chunks_query.iter().map(|(_, r)| r.clone()));

    chunks_query.par_iter().for_each(|(e, _)| {
        commands.command_scope(|mut c| c.entity(e).despawn());
    });
}

pub fn handle_render_tilemap_removal(
    despawned_tilemaps: Res<RemovedTilemapsRecorder>,
    mut textures: ResMut<TilemapTextureStorage>,
    mut mesh_storage: ResMut<TilemapMeshStorage>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    mut buffers: ResMut<TilemapBuffers>,
) {
    for tilemap in &despawned_tilemaps.0 {
        textures.processed.remove(tilemap);
        mesh_storage.storage.remove(tilemap);
        bind_groups.bind_groups.remove(tilemap);
        buffers.individual.remove(tilemap);
    }
}

pub fn handle_render_tile_removal(
    despawned_tiles: Res<RemovedTilesRecorder>,
    mut mesh_storage: ResMut<TilemapMeshStorage>,
) {
    for (tilemap, index) in &despawned_tiles.0 {
        let Some(chunks) = mesh_storage.storage.get_mut(&tilemap.0) else {
            continue;
        };

        let index = index.flattend();
        let Some(chunk) = chunks.chunks.get_mut(&index.chunk_index) else {
            continue;
        };

        chunk.set(index.in_chunk_index, None);
    }
}

pub fn handle_render_chunks_removal(
    removed_tilemap_chunks: Res<RemovedTilemapChunksRecorder>,
    mut mesh_storage: ResMut<TilemapMeshStorage>,
) {
    for removal in &removed_tilemap_chunks.0 {
        if let Some(chunks) = mesh_storage.storage.get_mut(&removal.tilemap) {
            chunks.chunks.remove(&removal.index);
        }
    }
}
