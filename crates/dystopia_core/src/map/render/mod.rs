use bevy::{
    app::{App, Plugin, PreUpdate, Update},
    color::LinearRgba,
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::{QueryItem, ROQueryItem},
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    math::{FloatOrd, IVec3},
    prelude::{
        Commands, Component, DetectChanges, Entity, IntoSystemConfigs, Query, Ref, Res, ResMut,
        With,
    },
    render::{
        extract_instances::{ExtractInstance, ExtractInstancesPlugin},
        mesh::GpuBufferInfo,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{PipelineCache, SpecializedRenderPipelines},
        view::{InheritedVisibility, Msaa, ViewUniformOffset, Visibility},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    transform::components::GlobalTransform,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    map::{
        render::{
            mesh::TilemapMeshStorage,
            resource::{TilemapBindGroups, TilemapBuffers, TilemapPipeline, TilemapPipelineKey},
            texture::TilemapTextureStorage,
        },
        storage::Chunk,
        tilemap::{
            FlattenedTileIndex, Tile, TileRenderSize, TilemapAnimations, TilemapStorage,
            TilemapTilesets, TilemapTint,
        },
    },
    simulation::MainCamera,
};

pub mod mesh;
pub mod resource;
pub mod texture;

pub(super) struct TilemapRenderPlugin;

impl Plugin for TilemapRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractInstancesPlugin::<ExtractedTilemap>::new())
            .add_systems(Update, texture::change_texture_usage)
            .add_systems(
                PreUpdate,
                |mut tilemaps_query: Query<&mut TilemapStorage>| {
                    tilemaps_query.iter_mut().for_each(|mut t| unsafe {
                        let cell = t.as_unsafe_cell();
                        (*cell.changed_tiles).clear();
                        (*cell.changed_chunks).clear();
                    });
                },
            );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_render_command::<Transparent2d, DrawTilemap>()
            .add_systems(ExtractSchedule, extract_visible_tilemap_renderers)
            .add_systems(
                Render,
                resource::prepare_buffers.in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                (texture::queue_tilemap_textures, texture::process_textures)
                    .chain()
                    .in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                (
                    mesh::register_tilemaps_in_storage,
                    mesh::prepare_tile_mesh_data,
                    mesh::prepare_tilemap_meshes,
                )
                    .chain()
                    .in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                resource::prepare_bind_groups.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(Render, queue_tilemaps.in_set(RenderSet::Queue))
            .init_resource::<TilemapBindGroups>()
            .init_resource::<TilemapBuffers>()
            .init_resource::<TilemapMeshStorage>()
            .init_resource::<TilemapTextureStorage>();
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<TilemapPipeline>()
            .init_resource::<SpecializedRenderPipelines<TilemapPipeline>>();
    }
}

/// The renderer (or say a marker) for the target tilemap.
///
/// Only visible tilemaps will have a corresponding renderer.
#[derive(Component)]
pub struct TilemapRenderer;

pub fn extract_visible_tilemap_renderers(
    mut commands: Commands,
    // TODO visibility system
    tilemaps: Extract<Query<(Entity, &Visibility, &InheritedVisibility), With<TilemapStorage>>>,
) {
    commands.insert_or_spawn_batch(
        tilemaps
            .iter()
            .filter_map(|(tm, v, iv)| match v {
                Visibility::Inherited => {
                    if iv.get() {
                        Some((tm, TilemapRenderer))
                    } else {
                        None
                    }
                }
                Visibility::Hidden => None,
                Visibility::Visible => Some((tm, TilemapRenderer)),
            })
            .collect::<Vec<_>>(),
    );
}

pub struct ExtractedTilemap {
    pub chunk_size: u32,
    pub tile_render_size: TileRenderSize,
    pub transform: GlobalTransform,
    pub tint: LinearRgba,
    pub tilesets: TilemapTilesets,
    pub changed_animations: Option<TilemapAnimations>,
    pub changed_tiles: Vec<(FlattenedTileIndex, Option<Tile>)>,
    pub changed_chunks: Vec<(IVec3, Option<Chunk<Tile>>)>,
}

impl ExtractInstance for ExtractedTilemap {
    type QueryData = (
        Read<TileRenderSize>,
        Read<GlobalTransform>,
        Read<TilemapTint>,
        Read<TilemapTilesets>,
        Read<TilemapStorage>,
        Ref<'static, TilemapAnimations>,
    );

    type QueryFilter = ();

    fn extract(
        (tile_render_size, transform, tint, tilesets, storage, animations): QueryItem<
            '_,
            Self::QueryData,
        >,
    ) -> Option<Self> {
        Some(Self {
            tile_render_size: *tile_render_size,
            transform: *transform,
            tint: tint.to_linear(),
            tilesets: tilesets.clone(),
            chunk_size: storage.chunk_size(),
            changed_animations: if animations.is_changed() {
                Some(animations.clone())
            } else {
                None
            },
            changed_tiles: storage
                .changed_tiles()
                .par_iter()
                .map(|t| (*t, storage.flattened_get(*t).cloned()))
                .collect(),
            changed_chunks: storage
                .changed_chunks()
                .iter()
                .map(|c| (*c, storage.get_chunk(*c).cloned()))
                .collect(),
        })
    }
}

pub fn queue_tilemaps(
    tilemaps_query: Query<Entity, With<TilemapRenderer>>,
    mut render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    main_view_query: Query<Entity, With<MainCamera>>,
    pipeline: Res<TilemapPipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut sp_pipeline: ResMut<SpecializedRenderPipelines<TilemapPipeline>>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    msaa: Res<Msaa>,
) {
    let Some(draw_function) = draw_functions.read().get_id::<DrawTilemap>() else {
        return;
    };

    let pipeline = sp_pipeline.specialize(
        &pipeline_cache,
        &pipeline,
        TilemapPipelineKey {
            msaa_samples: msaa.samples(),
        },
    );

    for main_view_entity in &main_view_query {
        let Some(render_phase) = render_phases.get_mut(&main_view_entity) else {
            continue;
        };

        for renderer in &tilemaps_query {
            render_phase.add(Transparent2d {
                sort_key: FloatOrd(0.),
                entity: renderer,
                pipeline,
                draw_function,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::NONE,
            });
        }
    }
}

pub type DrawTilemap = (
    SetItemPipeline,
    BindTilemapBindGroups<0>,
    DrawTilemapChunkMeshes,
);

pub struct BindTilemapBindGroups<const B: usize>;
impl<const B: usize> RenderCommand<Transparent2d> for BindTilemapBindGroups<B> {
    type Param = (SRes<TilemapBindGroups>, SRes<TilemapBuffers>);

    type ViewQuery = Read<ViewUniformOffset>;

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        view_uniform_offset: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (bind_groups, buffers): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let (Some(bind_group), Some(individual)) = (
            bind_groups.into_inner().bind_groups.get(&item.entity),
            buffers.individual.get(&item.entity),
        ) {
            if let Some(uniform_offset) = individual.uniform_offset {
                pass.set_bind_group(B, bind_group, &[view_uniform_offset.offset, uniform_offset]);
                return RenderCommandResult::Success;
            }
        }

        RenderCommandResult::Failure
    }
}

pub struct DrawTilemapChunkMeshes;
impl RenderCommand<Transparent2d> for DrawTilemapChunkMeshes {
    type Param = SRes<TilemapMeshStorage>;

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        mesh_storage: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(chunks) = mesh_storage.into_inner().storage.get(&item.entity) else {
            return RenderCommandResult::Failure;
        };

        for chunk in chunks.chunks.values() {
            let Some(gpu_mesh) = &chunk.gpu_mesh else {
                continue;
            };

            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    count,
                    index_format,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                GpuBufferInfo::NonIndexed => pass.draw(0..gpu_mesh.vertex_count, 0..1),
            }
        }

        RenderCommandResult::Success
    }
}
