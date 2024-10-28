use anyhow::Result;
use itertools::Itertools;
use log::{debug, info};
use std::sync::{Arc, Mutex};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::backup::models::backup_config::BackupConfig;
use crate::backup::models::backup_metadata::{BackupMetadata, FileMetadataMap};
use crate::backup::models::chunk::Chunk;
use crate::backup::services::chunk_reader_writer::ChunkReaderWriter;
use crate::backup::services::chunk_storage::ChunkStorage;
use crate::backup::services::file_chunker::FileChunker;
use crate::backup::symlink::Symlink;

pub struct BackupService {
    backup_config: Arc<BackupConfig>,
    file_chunker: Arc<Mutex<FileChunker>>,
    chunk_reader_writer: Arc<ChunkReaderWriter>,
    chunk_storage: Arc<Mutex<Box<dyn ChunkStorage + Send + Sync + 'static>>>,

    symlinks: Vec<Symlink>,
    // filepath -> FileMetadata
    file_metadata_map: FileMetadataMap,
}

impl BackupService {
    pub fn new(
        backup_config: Arc<BackupConfig>,
        file_chunker: Arc<Mutex<FileChunker>>,
        chunk_reader_writer: Arc<ChunkReaderWriter>,
        chunk_storage: Arc<Mutex<Box<dyn ChunkStorage + Send + Sync + 'static>>>,
    ) -> BackupService {
        BackupService {
            backup_config,
            file_chunker,
            chunk_reader_writer,
            chunk_storage,
            symlinks: Default::default(),
            file_metadata_map: Default::default(),
        }
    }

    pub fn walk(&mut self) -> Result<()> {
        for dir_entry_result in WalkDir::new(&self.backup_config.input_path) {
            let dir_entry = dir_entry_result?;
            if dir_entry.path().is_dir() {
                // currently directories are useless for us
                continue;
            }

            // TODO: how to backup and restore symlinks? wtf?
            if dir_entry.path().is_symlink() {
                self.symlinks.push(Symlink::new(
                    dir_entry.path().display().to_string(),
                    fs::read_link(dir_entry.path())?.display().to_string(),
                ));
            }

            let file_metadata = self
                .file_chunker
                .lock()
                .unwrap()
                .chunk_file(dir_entry.path())?;

            self.file_metadata_map.insert(
                dir_entry.path().display().to_string(),
                file_metadata.clone(),
            );

            for (hash, file_chunk) in file_metadata.chunks {
                self.chunk_storage.lock().unwrap().add_chunk(Chunk {
                    hash: hash.clone(),
                    length: file_chunk.length,
                })?
            }
        }
        Ok(())
    }

    pub fn backup(&mut self) -> Result<()> {
        info!("Walking directory: {}...", self.backup_config.input_path);
        self.walk()?;
        info!("Done walking directory");

        let old_backup_metadata =
            BackupMetadata::deserialize(Path::new(&self.backup_config.output_path))?;

        for (file_path, file_metadata) in self.file_metadata_map.iter() {
            if let Some(old_file_metadata) = old_backup_metadata.file_metadata_map.get(file_path) {
                if file_metadata.fingerprint() != old_file_metadata.fingerprint() {
                    info!("Files: {} are not identical", file_path);

                    let chunks: Vec<_> = file_metadata
                        .chunks
                        .keys()
                        .filter(|key| !old_file_metadata.chunks.contains_key(*key))
                        .collect();

                    info!("New chunks: {:?}", chunks);
                } else {
                    // info!("File: {} are identical", file_path);
                }
            }
        }

        info!(
            "Writing backup metadata to: {}...",
            self.backup_config.output_path
        );

        let backup_metadata = BackupMetadata::new_with_data(
            self.chunk_storage.lock().unwrap().chunk_map()?,
            self.file_metadata_map.clone(),
            self.symlinks.clone(),
        );
        backup_metadata.serialize(Path::new(&self.backup_config.output_path))?;

        info!(
            "Done writing backup metadata to: {}",
            self.backup_config.output_path
        );
        info!(
            "Stored: {} MB",
            self.chunk_storage
                .lock()
                .unwrap()
                .chunk_map()?
                .values()
                .map(|value| value.length)
                .reduce(|a, b| a + b)
                .unwrap()
                / 1024
                / 1024
        );

        Ok(())
    }

    pub fn restore(&mut self) -> Result<()> {
        let backup_metadata =
            BackupMetadata::deserialize(Path::new(&self.backup_config.input_path))?;
        self.symlinks = backup_metadata.symlinks.clone();
        self.file_metadata_map = backup_metadata.file_metadata_map.clone();
        self.chunk_storage
            .lock()
            .unwrap()
            .load_chunk_map(backup_metadata.chunk_map.clone())?;

        for (output_file_path, file_metadata) in self.file_metadata_map.iter() {
            debug!("Restoring: {}", output_file_path);
            let moved_output_filepath =
                PathBuf::from(&self.backup_config.output_path).join(output_file_path);
            let mut writer = self
                .chunk_reader_writer
                .build_operator()?
                .writer(moved_output_filepath.to_str().unwrap())?;

            for (hash, _) in file_metadata
                .chunks
                .iter()
                .sorted_by(|(_, a), (_, b)| Ord::cmp(&a.offset, &b.offset))
            {
                let chunk_data = self
                    .chunk_reader_writer
                    .read_chunk(hash, self.backup_config.input_path.as_ref())?;
                writer.write(chunk_data)?
            }
        }
        Ok(())
    }
}
