use crate::backup::models::backup_config::BackupConfig;
use crate::backup::models::chunk::Chunk;
use crate::backup::models::file_chunk::FileChunk;
use crate::backup::models::file_metadata::FileMetadata;
use crate::backup::services::chunk_reader_writer::ChunkReaderWriter;
use crate::backup::services::chunk_storage::ChunkStorage;
use anyhow::Result;
use fastcdc::v2020::{ChunkData, StreamCDC};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct FileChunker {
    backup_config: Arc<BackupConfig>,
    chunk_reader_writer: Arc<ChunkReaderWriter>,
    chunk_storage: Arc<Mutex<Box<dyn ChunkStorage + Send + Sync + 'static>>>,
}

impl FileChunker {
    pub fn new(
        backup_config: Arc<BackupConfig>,
        chunk_reader_writer: Arc<ChunkReaderWriter>,
        chunk_storage: Arc<Mutex<Box<dyn ChunkStorage + Send + Sync + 'static>>>,
    ) -> FileChunker {
        FileChunker {
            chunk_reader_writer,
            backup_config,
            chunk_storage,
        }
    }

    pub fn chunk_file(&mut self, file_path: &Path) -> Result<FileMetadata> {
        let file = File::open(file_path).expect("cannot open file!");
        let mut file_metadata = FileMetadata::new(file_path.display().to_string());
        let chunker = StreamCDC::new(
            file,
            self.backup_config.min_size(),
            self.backup_config.average_size,
            self.backup_config.max_size(),
        );

        for chunk_data_result in chunker {
            let chunk_data: ChunkData = chunk_data_result?;
            let chunk = Chunk::from(&chunk_data);

            let mut unlocked_chunk_storage = self.chunk_storage.lock().unwrap();
            if !unlocked_chunk_storage.chunk_exists(&chunk.hash) {
                self.chunk_reader_writer.write_chunk(
                    &chunk.hash,
                    chunk_data.data,
                    self.backup_config.output_path.as_ref(),
                )?;
                unlocked_chunk_storage.add_chunk(chunk.clone())?;
            }

            file_metadata.chunks.insert(
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
