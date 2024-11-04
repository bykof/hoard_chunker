use crate::backup::models::backup_config::BackupConfig;
use crate::backup::models::chunk::Chunk;
use crate::backup::models::file_chunk::FileChunk;
use crate::backup::models::file_metadata::FileMetadata;
use crate::backup::services::chunk_storage::ChunkStorage;
use anyhow::Result;
use fastcdc::v2020::{ChunkData, StreamCDC};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

pub struct FileChunker {
    backup_config: Arc<BackupConfig>,
    chunk_storage: Arc<Box<dyn ChunkStorage + Send + Sync>>,
}

impl FileChunker {
    pub fn new(
        backup_config: Arc<BackupConfig>,
        chunk_storage: Arc<Box<dyn ChunkStorage + Send + Sync>>,
    ) -> FileChunker {
        FileChunker {
            backup_config,
            chunk_storage,
        }
    }

    pub fn chunk_file(&self, file_path: &Path) -> Result<FileMetadata> {
        let file = File::open(file_path).expect("cannot open file!");
        let mut file_metadata = FileMetadata::new(file_path.display().to_string());
        let chunker = StreamCDC::new(
            &file,
            self.backup_config.min_size(),
            self.backup_config.average_size,
            self.backup_config.max_size(),
        );

        for chunk_data_result in chunker.into_iter() {
            let chunk_data: ChunkData = chunk_data_result?;
            let chunk = Chunk::from(&chunk_data);

            if !self.chunk_storage.chunk_exists(&chunk.hash) {
                self.chunk_storage
                    .store_chunk(&chunk.hash, &chunk_data.data)
                    .expect("cannot write chunk!");
                self.chunk_storage.add_chunk(chunk.clone())?;
            }

            file_metadata.add_chunk(
                chunk.hash.to_string(),
                FileChunk {
                    hash: chunk.hash.to_string(),
                    offset: chunk_data.offset.clone(),
                    length: chunk_data.length.clone(),
                },
            );
        }

        Ok(file_metadata)
    }
}
