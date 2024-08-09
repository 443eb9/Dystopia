use bevy::{
    asset::{DirectAssetAccessExt, Handle},
    color::ColorToComponents,
    ecs::entity::EntityHashMap,
    math::{Mat4, UVec2, Vec2, Vec4},
    prelude::{Entity, FromWorld, Query, Res, ResMut, Resource, With, World},
    render::{
        extract_instances::ExtractedInstances,
        globals::{GlobalsBuffer, GlobalsUniform},
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState,
            BufferUsages, BufferVec, ColorTargetState, ColorWrites, FragmentState,
            MultisampleState, RawBufferVec, RenderPipelineDescriptor, SamplerBindingType, Shader,
            ShaderStages, ShaderType, SpecializedRenderPipeline, TextureFormat, TextureSampleType,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ViewUniform, ViewUniforms},
    },
};

use bevy::render::render_resource::binding_types as binding;

use crate::map::render::{texture::TilemapTextureStorage, ExtractedTilemap, TilemapRenderer};

#[derive(Resource)]
pub struct TilemapPipeline {
    pub shader: Handle<Shader>,
    pub layout: BindGroupLayout,
}

impl FromWorld for TilemapPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let shader = world.load_asset("shaders/tilemap/tilemap.wgsl");

        let layout = render_device.create_bind_group_layout(
            "tilemap_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    binding::uniform_buffer::<ViewUniform>(true),
                    binding::uniform_buffer::<GlobalsUniform>(false),
                    binding::uniform_buffer::<TilemapUniform>(true),
                    binding::texture_2d_array(TextureSampleType::Float { filterable: true }),
                    binding::sampler(SamplerBindingType::Filtering),
                    binding::storage_buffer_read_only::<TilemapRenderTextureDescriptor>(false),
                    binding::storage_buffer_read_only::<u32>(false),
                ),
            ),
        );

        Self { shader, layout }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilemapPipelineKey {
    pub msaa_samples: u32,
}

impl SpecializedRenderPipeline for TilemapPipeline {
    type Key = TilemapPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("tilemap_pipeline".into()),
            layout: vec![self.layout.clone()],
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout::from_vertex_formats(
                    VertexStepMode::Vertex,
                    vec![
                        // position
                        VertexFormat::Float32x3,
                        // color
                        VertexFormat::Float32x4,
                        // atlas_index
                        VertexFormat::Uint32x4,
                        // tile_index
                        VertexFormat::Sint32x3,
                    ],
                )],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples,
                ..Default::default()
            },
        }
    }
}

#[derive(ShaderType)]
pub struct TilemapUniform {
    pub tile_render_size: Vec2,
    pub world_from_model: Mat4,
    pub tint: Vec4,
}

#[derive(ShaderType)]
pub struct TilemapRenderTextureDescriptor {
    pub tile_count: UVec2,
}

pub struct TilemapIndividualRenderData {
    pub uniform_offset: Option<u32>,
    pub texture_desc: BufferVec<TilemapRenderTextureDescriptor>,
    pub animations: RawBufferVec<u32>,
}

impl Default for TilemapIndividualRenderData {
    fn default() -> Self {
        Self {
            uniform_offset: Default::default(),
            texture_desc: BufferVec::new(BufferUsages::STORAGE),
            animations: RawBufferVec::new(BufferUsages::STORAGE),
        }
    }
}

#[derive(Resource)]
pub struct TilemapBuffers {
    pub uniform: BufferVec<TilemapUniform>,
    pub individual: EntityHashMap<TilemapIndividualRenderData>,
}

impl Default for TilemapBuffers {
    fn default() -> Self {
        Self {
            uniform: BufferVec::new(BufferUsages::UNIFORM),
            individual: Default::default(),
        }
    }
}

#[derive(Resource, Default)]
pub struct TilemapBindGroups {
    pub bind_groups: EntityHashMap<BindGroup>,
}

pub fn prepare_buffers(
    tilemaps: Res<ExtractedInstances<ExtractedTilemap>>,
    mut buffers: ResMut<TilemapBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    buffers.uniform.clear();

    for (entity, tilemap) in tilemaps.iter() {
        let offset = buffers.uniform.push(TilemapUniform {
            tile_render_size: tilemap.tile_render_size.0,
            world_from_model: tilemap.transform.compute_matrix(),
            tint: tilemap.tint.to_vec4(),
        });

        let individual = buffers.individual.entry(*entity).or_default();
        individual.uniform_offset = Some(offset as u32);

        individual.texture_desc.clear();
        tilemap.tilesets.textures().iter().for_each(|t| {
            individual
                .texture_desc
                .push(TilemapRenderTextureDescriptor {
                    tile_count: t.desc.size / t.desc.tile_size,
                });
        });
        individual
            .texture_desc
            .write_buffer(&render_device, &render_queue);

        if let Some(changed_animations) = &tilemap.changed_animations {
            individual.animations.clear();
            *individual.animations.values_mut() = changed_animations.to_vec();
            individual
                .animations
                .write_buffer(&render_device, &render_queue);
        }
    }

    buffers.uniform.write_buffer(&render_device, &render_queue);
}

pub fn prepare_bind_groups(
    tilemaps_query: Query<Entity, With<TilemapRenderer>>,
    pipeline: Res<TilemapPipeline>,
    view_uniforms: Res<ViewUniforms>,
    global_uniforms: Res<GlobalsBuffer>,
    tilemap_buffers: Res<TilemapBuffers>,
    tilemap_textures: Res<TilemapTextureStorage>,
    mut bind_groups: ResMut<TilemapBindGroups>,
    render_device: Res<RenderDevice>,
) {
    let (Some(view_uniforms), Some(global_uniforms)) = (
        view_uniforms.uniforms.binding(),
        global_uniforms.buffer.binding(),
    ) else {
        return;
    };

    for tilemap in &tilemaps_query {
        let (Some(tilemap_uniforms), Some(tilemap_texture), Some(individual)) = (
            tilemap_buffers.uniform.binding(),
            tilemap_textures.processed.get(&tilemap),
            tilemap_buffers.individual.get(&tilemap),
        ) else {
            continue;
        };

        let (Some(texture_desc), Some(animations)) = (
            individual.texture_desc.binding(),
            individual.animations.binding(),
        ) else {
            continue;
        };

        let bind_group = render_device.create_bind_group(
            format!("tilemap_bind_group_{:?}", tilemap).as_str(),
            &pipeline.layout,
            &BindGroupEntries::sequential((
                view_uniforms.clone(),
                global_uniforms.clone(),
                tilemap_uniforms,
                &tilemap_texture.texture_view,
                &tilemap_texture.sampler,
                texture_desc,
                animations,
            )),
        );

        bind_groups.bind_groups.insert(tilemap, bind_group);
    }
}
