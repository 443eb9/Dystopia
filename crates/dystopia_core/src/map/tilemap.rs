use bevy::{
    asset::Handle,
    color::Color,
    math::{IVec2, UVec2, UVec3, Vec2},
    prelude::{Component, Deref, DerefMut},
    render::{render_resource::FilterMode, texture::Image},
    utils::HashSet,
};

use crate::util::chunking::{Chunk, ChunkStorageIndex, ChunkedStorage, DEFAULT_CHUNK_SIZE};

#[derive(Clone)]
pub struct Tile {
    pub index: TileIndex,
    pub atlas_index: TileAtlasIndex,
    pub tint: Color,
    pub visible: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            index: Default::default(),
            atlas_index: Default::default(),
            tint: Default::default(),
            visible: true,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct TileIndex {
    direct: IVec2,
    #[deref]
    flattened: FlattenedTileIndex,
}

impl TileIndex {
    pub fn new(direct: IVec2, flattened: FlattenedTileIndex) -> Self {
        Self { direct, flattened }
    }

    pub fn from_direct(direct: IVec2, chunk_size: u32) -> Self {
        Self {
            direct,
            flattened: FlattenedTileIndex::from_direct(direct, chunk_size),
        }
    }

    pub fn direct(&self) -> IVec2 {
        self.direct
    }

    pub fn flattened(&self) -> FlattenedTileIndex {
        self.flattened
    }
}

#[derive(Debug, Clone, Copy)]
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

/// Rendered size of a single tile.
#[derive(Component, Debug, Default, Clone, Copy, Deref, DerefMut)]
pub struct TileRenderSize(pub Vec2);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChunkedTileIndex {
    pub chunk_index: IVec2,
    pub in_chunk_index: UVec3,
}

/// The fastest index for looking up tiles.
pub type FlattenedTileIndex = ChunkStorageIndex<IVec2>;

impl FlattenedTileIndex {
    #[inline]
    pub fn from_direct(index: IVec2, chunk_size: u32) -> Self {
        let chunk_size = chunk_size as i32;
        let ic = (index % chunk_size).abs();
        Self {
            in_chunk: index / chunk_size,
            in_chunk_at: (ic.x + ic.y * chunk_size) as usize,
        }
    }

    #[inline]
    pub fn from_chunked(index: ChunkedTileIndex, chunk_size: u32) -> Self {
        FlattenedTileIndex {
            in_chunk: index.chunk_index,
            in_chunk_at: (index.in_chunk_index.x
                + index.in_chunk_index.y * chunk_size
                + index.in_chunk_index.z * chunk_size * chunk_size)
                as usize,
        }
    }
}

/// Stores all entities on this tilemap.
#[derive(Component)]
pub struct TilemapStorage {
    internal: ChunkedStorage<IVec2, Tile>,
    changed_tiles: HashSet<FlattenedTileIndex>,
    changed_chunks: HashSet<IVec2>,
}

pub struct UnsafePubTilemapStorageCell {
    pub internal: *mut ChunkedStorage<IVec2, Tile>,
    pub changed_tiles: *mut HashSet<FlattenedTileIndex>,
    pub changed_chunks: *mut HashSet<IVec2>,
}

impl Default for TilemapStorage {
    fn default() -> Self {
        Self::new(DEFAULT_CHUNK_SIZE)
    }
}

impl TilemapStorage {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            internal: ChunkedStorage::new(chunk_size),
            changed_chunks: Default::default(),
            changed_tiles: Default::default(),
        }
    }

    #[inline]
    pub fn chunk_size(&self) -> u32 {
        self.internal.chunk_size()
    }

    #[inline]
    pub fn changed_tiles(&self) -> &HashSet<FlattenedTileIndex> {
        &self.changed_tiles
    }

    #[inline]
    pub fn changed_chunks(&self) -> &HashSet<IVec2> {
        &self.changed_chunks
    }

    #[inline]
    pub unsafe fn as_unsafe_cell_readonly(&self) -> UnsafePubTilemapStorageCell {
        UnsafePubTilemapStorageCell {
            internal: std::ptr::from_ref(&self.internal).cast_mut(),
            changed_tiles: std::ptr::from_ref(&self.changed_tiles).cast_mut(),
            changed_chunks: std::ptr::from_ref(&self.changed_chunks).cast_mut(),
        }
    }

    #[inline]
    pub unsafe fn as_unsafe_cell(&mut self) -> UnsafePubTilemapStorageCell {
        UnsafePubTilemapStorageCell {
            internal: std::ptr::from_mut(&mut self.internal),
            changed_tiles: std::ptr::from_mut(&mut self.changed_tiles),
            changed_chunks: std::ptr::from_mut(&mut self.changed_chunks),
        }
    }

    #[inline]
    pub fn get(&self, index: IVec2) -> Option<&Tile> {
        self.flattened_get(FlattenedTileIndex::from_direct(index, self.chunk_size()))
    }

    #[inline]
    pub fn chunked_get(&self, index: ChunkedTileIndex) -> Option<&Tile> {
        self.flattened_get(FlattenedTileIndex::from_chunked(index, self.chunk_size()))
    }

    #[inline]
    pub fn flattened_get(&self, index: FlattenedTileIndex) -> Option<&Tile> {
        self.internal.get(&index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: IVec2) -> Option<&mut Tile> {
        self.flattened_get_mut(FlattenedTileIndex::from_direct(index, self.chunk_size()))
    }

    #[inline]
    pub fn chunked_get_mut(&mut self, index: ChunkedTileIndex) -> Option<&mut Tile> {
        self.flattened_get_mut(FlattenedTileIndex::from_chunked(index, self.chunk_size()))
    }

    #[inline]
    pub fn flattened_get_mut(&mut self, index: FlattenedTileIndex) -> Option<&mut Tile> {
        self.changed_tiles.insert(index);
        self.internal.get_mut(&index)
    }

    #[inline]
    pub fn set(&mut self, tile: Tile) -> Option<Tile> {
        self.changed_tiles.insert(tile.index.flattened);
        self.internal.set(tile.index.flattened, tile)
    }

    #[inline]
    pub fn remove(&mut self, index: IVec2) -> Option<Tile> {
        self.flattened_remove(FlattenedTileIndex::from_direct(index, self.chunk_size()))
    }

    #[inline]
    pub fn chunked_remove(&mut self, index: ChunkedTileIndex) -> Option<Tile> {
        self.flattened_remove(FlattenedTileIndex::from_chunked(index, self.chunk_size()))
    }

    #[inline]
    pub fn flattened_remove(&mut self, index: FlattenedTileIndex) -> Option<Tile> {
        self.changed_tiles.insert(index);
        self.internal.remove(&index)
    }

    #[inline]
    pub fn get_chunk(&self, index: IVec2) -> Option<&Chunk<Tile>> {
        self.internal.get_chunk(&index)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, index: IVec2) -> Option<&mut Chunk<Tile>> {
        self.changed_chunks.insert(index);
        self.internal.get_chunk_mut(&index)
    }

    #[inline]
    pub fn set_chunk(&mut self, index: IVec2, chunk: Chunk<Tile>) -> Option<Chunk<Tile>> {
        self.changed_chunks.insert(index);
        self.internal.set_chunk(index, chunk)
    }

    #[inline]
    pub fn remove_chunk(&mut self, index: IVec2) -> Option<Chunk<Tile>> {
        self.changed_chunks.insert(index);
        self.internal.remove_chunk(&index)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.changed_chunks.extend(self.internal.keys());
        self.internal.clear();
    }
}

impl From<ChunkedStorage<IVec2, Tile>> for TilemapStorage {
    fn from(value: ChunkedStorage<IVec2, Tile>) -> Self {
        Self {
            changed_chunks: value.keys().cloned().collect(),
            internal: value,
            changed_tiles: Default::default(),
        }
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
    pub(crate) size: UVec2,
    pub(crate) filter_mode: FilterMode,
    pub(crate) textures: Vec<TilemapTexture>,
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
pub struct TilemapTint(pub Color);

/// Layout: `[fps, frame_1_tex, frame_1_atl, frame_2_tex, frame_2_atl, fps, frame_1_tex, frame_1_atl, ...]`
#[derive(Component, Debug, Clone)]
pub struct TilemapAnimations(Vec<u32>);

impl From<Vec<u32>> for TilemapAnimations {
    fn from(value: Vec<u32>) -> Self {
        Self(value)
    }
}

impl Default for TilemapAnimations {
    fn default() -> Self {
        // A dummy value. This will force the gpu-side buffer to be created.
        // If leave empty, [`RawBufferVec::write_buffer`] will not take affect.
        Self([0].into())
    }
}

impl TilemapAnimations {
    pub fn bytes(&self) -> &Vec<u32> {
        &self.0
    }

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
        self.0.push(fps);
        let anim = TileAnimation {
            start: self.0.len(),
            len: animation.len(),
        };
        self.0.extend(
            animation
                .into_iter()
                .flat_map(|frame| [frame.texture, frame.atlas]),
        );
        anim
    }
}
