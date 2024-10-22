use anyhow::Result;
use fastcdc::v2020::ChunkData;

use super::{backup_config::BackupConfig, backup_hash::split_hash_as_path, chunk_table::Chunk};

pub struct ChunkWriter<'a> {
    backup_config: &'a BackupConfig,
}

impl ChunkWriter<'_> {
    pub fn new<'a>(backup_config: &'a BackupConfig) -> ChunkWriter<'a> {
        ChunkWriter { backup_config }
    }
    pub fn write(&self, chunk: &Chunk, chunk_data: &ChunkData) -> Result<()> {
        let operator = self.backup_config.build_operator()?;
        let file_path =
            split_hash_as_path(self.backup_config.output_path.clone(), chunk.hash.clone());

        Ok(operator.write(file_path.to_str().unwrap(), chunk_data.data.clone())?)
    }
}
