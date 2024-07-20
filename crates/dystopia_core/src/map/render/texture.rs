use bevy::{
    asset::{Assets, Handle},
    ecs::entity::EntityHashMap,
    prelude::{Entity, Local, Query, Res, ResMut, Resource},
    render::{
        extract_instances::ExtractedInstances,
        render_asset::RenderAssets,
        render_resource::{
            Extent3d, ImageCopyTexture, Origin3d, SamplerDescriptor, TextureAspect,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{GpuImage, Image},
    },
    utils::HashSet,
};

use crate::map::{render::ExtractedTilemap, tilemap::TilemapTilesets};

#[derive(Resource, Default)]
pub struct TilemapTextureStorage {
    to_process: EntityHashMap<TilemapTilesets>,
    pub processed: EntityHashMap<GpuImage>,
}

impl TilemapTextureStorage {
    pub fn insert(&mut self, tilemap: Entity, tilesets: TilemapTilesets) {
        self.to_process.insert(tilemap, tilesets);
    }
}

pub fn change_texture_usage(
    tilemaps_query: Query<&TilemapTilesets>,
    mut images: ResMut<Assets<Image>>,
    mut try_next_frame: Local<HashSet<Handle<Image>>>,
) {
    tilemaps_query
        .iter()
        .flat_map(|ts| ts.textures())
        .map(|t| t.handle.clone())
        .chain(try_next_frame.drain().collect::<Vec<_>>())
        .for_each(|handle| {
            let Some(image) = images.get_mut(&handle) else {
                try_next_frame.insert(handle.clone());
                return;
            };

            image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
        });
}

pub fn queue_tilemap_textures(
    tilemaps: Res<ExtractedInstances<ExtractedTilemap>>,
    mut texture_storage: ResMut<TilemapTextureStorage>,
) {
    for (entity, tilemap) in tilemaps.iter() {
        if !texture_storage.processed.contains_key(entity) {
            texture_storage
                .to_process
                .insert(*entity, tilemap.tilesets.clone());
        }
    }
}

pub fn process_textures(
    mut texture_storage: ResMut<TilemapTextureStorage>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let mut to_process = EntityHashMap::default();
    std::mem::swap(&mut to_process, &mut texture_storage.to_process);
    for (tilemap, tileset) in to_process {
        if texture_storage.processed.contains_key(&tilemap) {
            continue;
        }

        let images = tileset.textures().iter().try_fold(
            Vec::with_capacity(tileset.textures().len()),
            |mut acc, t| {
                gpu_images.get(&t.handle).map(|img| {
                    acc.push(img);
                    acc
                })
            },
        );

        let Some(images) = images else {
            texture_storage.to_process.insert(tilemap, tileset);
            continue;
        };

        let texture = render_device.create_texture(&TextureDescriptor {
            label: Some(&format!("tilemap_texture_{:?}", tilemap)),
            size: Extent3d {
                width: tileset.size().x,
                height: tileset.size().y,
                depth_or_array_layers: tileset.textures().len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let mut encoder = render_device.create_command_encoder(&Default::default());

        for (depth, image) in images.into_iter().enumerate() {
            encoder.copy_texture_to_texture(
                image.texture.as_image_copy(),
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: depth as u32,
                    },
                    aspect: TextureAspect::All,
                },
                image.texture.size(),
            )
        }

        render_queue.submit([encoder.finish()]);

        let texture_view = texture.create_view(&Default::default());
        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("tilemap_texture_sampler_{}", tilemap)),
            mag_filter: tileset.filter_mode(),
            min_filter: tileset.filter_mode(),
            mipmap_filter: tileset.filter_mode(),
            ..Default::default()
        });

        texture_storage.processed.insert(
            tilemap,
            GpuImage {
                texture,
                texture_view,
                texture_format: TextureFormat::Rgba8Unorm,
                sampler,
                size: tileset.size(),
                mip_level_count: 1,
            },
        );
    }
}
