use bevy::{
    color::ColorToComponents,
    ecs::entity::EntityHashMap,
    math::{IVec2, Vec3, Vec4},
    prelude::{Res, ResMut, Resource},
    render::{
        extract_instances::ExtractedInstances,
        mesh::{
            BaseMeshPipelineKey, GpuBufferInfo, GpuMesh, Indices, Mesh, MeshVertexAttribute,
            MeshVertexBufferLayouts, PrimitiveTopology,
        },
        render_asset::RenderAssetUsages,
        render_resource::{BufferInitDescriptor, BufferUsages, IndexFormat, VertexFormat},
        renderer::RenderDevice,
    },
    utils::HashMap,
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::map::{
    render::ExtractedTilemap,
    tilemap::{FlattenedTileIndex, Tile, TileAtlasIndex},
};

pub const TILEMAP_MESH_ATLAS_INDEX_ATTR: MeshVertexAttribute =
    MeshVertexAttribute::new("Atlas_Index", 1433223, VertexFormat::Uint32x4);
pub const TILEMAP_MESH_TILE_INDEX_ATTR: MeshVertexAttribute =
    MeshVertexAttribute::new("Tile_Index", 1433224, VertexFormat::Sint32x2);

#[derive(Clone)]
pub struct TileMeshData {
    pub tint: Vec4,
    /// If not animated: `[texture_index, atlas_index, 0, 0]`
    ///
    /// If animated: `[start, len, 1, offset_milisec]`
    pub atlas_index: [u32; 4],
    pub tile_index: IVec2,
}

pub struct TilemapRenderChunk {
    pub chunk_size: u32,
    pub mesh: Option<Mesh>,
    pub gpu_mesh: Option<GpuMesh>,

    tiles: Vec<Option<TileMeshData>>,
    is_dirty: bool,
}

impl TilemapRenderChunk {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            tiles: vec![None; chunk_size.pow(2) as usize],
            mesh: Default::default(),
            gpu_mesh: Default::default(),
            is_dirty: true,
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, tile: Option<TileMeshData>) {
        self.tiles[index] = tile;
        self.is_dirty = true;
    }

    #[inline]
    pub fn tiles(&self) -> &Vec<Option<TileMeshData>> {
        &self.tiles
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}

pub struct TilemapRenderChunks {
    pub chunk_size: u32,
    pub chunks: HashMap<IVec2, TilemapRenderChunk>,
}

#[derive(Resource, Default)]
pub struct TilemapMeshStorage {
    pub storage: EntityHashMap<TilemapRenderChunks>,
}

pub fn register_tilemaps_in_storage(
    tilemaps: Res<ExtractedInstances<ExtractedTilemap>>,
    mut mesh_storage: ResMut<TilemapMeshStorage>,
) {
    for (entity, tilemap) in tilemaps.iter() {
        if !mesh_storage.storage.contains_key(entity) {
            mesh_storage.storage.insert(
                *entity,
                TilemapRenderChunks {
                    chunk_size: tilemap.chunk_size,
                    chunks: Default::default(),
                },
            );
        }
    }
}

pub fn prepare_tile_mesh_data(
    tilemaps: Res<ExtractedInstances<ExtractedTilemap>>,
    mut mesh_storage: ResMut<TilemapMeshStorage>,
) {
    for (tilemap_entity, tilemap) in tilemaps.iter() {
        let chunks = mesh_storage.storage.get_mut(tilemap_entity).unwrap();

        for (index, tile) in &tilemap.changed_tiles {
            update_tile_mesh(index, tile, chunks);
        }

        for (index, chunk) in &tilemap.changed_chunks {
            if let Some(chunk) = chunk {
                for (i, tile) in chunk.iter().enumerate() {
                    update_tile_mesh(
                        &FlattenedTileIndex {
                            chunk_index: *index,
                            in_chunk_index: i,
                        },
                        tile,
                        chunks,
                    );
                }
            } else {
                chunks.chunks.remove(index);
            }
        }
    }
}

fn update_tile_mesh(
    index: &FlattenedTileIndex,
    tile: &Option<Tile>,
    chunks: &mut TilemapRenderChunks,
) {
    let Some(tile) = tile else {
        if let Some(chunk) = chunks.chunks.get_mut(&index.chunk_index) {
            chunk.set(index.in_chunk_index, None);
        }
        return;
    };

    let index = tile.index.flattened();
    let chunk = chunks
        .chunks
        .entry(index.chunk_index)
        .or_insert_with(|| TilemapRenderChunk::new(chunks.chunk_size));

    let data = tile.visible.then_some(TileMeshData {
        tint: tile.tint.to_linear().to_vec4(),
        atlas_index: match tile.atlas_index {
            TileAtlasIndex::Static(mut s) => {
                s = s.encode();
                [s.texture, s.atlas, 0, 0]
            }
            TileAtlasIndex::Animated {
                anim,
                offset_milisec,
            } => [anim.start as u32, anim.len as u32, 1, offset_milisec],
        },
        tile_index: tile.index.direct(),
    });

    chunk.set(index.in_chunk_index, data);
}

pub fn prepare_tilemap_meshes(
    mut mesh_storage: ResMut<TilemapMeshStorage>,
    render_device: Res<RenderDevice>,
) {
    for (tilemap, render_chunks) in mesh_storage.storage.iter_mut() {
        render_chunks
            .chunks
            .par_iter_mut()
            .filter(|(_, c)| c.is_dirty)
            .for_each(|(index, chunk)| {
                let n = chunk.tiles.len();
                let mut vertex_indices = Vec::with_capacity(n * 6);

                let mut vertex_position = Vec::with_capacity(n * 4);
                let mut atlas_indices = Vec::with_capacity(n * 4);
                let mut vertex_color = Vec::with_capacity(n * 4);
                let mut tile_indices = Vec::with_capacity(n * 4);

                // TODO if not adding 6 vertices, the rectangle mesh cannot be built
                //      looks like the 3 at the back cannot be recognized.
                //      maybe try again in next version of bevy.
                for (i_tile, tile) in chunk.tiles.iter().filter_map(|t| t.as_ref()).enumerate() {
                    vertex_position.extend_from_slice(&[
                        Vec3::ZERO,
                        Vec3::ZERO,
                        Vec3::ZERO,
                        Vec3::ZERO,
                        Vec3::ZERO,
                        Vec3::ZERO,
                    ]);
                    atlas_indices.extend_from_slice(&[
                        tile.atlas_index,
                        tile.atlas_index,
                        tile.atlas_index,
                        tile.atlas_index,
                        tile.atlas_index,
                        tile.atlas_index,
                    ]);
                    vertex_color.extend_from_slice(&[
                        tile.tint, tile.tint, tile.tint, tile.tint, tile.tint, tile.tint,
                    ]);
                    tile_indices.extend_from_slice(&[
                        tile.tile_index,
                        tile.tile_index,
                        tile.tile_index,
                        tile.tile_index,
                        tile.tile_index,
                        tile.tile_index,
                    ]);

                    let base_index = i_tile as u32 * 6;
                    vertex_indices.extend_from_slice(&[
                        base_index,
                        base_index + 1,
                        base_index + 3,
                        base_index + 1,
                        base_index + 2,
                        base_index + 3,
                    ]);
                }

                let mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex_position)
                .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vertex_color)
                .with_inserted_attribute(TILEMAP_MESH_ATLAS_INDEX_ATTR, atlas_indices)
                .with_inserted_attribute(TILEMAP_MESH_TILE_INDEX_ATTR, tile_indices)
                .with_inserted_indices(Indices::U32(vertex_indices));

                let vertex_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some(&format!("tilemap_chunk_mesh_{}_{}", tilemap, index)),
                    contents: &mesh.get_vertex_buffer_data(),
                    usage: BufferUsages::VERTEX,
                });
                let vertex_count = mesh.count_vertices() as u32;
                let buffer_info =
                    mesh.get_index_buffer_bytes()
                        .map_or(GpuBufferInfo::NonIndexed, |data| GpuBufferInfo::Indexed {
                            buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                                label: None,
                                contents: data,
                                usage: BufferUsages::INDEX,
                            }),
                            count: vertex_count,
                            index_format: IndexFormat::Uint32,
                        });

                let layout =
                    mesh.get_mesh_vertex_buffer_layout(&mut MeshVertexBufferLayouts::default());
                let key_bits =
                    BaseMeshPipelineKey::from_primitive_topology(PrimitiveTopology::TriangleList);

                let gpu_mesh = GpuMesh {
                    vertex_buffer,
                    vertex_count,
                    morph_targets: None,
                    buffer_info,
                    key_bits,
                    layout,
                };

                chunk.gpu_mesh = Some(gpu_mesh);
                chunk.mesh = Some(mesh);

                chunk.is_dirty = false;
            });
    }
}
