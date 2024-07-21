use bevy::{
    color::ColorToComponents,
    ecs::entity::EntityHashMap,
    math::{IVec3, Vec3, Vec4},
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
    render::{ExtractedTile, ExtractedTilemap},
    tilemap::TileAtlasIndex,
};

pub const TILEMAP_MESH_ATLAS_INDEX_ATTR: MeshVertexAttribute =
    MeshVertexAttribute::new("Atlas_Index", 1433223, VertexFormat::Uint32x3);
pub const TILEMAP_MESH_TILE_INDEX_ATTR: MeshVertexAttribute =
    MeshVertexAttribute::new("Tile_Index", 1433224, VertexFormat::Sint32x3);

#[derive(Clone)]
pub struct TileMeshData {
    pub tint: Vec4,
    /// If not animated: `[texture_index, atlas_index, 0]`
    ///
    /// If animated: `[start, end, 1]`
    pub atlas_index: [u32; 3],
    pub tile_index: IVec3,
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
            tiles: vec![None; chunk_size.pow(3) as usize],
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
    pub chunks: HashMap<IVec3, TilemapRenderChunk>,
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
    tiles: Res<ExtractedInstances<ExtractedTile>>,
    mut mesh_storage: ResMut<TilemapMeshStorage>,
) {
    for tile in tiles.values() {
        let index = tile.index.flattend();
        let chunks = mesh_storage.storage.get_mut(&tile.binded_tilemap).unwrap();
        let chunk = chunks
            .chunks
            .entry(index.chunk_index)
            .or_insert_with(|| TilemapRenderChunk::new(chunks.chunk_size));

        let data = {
            if tile.changed_vis.is_some_and(|v| !v) {
                None
            } else {
                Some(TileMeshData {
                    tint: tile.tint.0.to_vec4(),
                    atlas_index: match tile.atlas_index {
                        TileAtlasIndex::Static { texture, atlas } => [texture, atlas, 0],
                    },
                    tile_index: tile.index.direct(),
                })
            }
        };

        chunk.set(index.in_chunk_index, data);
    }
}

pub fn prepare_tilemap_meshes(
    mut mesh_storage: ResMut<TilemapMeshStorage>,
    render_device: Res<RenderDevice>,
) {
    for (tilemap, render_chunks) in mesh_storage.storage.iter_mut() {
        render_chunks
            .chunks
            .par_iter_mut()
            .for_each(|(index, chunk)| {
                if !chunk.is_dirty {
                    return;
                }

                let n = (chunk.chunk_size * chunk.chunk_size) as usize;
                let mut vertex_index = 0;
                let mut vertex_indices = Vec::with_capacity(n * 4);

                let mut vertex_position = Vec::with_capacity(n * 3);
                let mut atlas_indices = Vec::with_capacity(n * 3);
                let mut vertex_color = Vec::with_capacity(n * 3);
                let mut tile_indices = Vec::with_capacity(n * 3);

                for tile in chunk.tiles.iter().filter_map(|t| t.as_ref()) {
                    vertex_position.extend_from_slice(&[Vec3::ZERO, Vec3::ZERO, Vec3::ZERO]);
                    atlas_indices.extend_from_slice(&[
                        tile.atlas_index,
                        tile.atlas_index,
                        tile.atlas_index,
                    ]);
                    vertex_color.extend_from_slice(&[tile.tint, tile.tint, tile.tint]);
                    vertex_indices.extend_from_slice(&[
                        vertex_index,
                        vertex_index + 1,
                        vertex_index + 2,
                    ]);
                    tile_indices.extend_from_slice(&[
                        tile.tile_index,
                        tile.tile_index,
                        tile.tile_index,
                    ]);

                    vertex_index += 3;
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
