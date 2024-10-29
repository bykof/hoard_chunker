use crate::backup::models::backup_config::BackupConfig;
use crate::backup::models::chunk::Chunk;
use crate::backup::services::chunk_reader_writer::ChunkReaderWriter;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ChunkMap = HashMap<String, Chunk>;

pub trait ChunkStorage: Send + Sync {
    fn add_chunk(&self, chunk: Chunk) -> Result<()>;

    fn chunk_exists(&self, hash: &str) -> bool;

    fn add_chunk_if_not_exists(&self, chunk: Chunk) -> Result<bool>;

    fn chunk_map(&self) -> Result<ChunkMap>;

    fn load_chunk_map(&self, chunk_map: ChunkMap) -> Result<()>;

    fn store_chunk(&self, hash: &str, data: &Vec<u8>) -> Result<()>;

    fn load_chunk(&self, hash: &str) -> Result<Vec<u8>>;
}

pub struct LocalChunkStorage {
    backup_config: Arc<BackupConfig>,
    chunk_map: Arc<Mutex<ChunkMap>>,
}

impl LocalChunkStorage {
    pub fn new(backup_config: Arc<BackupConfig>) -> Self {
        LocalChunkStorage {
            backup_config,
            chunk_map: Default::default(),
        }
    }
}

impl ChunkStorage for LocalChunkStorage {
    fn add_chunk(&self, chunk: Chunk) -> Result<()> {
        self.chunk_map
            .lock()
            .unwrap()
            .insert(chunk.hash.clone(), chunk.clone());
        Ok(())
    }
    fn chunk_exists(&self, hash: &str) -> bool {
        self.chunk_map.lock().unwrap().contains_key(hash)
    }

    fn add_chunk_if_not_exists(&self, chunk: Chunk) -> Result<bool> {
        if !self.chunk_exists(&chunk.hash) {
            self.add_chunk(chunk)?;
            return Ok(true);
        }

        Ok(false)
    }
    fn chunk_map(&self) -> Result<ChunkMap> {
        Ok(self.chunk_map.lock().unwrap().clone())
    }

    fn load_chunk_map(&self, chunk_map: ChunkMap) -> Result<()> {
        Ok(*self.chunk_map.lock().unwrap() = chunk_map)
    }

    fn store_chunk(&self, hash: &str, data: &Vec<u8>) -> Result<()> {
        let chunk_reader_writer = ChunkReaderWriter::new();
        chunk_reader_writer.write_chunk(hash, data, self.backup_config.output_path.as_ref())
    }

    fn load_chunk(&self, hash: &str) -> Result<Vec<u8>> {
        let chunk_reader_writer = ChunkReaderWriter::new();
        chunk_reader_writer.read_chunk(hash, self.backup_config.input_path.as_ref())
    }
}
