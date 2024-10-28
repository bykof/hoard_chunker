use crate::backup::models::chunk::Chunk;
use anyhow::Result;
use std::collections::HashMap;

pub type ChunkMap = HashMap<String, Chunk>;

pub trait ChunkStorage: Send + Sync + 'static {
    fn add_chunk(&mut self, chunk: Chunk) -> Result<()>;

    fn chunk_exists(&self, hash: &str) -> bool;

    fn add_chunk_if_not_exists(&mut self, chunk: Chunk) -> Result<bool>;

    fn chunk_map(&self) -> Result<ChunkMap>;

    fn load_chunk_map(&mut self, chunk_map: ChunkMap) -> Result<()>;
}

pub struct LocalChunkStorage {
    // hash -> Chunk
    chunk_map: ChunkMap,
}

impl LocalChunkStorage {
    pub fn new() -> Self {
        LocalChunkStorage {
            chunk_map: Default::default(),
        }
    }
}

impl ChunkStorage for LocalChunkStorage {
    fn add_chunk(&mut self, chunk: Chunk) -> Result<()> {
        self.chunk_map.insert(chunk.hash.clone(), chunk.clone());
        Ok(())
    }
    fn chunk_exists(&self, hash: &str) -> bool {
        self.chunk_map.contains_key(hash)
    }

    fn add_chunk_if_not_exists(&mut self, chunk: Chunk) -> Result<bool> {
        if !self.chunk_exists(&chunk.hash) {
            self.add_chunk(chunk)?;
            return Ok(true);
        }

        Ok(false)
    }
    fn chunk_map(&self) -> Result<ChunkMap> {
        Ok(self.chunk_map.clone())
    }

    fn load_chunk_map(&mut self, chunk_map: ChunkMap) -> Result<()> {
        Ok(self.chunk_map = chunk_map)
    }
}
