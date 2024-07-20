use std::hash::Hash;

use bevy::{
    prelude::{Deref, DerefMut},
    utils::{Entry, HashMap},
};

pub const DEFAULT_CHUNK_SIZE: u32 = 8;

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

pub trait ChunkableIndex {
    type ChunkIndex: Clone + Eq + Hash;

    fn in_chunk(&self) -> Self::ChunkIndex;
    fn in_chunk_at(&self) -> usize;
}

#[derive(Default, Clone)]
pub struct ChunkedStorage<I, T, const DIM: u32>
where
    I: ChunkableIndex,
    T: Clone,
{
    chunk_size: u32,
    storage: HashMap<I::ChunkIndex, Chunk<T>>,
}

impl<I, T, const DIM: u32> ChunkedStorage<I, T, DIM>
where
    I: ChunkableIndex,
    T: Clone,
{
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            storage: Default::default(),
        }
    }

    #[inline]
    pub fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    #[inline]
    pub fn get(&self, index: &I) -> Option<&T> {
        self.storage
            .get(&index.in_chunk())
            .and_then(|c| c[index.in_chunk_at()].as_ref())
    }

    #[inline]
    pub fn get_mut(&mut self, index: &I) -> Option<&mut T> {
        self.storage
            .get_mut(&index.in_chunk())
            .and_then(|c| c[index.in_chunk_at()].as_mut())
    }

    #[inline]
    pub fn get_chunk(&self, index: &I::ChunkIndex) -> Option<&Chunk<T>> {
        self.storage.get(index)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, index: &I::ChunkIndex) -> Option<&mut Chunk<T>> {
        self.storage.get_mut(index)
    }

    #[inline]
    pub fn set(&mut self, index: I, item: T) -> Option<T> {
        let slot = &mut self
            .storage
            .entry(index.in_chunk())
            .or_insert_with(|| Chunk(vec![None; self.chunk_size.pow(DIM) as usize]))
            [index.in_chunk_at()];
        std::mem::replace(slot, Some(item))
    }

    #[inline]
    pub fn set_chunk(&mut self, index: &I::ChunkIndex, chunk: Chunk<T>) -> Option<Chunk<T>> {
        match self.storage.entry(index.clone()) {
            Entry::Occupied(mut e) => Some(std::mem::replace(e.get_mut(), chunk)),
            Entry::Vacant(e) => {
                e.insert(chunk);
                None
            }
        }
    }
}
