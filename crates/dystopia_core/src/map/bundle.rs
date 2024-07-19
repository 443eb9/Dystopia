use bevy::{
    prelude::Bundle,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
};

use crate::map::tilemap::{FlattenedTileIndex, TileAtlasIndex, TileRenderSize, TilemapStorage};

#[derive(Bundle, Default)]
pub struct TilemapBundle {
    pub tile_render_size: TileRenderSize,
    pub storgae: TilemapStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
}

#[derive(Bundle, Default)]
pub struct TileBundle {
    pub index: FlattenedTileIndex,
    pub atlas_index: TileAtlasIndex,
    pub visibility: Visibility,
}
