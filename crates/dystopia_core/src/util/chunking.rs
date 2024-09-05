use std::hash::Hash;

use bevy::{
    prelude::{Deref, DerefMut},
    utils::{Entry, HashMap},
};
use hashbrown::hash_map::{Iter, IterMut};

pub trait ChunkIndex: Clone + PartialEq + Eq + Hash {
    const DIM: u32;
}

macro_rules! impl_chunk_index {
    ($ty: ty, $dim: literal) => {
        impl ChunkIndex for $ty {
            const DIM: u32 = $dim;
        }
    };
}

impl_chunk_index!(bevy::math::IVec2, 2);
impl_chunk_index!(bevy::math::IVec3, 3);
impl_chunk_index!(bevy::math::IVec4, 4);

impl_chunk_index!(bevy::math::UVec2, 2);
impl_chunk_index!(bevy::math::UVec3, 3);
impl_chunk_index!(bevy::math::UVec4, 4);

pub const DEFAULT_CHUNK_SIZE: u32 = 16;

#[derive(Clone, Deref, DerefMut)]
pub struct Chunk<T>(Vec<Option<T>>);

impl<T> Chunk<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl<T> From<Vec<Option<T>>> for Chunk<T> {
    fn from(value: Vec<Option<T>>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkStorageIndex<I: Hash> {
    pub in_chunk: I,
    pub in_chunk_at: usize,
}

#[derive(Deref, DerefMut)]
pub struct ChunkedStorage<I, T>
where
    I: ChunkIndex,
    T: Clone,
{
    chunk_size: u32,
    #[deref]
    storage: HashMap<I, Chunk<T>>,
}

impl<I, T> ChunkedStorage<I, T>
where
    I: ChunkIndex,
    T: Clone,
{
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            storage: Default::default(),
        }
    }

    pub fn new_init(chunk_size: u32, storage: HashMap<I, Chunk<T>>) -> Self {
        Self {
            chunk_size,
            storage,
        }
    }

    #[inline]
    pub fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    #[inline]
    pub fn contains(&self, index: &ChunkStorageIndex<I>) -> bool {
        self.storage
            .get(&index.in_chunk)
            .is_some_and(|c| c[index.in_chunk_at].is_some())
    }

    #[inline]
    pub fn contains_chunk(&self, index: &I) -> bool {
        self.storage.contains_key(index)
    }

    #[inline]
    pub fn get(&self, index: &ChunkStorageIndex<I>) -> Option<&T> {
        self.storage
            .get(&index.in_chunk)
            .and_then(|c| c[index.in_chunk_at].as_ref())
    }

    #[inline]
    pub fn get_mut(&mut self, index: &ChunkStorageIndex<I>) -> Option<&mut T> {
        self.storage
            .get_mut(&index.in_chunk)
            .and_then(|c| c[index.in_chunk_at].as_mut())
    }

    #[inline]
    pub fn get_or_insert(&mut self, index: ChunkStorageIndex<I>, item: T) -> &mut T {
        self.storage
            .entry(index.in_chunk)
            .or_insert_with(|| Chunk(vec![None; self.chunk_size.pow(I::DIM) as usize]))
            [index.in_chunk_at]
            .get_or_insert(item)
    }

    #[inline]
    pub fn get_or_insert_with(&mut self, index: ChunkStorageIndex<I>, f: impl Fn() -> T) -> &mut T {
        self.storage
            .entry(index.in_chunk)
            .or_insert_with(|| Chunk(vec![None; self.chunk_size.pow(I::DIM) as usize]))
            [index.in_chunk_at]
            .get_or_insert_with(f)
    }

    #[inline]
    pub fn get_chunk(&self, index: &I) -> Option<&Chunk<T>> {
        self.storage.get(index)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, index: &I) -> Option<&mut Chunk<T>> {
        self.storage.get_mut(index)
    }

    #[inline]
    pub fn get_chunk_or_insert(&mut self, index: I, chunk: Chunk<T>) -> &mut Chunk<T> {
        self.storage.entry(index).or_insert(chunk)
    }

    #[inline]
    pub fn get_chunk_or_insert_with(
        &mut self,
        index: I,
        f: impl Fn() -> Chunk<T>,
    ) -> &mut Chunk<T> {
        self.storage.entry(index).or_insert_with(f)
    }

    #[inline]
    pub fn remove(&mut self, index: &ChunkStorageIndex<I>) -> Option<T> {
        self.storage
            .entry(index.in_chunk.clone())
            .or_insert_with(|| Chunk(vec![None; self.chunk_size.pow(I::DIM) as usize]))
            [index.in_chunk_at]
            .take()
    }

    #[inline]
    pub fn remove_chunk(&mut self, index: &I) -> Option<Chunk<T>> {
        self.storage.remove(index)
    }

    #[inline]
    pub fn set(&mut self, index: ChunkStorageIndex<I>, item: T) -> Option<T> {
        let slot = &mut self
            .storage
            .entry(index.in_chunk)
            .or_insert_with(|| Chunk(vec![None; self.chunk_size.pow(I::DIM) as usize]))
            [index.in_chunk_at];
        std::mem::replace(slot, Some(item))
    }

    #[inline]
    pub fn set_chunk(&mut self, index: I, chunk: Chunk<T>) -> Option<Chunk<T>> {
        match self.storage.entry(index) {
            Entry::Occupied(mut e) => Some(std::mem::replace(e.get_mut(), chunk)),
            Entry::Vacant(e) => {
                e.insert(chunk);
                None
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.storage.clear();
    }

    #[inline]
    pub fn iter(&self) -> Iter<I, Chunk<T>> {
        self.storage.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<I, Chunk<T>> {
        self.storage.iter_mut()
    }
}

impl<I, T> Clone for ChunkedStorage<I, T>
where
    I: ChunkIndex + Clone,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            chunk_size: self.chunk_size.clone(),
            storage: self.storage.clone(),
        }
    }
}

impl<I, T> Default for ChunkedStorage<I, T>
where
    I: ChunkIndex,
    T: Clone,
{
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            storage: Default::default(),
        }
    }
}
