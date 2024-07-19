use bevy::{
    app::{App, Plugin},
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::{QueryItem, ROQueryItem},
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    math::FloatOrd,
    prelude::{Changed, Commands, Component, Entity, IntoSystemConfigs, Query, Res, ResMut, With},
    render::{
        extract_instances::{ExtractInstance, ExtractInstancesPlugin},
        mesh::GpuBufferInfo,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{PipelineCache, SpecializedRenderPipelines},
        view::{Msaa, ViewUniformOffset, ViewVisibility},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    transform::components::GlobalTransform,
};

use crate::{
    map::{
        render::{
            mesh::TilemapMeshStorage,
            resource::{TilemapBindGroups, TilemapBuffers, TilemapPipeline, TilemapPipelineKey},
            texture::TilemapTextureStorage,
        },
        tilemap::{
            FlattenedTileIndex, TileAtlasIndex, TileBindedTilemap, TileRenderSize, TileTint,
            TilemapStorage, TilemapTint,
        },
    },
    simulation::MainCamera,
};

pub mod mesh;
pub mod resource;
pub mod texture;

pub struct DystopiaMapRenderPlugin;

impl Plugin for DystopiaMapRenderPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_plugins(ExtractInstancesPlugin::<ExtractedTile>::new())
            .add_plugins(ExtractInstancesPlugin::<ExtractedTilemap>::extract_visible())
            .add_render_command::<Transparent2d, DrawTilemap>()
            .add_systems(ExtractSchedule, extract_visible_tilemap_renderers)
            .add_systems(
                Render,
                (
                    mesh::prepare_tilemap_meshes,
                    mesh::prepare_tiles.after(mesh::prepare_tilemap_meshes),
                    resource::prepare_buffers,
                    texture::process_textures,
                )
                    .in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                resource::prepare_bind_groups.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(Render, queue_tilemap.in_set(RenderSet::Queue))
            // TODO why we need to init it manually?
            .init_resource::<bevy::render::extract_instances::ExtractedInstances<ExtractedTilemap>>(
            )
            // TODO why we need to init it manually?
            .init_resource::<bevy::render::extract_instances::ExtractedInstances<ExtractedTile>>()
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
/// As we are not extracting tilemaps into render world by spawning them,
/// tilemap entities are in fact not exist, and cannot be queried.
///
/// Only visible tilemaps will have a corresponding renderer.
#[derive(Component)]
pub struct TilemapRenderer(pub Entity);

pub fn extract_visible_tilemap_renderers(
    mut commands: Commands,
    tilemaps: Extract<Query<(Entity, &ViewVisibility), With<TilemapStorage>>>,
) {
    commands.spawn_batch(
        tilemaps
            .iter()
            .filter_map(|(tm, v)| {
                if v.get() {
                    Some(TilemapRenderer(dbg!(tm)))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    );
}

pub struct ExtractedTilemap {
    pub tile_render_size: TileRenderSize,
    pub transform: GlobalTransform,
    pub tint: TilemapTint,
}

impl ExtractInstance for ExtractedTilemap {
    type QueryData = (
        Read<TileRenderSize>,
        Read<GlobalTransform>,
        Read<TilemapTint>,
    );

    type QueryFilter = ();

    fn extract(item: QueryItem<'_, Self::QueryData>) -> Option<Self> {
        Some(Self {
            tile_render_size: *item.0,
            transform: *item.1,
            tint: *item.2,
        })
    }
}

pub struct ExtractedTile {
    pub binded_tilemap: Entity,
    pub index: FlattenedTileIndex,
    pub atlas_index: TileAtlasIndex,
    pub tint: TileTint,
}

impl ExtractInstance for ExtractedTile {
    type QueryData = (
        Read<TileBindedTilemap>,
        Read<FlattenedTileIndex>,
        Read<TileAtlasIndex>,
        Read<TileTint>,
    );

    type QueryFilter = Changed<FlattenedTileIndex>;

    fn extract(item: QueryItem<'_, Self::QueryData>) -> Option<Self> {
        Some(Self {
            binded_tilemap: item.0 .0,
            index: *item.1,
            atlas_index: *item.2,
            tint: *item.3,
        })
    }
}

pub fn queue_tilemap(
    renderers_query: Query<&TilemapRenderer>,
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

        for renderer in &renderers_query {
            render_phase.add(Transparent2d {
                sort_key: FloatOrd(0.),
                entity: renderer.0,
                pipeline,
                draw_function,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex(0),
            });
        }
    }
}

pub type DrawTilemap = (BindTilemapBindGroups<0>,);

pub struct BindTilemapBindGroups<const B: usize>;
impl<const B: usize> RenderCommand<Transparent2d> for BindTilemapBindGroups<B> {
    type Param = SRes<TilemapBindGroups>;

    type ViewQuery = Read<ViewUniformOffset>;

    type ItemQuery = ();

    fn render<'w>(
        item: &Transparent2d,
        view_uniform_offset: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(bind_group) = bind_groups.into_inner().bind_groups.get(&item.entity) {
            pass.set_bind_group(B, bind_group, &[view_uniform_offset.offset]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct DrawTilemapChunkMeshes;
impl RenderCommand<Transparent2d> for DrawTilemapChunkMeshes {
    type Param = SRes<TilemapMeshStorage>;

    type ViewQuery = ();

    type ItemQuery = ();

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

        for chunk in chunks.values() {
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

                    println!("Drawn!");
                }
                GpuBufferInfo::NonIndexed => {
                    pass.draw(0..gpu_mesh.vertex_count, 0..1);

                    println!("Drawn!");
                }
            }
        }
        RenderCommandResult::Success
    }
}
