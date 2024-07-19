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

use crate::map::{render::ExtractedTile, tilemap::TileAtlasIndex};

pub const TILEMAP_MESH_ATLAS_INDEX_ATTR: MeshVertexAttribute =
    MeshVertexAttribute::new("Atlas_Index", 1433223, VertexFormat::Uint32x3);

pub struct TileMeshData {
    /// If not animated: `[texture_index, atlas_index, 0]`
    ///
    /// If animated: `[start, end, 1]`
    pub atlas_index: [u32; 3],
    pub tint: Vec4,
}

#[derive(Default)]
pub struct TilemapRenderChunk {
    pub chunk_size: u32,
    pub tiles: Vec<Option<TileMeshData>>,
    pub mesh: Option<Mesh>,
    pub gpu_mesh: Option<GpuMesh>,
    pub is_dirty: bool,
}

impl TilemapRenderChunk {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            tiles: Default::default(),
            mesh: Default::default(),
            gpu_mesh: Default::default(),
            is_dirty: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct TilemapMeshStorage {
    pub storage: EntityHashMap<HashMap<IVec3, TilemapRenderChunk>>,
}

pub fn prepare_tiles(
    tiles: Res<ExtractedInstances<ExtractedTile>>,
    mut mesh_storage: ResMut<TilemapMeshStorage>,
) {
    for tile in tiles.values() {
        let chunks = mesh_storage.storage.entry(tile.binded_tilemap).or_default();
        let chunk = chunks.entry(tile.index.chunk_index).or_default();
        chunk.tiles[tile.index.in_chunk_index] = Some(TileMeshData {
            atlas_index: match tile.atlas_index {
                TileAtlasIndex::Static { texture, atlas } => [texture, atlas, 0],
            },
            tint: tile.tint.0.to_vec4(),
        });
        chunk.is_dirty = true;
    }
}

pub fn prepare_tilemap_meshes(
    mut mesh_storage: ResMut<TilemapMeshStorage>,
    render_device: Res<RenderDevice>,
) {
    for (tilemap, render_chunks) in mesh_storage.storage.iter_mut() {
        render_chunks.par_iter_mut().for_each(|(index, chunk)| {
            if !chunk.is_dirty {
                return;
            }

            let n = (chunk.chunk_size * chunk.chunk_size) as usize;
            let mut vertex_index = 0;
            let mut vertex_position = Vec::with_capacity(n * 4);
            let mut atlas_indices = Vec::with_capacity(n * 4);
            let mut vertex_color = Vec::with_capacity(n * 4);
            let mut vertex_indices = Vec::with_capacity(n * 6);

            for tile in chunk.tiles.iter().filter_map(|t| t.as_ref()) {
                vertex_position.extend_from_slice(&[
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
                ]);
                vertex_color.extend_from_slice(&[tile.tint, tile.tint, tile.tint, tile.tint]);
                vertex_indices.extend_from_slice(&[
                    vertex_index,
                    vertex_index + 1,
                    vertex_index + 3,
                    vertex_index + 1,
                    vertex_index + 2,
                    vertex_index + 3,
                ]);

                vertex_index += 4;
            }

            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex_position)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vertex_color)
            .with_inserted_attribute(TILEMAP_MESH_ATLAS_INDEX_ATTR, atlas_indices)
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
