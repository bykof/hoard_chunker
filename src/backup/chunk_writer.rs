use std::path::PathBuf;

use anyhow::Result;
use fastcdc::v2020::ChunkData;

use super::{backup_config::BackupConfig, chunk_table::Chunk};

pub struct ChunkWriter<'a> {
    backup_config: &'a BackupConfig,
}

impl ChunkWriter<'_> {
    pub fn new<'a>(backup_config: &'a BackupConfig) -> ChunkWriter<'a> {
        ChunkWriter { backup_config }
    }
    pub fn write(&self, chunk: &Chunk, chunk_data: &ChunkData) -> Result<()> {
        let operator = self.backup_config.build_operator()?;
        let mut file_path = PathBuf::from(&self.backup_config.output_path);
        for path_step in &chunk.split_hash() {
            file_path.push(path_step);
        }

        Ok(operator.write(file_path.to_str().unwrap(), chunk_data.data.clone())?)
    }
}
