use bevy::{
    prelude::Bundle,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
};

use crate::map::tilemap::{
    TileRenderSize, TilemapAnimations, TilemapStorage, TilemapTilesets, TilemapTint,
};

#[derive(Bundle, Default)]
pub struct TilemapBundle {
    pub tile_render_size: TileRenderSize,
    pub storgae: TilemapStorage,
    pub tint: TilemapTint,
    pub tilesets: TilemapTilesets,
    pub animations: TilemapAnimations,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
}
