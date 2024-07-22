use bevy::{
    asset::Handle,
    color::LinearRgba,
    math::{IVec3, UVec2, UVec3, Vec2},
    prelude::{Commands, Component, Deref, DerefMut, Entity},
    render::{render_resource::FilterMode, texture::Image},
};

use crate::map::{
    bundle::TileBundle,
    removal::{DespawnMe, RemoveTilemapChunk},
    storage::{Chunk, ChunkableIndex, ChunkedStorage, DEFAULT_CHUNK_SIZE},
};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct TileBindedTilemap(pub Entity);

impl Default for TileBindedTilemap {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component, Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct TileIndex {
    direct: IVec3,
    #[deref]
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

#[derive(Component, Debug, Clone, Copy)]
pub enum TileAtlasIndex {
    Static(TileStaticAtlas),
    Animated {
        anim: TileAnimation,
        offset_milisec: u32,
    },
}

impl Default for TileAtlasIndex {
    fn default() -> Self {
        Self::Static(Default::default())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TileStaticAtlas {
    pub texture: u32,
    pub atlas: u32,
    pub flip: TileFlip,
}

impl From<(u32, u32)> for TileStaticAtlas {
    fn from(value: (u32, u32)) -> Self {
        Self {
            texture: value.0,
            atlas: value.1,
            flip: TileFlip::NONE,
        }
    }
}

impl From<(u32, u32, TileFlip)> for TileStaticAtlas {
    fn from(value: (u32, u32, TileFlip)) -> Self {
        Self {
            texture: value.0,
            atlas: value.1,
            flip: value.2,
        }
    }
}

impl TileStaticAtlas {
    pub fn encode(self) -> Self {
        Self {
            texture: self.texture,
            atlas: self.atlas ^ (self.flip.bits() << 30),
            flip: TileFlip::NONE,
        }
    }

    pub fn decode(self) -> Self {
        Self {
            texture: self.texture,
            atlas: self.atlas & (0x3FFF_FFFF),
            flip: TileFlip::from_bits(self.atlas >> 30).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TileAnimation {
    pub(crate) start: usize,
    pub(crate) len: usize,
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct TileFlip: u32 {
        const NONE       = 0b00;
        const HORIZONTAL = 0b10;
        const VERTICAL   = 0b01;
        const BOTH       = 0b11;
    }
}

#[derive(Component, Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct TileTint(pub LinearRgba);

/// Rendered size of a single tile.
#[derive(Component, Debug, Default, Clone, Copy, Deref, DerefMut)]
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
#[derive(Component)]
pub struct TilemapStorage {
    tilemap: Entity,
    internal: ChunkedStorage<FlattenedTileIndex, Entity, 3>,
}

impl Default for TilemapStorage {
    fn default() -> Self {
        Self {
            tilemap: Entity::PLACEHOLDER,
            internal: ChunkedStorage::new(DEFAULT_CHUNK_SIZE),
        }
    }
}

impl TilemapStorage {
    pub fn new(binded_tilemap: Entity, chunk_size: u32) -> Self {
        Self {
            tilemap: binded_tilemap,
            internal: ChunkedStorage::new(chunk_size),
        }
    }

    #[inline]
    pub fn chunk_size(&self) -> u32 {
        self.internal.chunk_size()
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
        self.internal.get(&index).cloned()
    }

    #[inline]
    pub fn get_chunk(&self, index: IVec3) -> Option<&Chunk<Entity>> {
        self.internal.get_chunk(&index)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, index: IVec3) -> Option<&mut Chunk<Entity>> {
        self.internal.get_chunk_mut(&index)
    }

    #[inline]
    pub fn set(&mut self, commands: &mut Commands, tile: TileBundle) -> Option<Entity> {
        self.internal
            .set(tile.index.flattend, commands.spawn(tile).id())
    }

    #[inline]
    pub fn set_chunk(&mut self, index: IVec3, chunk: Chunk<Entity>) {
        self.internal.set_chunk(&index, chunk);
    }

    #[inline]
    pub fn remove(&mut self, commands: &mut Commands, index: IVec3) -> Option<Entity> {
        self.flattened_remove(
            commands,
            FlattenedTileIndex::from_direct(index, self.chunk_size()),
        )
    }

    #[inline]
    pub fn chunked_remove(
        &mut self,
        commands: &mut Commands,
        index: ChunkedTileIndex,
    ) -> Option<Entity> {
        self.flattened_remove(commands, index.flatten(self.chunk_size()))
    }

    #[inline]
    pub fn flattened_remove(
        &mut self,
        commands: &mut Commands,
        index: FlattenedTileIndex,
    ) -> Option<Entity> {
        self.internal.remove(&index).inspect(|e| {
            commands.entity(*e).insert(DespawnMe);
        })
    }

    #[inline]
    pub fn remove_chunk(&mut self, commands: &mut Commands, index: IVec3) -> Option<Chunk<Entity>> {
        commands.spawn(RemoveTilemapChunk {
            tilemap: self.tilemap,
            index,
        });
        self.internal.remove_chunk(&index)
    }

    #[inline]
    pub fn despawn(&self, commands: &mut Commands) {
        commands.entity(self.tilemap).insert(DespawnMe);
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

#[derive(Component, Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct TilemapTint(pub LinearRgba);

/// Layout: `[fps, frame_1_tex, frame_1_atl, frame_2_tex, frame_2_atl, fps, frame_1_tex, frame_1_atl, ...]`
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct TilemapAnimations(Vec<u32>);

impl Default for TilemapAnimations {
    fn default() -> Self {
        // A dummy value. This will force the gpu-side buffer to be created.
        // If leave empty, [`RawBufferVec::write_buffer`] will not take affect.
        Self([0].into())
    }
}

impl TilemapAnimations {
    pub fn register(
        &mut self,
        animation: impl IntoIterator<IntoIter: Iterator<Item = impl Into<TileStaticAtlas>>>,
        fps: u32,
    ) -> TileAnimation {
        let animation = animation
            .into_iter()
            .map(Into::into)
            .map(|f| f.encode())
            .collect::<Vec<_>>();
        self.push(fps);
        let anim = TileAnimation {
            start: self.len(),
            len: animation.len(),
        };
        self.extend(
            animation
                .into_iter()
                .flat_map(|frame| [frame.texture, frame.atlas]),
        );
        anim
    }
}
