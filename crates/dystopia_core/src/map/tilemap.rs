use bevy::{
    asset::Handle,
    color::LinearRgba,
    math::{IVec3, UVec2, UVec3, Vec2},
    prelude::{Commands, Component, Deref, DerefMut, Entity},
    render::{render_resource::FilterMode, texture::Image},
};

use crate::map::{
    bundle::TileBundle,
    storage::{Chunk, ChunkableIndex, ChunkedStorage, DEFAULT_CHUNK_SIZE},
};

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TileIndex {
    direct: IVec3,
    flattend: FlattenedTileIndex,
}

impl TileIndex {
    pub fn new(direct: IVec3, chunk_size: u32) -> Self {
        assert!(
            matches!(direct.element_sum(), 1 | 2),
            "Invalid tile index {}. The element-wise sum of index must be 1 or 2.",
            direct
        );

        Self {
            direct,
            flattend: FlattenedTileIndex::from_direct(direct, chunk_size),
        }
    }

    pub fn direct(&self) -> IVec3 {
        self.direct
    }

    pub fn flattend(&self) -> FlattenedTileIndex {
        self.flattend
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileAtlasIndex {
    Static { texture: u32, atlas: u32 },
    // TODO animated
}

impl Default for TileAtlasIndex {
    fn default() -> Self {
        Self::Static {
            texture: 0,
            atlas: 0,
        }
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TileTint(pub LinearRgba);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileBindedTilemap(pub Entity);

impl Default for TileBindedTilemap {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

/// Rendered size of a single tile.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TileRenderSize(pub Vec2);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChunkedTileIndex {
    pub chunk_index: IVec3,
    pub in_chunk_index: UVec3,
}

impl ChunkedTileIndex {
    pub fn flatten(self, chunk_size: u32) -> FlattenedTileIndex {
        FlattenedTileIndex {
            chunk_index: self.chunk_index,
            in_chunk_index: (self.in_chunk_index.x
                + self.in_chunk_index.y * chunk_size
                + self.in_chunk_index.z * chunk_size * chunk_size)
                as usize,
        }
    }
}

/// The fastest index for looking up tiles.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FlattenedTileIndex {
    pub chunk_index: IVec3,
    pub in_chunk_index: usize,
}

impl ChunkableIndex for FlattenedTileIndex {
    type ChunkIndex = IVec3;

    #[inline]
    fn in_chunk(&self) -> Self::ChunkIndex {
        self.chunk_index
    }

    #[inline]
    fn in_chunk_at(&self) -> usize {
        self.in_chunk_index
    }
}

impl FlattenedTileIndex {
    #[inline]
    pub fn from_direct(index: IVec3, chunk_size: u32) -> Self {
        let chunk_size = chunk_size as i32;
        let ic = (index % chunk_size).abs();
        Self {
            chunk_index: index / chunk_size,
            in_chunk_index: (ic.x + ic.y * chunk_size + ic.z * chunk_size * chunk_size) as usize,
        }
    }
}

/// Stores all entities on this tilemap.
#[derive(Component, Deref, DerefMut)]
pub struct TilemapStorage(#[deref] ChunkedStorage<FlattenedTileIndex, Entity, 3>);

impl Default for TilemapStorage {
    fn default() -> Self {
        Self(ChunkedStorage::new(DEFAULT_CHUNK_SIZE))
    }
}

impl TilemapStorage {
    pub fn new(chunk_size: u32) -> Self {
        Self(ChunkedStorage::new(chunk_size))
    }

    #[inline]
    pub fn chunk_size(&self) -> u32 {
        self.0.chunk_size()
    }

    #[inline]
    pub fn get(&self, index: IVec3) -> Option<Entity> {
        self.flattened_get(FlattenedTileIndex::from_direct(index, self.chunk_size()))
    }

    #[inline]
    pub fn chunked_get(&self, index: ChunkedTileIndex) -> Option<Entity> {
        self.flattened_get(index.flatten(self.chunk_size()))
    }

    #[inline]
    pub fn flattened_get(&self, index: FlattenedTileIndex) -> Option<Entity> {
        self.0.get(&index).cloned()
    }

    #[inline]
    pub fn get_chunk(&self, index: IVec3) -> Option<&Chunk<Entity>> {
        self.0.get_chunk(&index)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, index: IVec3) -> Option<&mut Chunk<Entity>> {
        self.0.get_chunk_mut(&index)
    }

    #[inline]
    pub fn set(&mut self, commands: &mut Commands, tile: TileBundle) -> Option<Entity> {
        self.0.set(tile.index.flattend, commands.spawn(tile).id())
    }

    #[inline]
    pub fn set_chunk(&mut self, index: IVec3, chunk: Chunk<Entity>) {
        self.0.set_chunk(&index, chunk);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TilemapTextureDescriptor {
    pub size: UVec2,
    pub tile_size: UVec2,
}

#[derive(Debug, Default, Clone)]
pub struct TilemapTexture {
    pub handle: Handle<Image>,
    pub desc: TilemapTextureDescriptor,
}

#[derive(Component, Debug, Default, Clone)]
pub struct TilemapTilesets {
    size: UVec2,
    filter_mode: FilterMode,
    textures: Vec<TilemapTexture>,
}

impl TilemapTilesets {
    pub fn new(textures: Vec<TilemapTexture>, filter_mode: FilterMode) -> Self {
        assert_ne!(
            textures.len(),
            0,
            "Invalid texture: Length must be larger than 0."
        );

        let mut size = UVec2::default();
        textures.iter().for_each(|t| {
            size = size.max(t.desc.size);
            assert_eq!(
                t.desc.size % t.desc.tile_size,
                UVec2::ZERO,
                "Invalid descriptor: `size` must be divisible by `tile_size`."
            );
        });

        Self {
            size,
            textures,
            filter_mode,
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn textures(&self) -> &Vec<TilemapTexture> {
        &self.textures
    }

    pub fn filter_mode(&self) -> FilterMode {
        self.filter_mode
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct TilemapTint(pub LinearRgba);
