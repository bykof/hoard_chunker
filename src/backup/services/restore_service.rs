use crate::backup::models::backup_config::BackupConfig;
use crate::backup::models::backup_metadata::BackupMetadata;
use crate::backup::services::chunk_reader_writer::ChunkReaderWriter;
use crate::backup::services::chunk_storage::ChunkStorage;
use itertools::Itertools;
use log::debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct RestoreService {
    backup_config: Arc<BackupConfig>,

    chunk_storage: Arc<Box<dyn ChunkStorage + Send + Sync>>,
    chunk_reader_writer: Arc<ChunkReaderWriter>,
}

impl RestoreService {
    pub fn new(
        backup_config: Arc<BackupConfig>,
        chunk_storage: Arc<Box<dyn ChunkStorage + Send + Sync>>,
        chunk_reader_writer: Arc<ChunkReaderWriter>,
    ) -> RestoreService {
        RestoreService {
            backup_config,
            chunk_storage,
            chunk_reader_writer,
        }
    }

    pub fn restore(&mut self) -> anyhow::Result<()> {
        let backup_metadata =
            BackupMetadata::deserialize(Path::new(&self.backup_config.input_path))?;
        let file_metadata_map = backup_metadata.file_metadata_map.clone();
        self.chunk_storage
            .load_chunk_map(backup_metadata.chunk_map.clone())?;

        for (output_file_path, file_metadata) in file_metadata_map.iter() {
            let output_filepath = output_file_path
                .strip_prefix("/")
                .unwrap_or(output_file_path);

            let moved_output_filepath =
                PathBuf::from(&self.backup_config.output_path).join(output_filepath);

            debug!("Restoring: {}", moved_output_filepath.display());

            let mut writer = self
                .chunk_reader_writer
                .build_operator()?
                .writer(moved_output_filepath.to_str().unwrap())?;

            for (hash, _) in file_metadata
                .chunks
                .iter()
                .sorted_by(|(_, a), (_, b)| Ord::cmp(&a.offset, &b.offset))
            {
                let chunk_data = self.chunk_storage.load_chunk(hash)?;
                writer.write(chunk_data)?
            }
            writer.close()?;
        }
        Ok(())
    }
}
