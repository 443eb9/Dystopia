use bevy::{
    asset::Handle,
    math::{IVec3, UVec2, UVec3, Vec2},
    prelude::{Component, Entity},
    render::texture::Image,
    utils::HashMap,
};

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TileAtlasIndex(pub u32);

/// Rendered size of a single tile.
#[derive(Component, Debug, Default)]
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

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FlattenedTileIndex {
    pub chunk_index: IVec3,
    pub in_chunk_index: usize,
}

impl FlattenedTileIndex {
    pub fn from_direct(index: IVec3, chunk_size: u32) -> Self {
        let chunk_size = chunk_size as i32;
        let ic = index % chunk_size;
        Self {
            chunk_index: index / chunk_size,
            in_chunk_index: (ic.x + ic.y * chunk_size + ic.z * chunk_size * chunk_size) as usize,
        }
    }
}

#[derive(Default, Clone)]
pub struct Chunk {
    content: Vec<Option<Entity>>,
}

/// Stores all entities on this tilemap.
#[derive(Component, Default)]
pub struct TilemapStorage {
    chunk_size: u32,
    storage: HashMap<IVec3, Chunk>,
}

impl TilemapStorage {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            storage: Default::default(),
        }
    }

    pub fn get(&self, index: IVec3) -> Option<Entity> {
        let cs = self.chunk_size as i32;
        let chunk_index = index / cs;
        let in_chunk_index = (index % cs).as_uvec3();
        self.chunked_get(ChunkedTileIndex {
            chunk_index,
            in_chunk_index,
        })
    }

    pub fn chunked_get(&self, index: ChunkedTileIndex) -> Option<Entity> {
        self.flattened_get(index.flatten(self.chunk_size))
    }

    pub fn flattened_get(&self, index: FlattenedTileIndex) -> Option<Entity> {
        self.storage
            .get(&index.chunk_index)
            .and_then(|c| c.content[index.in_chunk_index])
    }

    pub fn get_chunk(&self, index: IVec3) -> Option<&Chunk> {
        self.storage.get(&index)
    }

    pub fn get_chunk_mut(&mut self, index: IVec3) -> Option<&mut Chunk> {
        self.storage.get_mut(&index)
    }

    pub fn set(&mut self, index: FlattenedTileIndex, tile: Entity) {
        if let Some(c) = self.storage.get_mut(&index.chunk_index) {
            c.content[index.in_chunk_index] = Some(tile)
        }
    }

    pub fn set_chunk(&mut self, index: IVec3, chunk: Chunk) {
        self.storage.insert(index, chunk);
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
    textures: Vec<TilemapTexture>,
}

impl TilemapTilesets {
    pub fn new(textures: Vec<TilemapTexture>) -> Self {
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

        Self { size, textures }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn textures(&self) -> &Vec<TilemapTexture> {
        &self.textures
    }
}
