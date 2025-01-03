use anyhow::Result;
use log::{debug, info};
use std::sync::Arc;
use std::time::Instant;
use std::{
    fs::{self},
    path::Path,
};
use walkdir::WalkDir;

use crate::backup::models::backup_config::BackupConfig;
use crate::backup::models::backup_metadata::{BackupMetadata, FileMetadataMap, SerializationType};
use crate::backup::models::chunk::Chunk;
use crate::backup::models::symlink::Symlink;
use crate::backup::services::chunk_storage::ChunkStorage;
use crate::backup::services::file_chunker::FileChunker;

pub struct BackupService {
    backup_config: Arc<BackupConfig>,
    file_chunker: Arc<FileChunker>,
    chunk_storage: Arc<Box<dyn ChunkStorage + Send + Sync>>,

    symlinks: Vec<Symlink>,
    // filepath -> FileMetadata
    file_metadata_map: FileMetadataMap,
}

impl BackupService {
    pub fn new(
        backup_config: Arc<BackupConfig>,
        file_chunker: Arc<FileChunker>,
        chunk_storage: Arc<Box<dyn ChunkStorage + Send + Sync>>,
    ) -> BackupService {
        BackupService {
            backup_config,
            file_chunker,
            chunk_storage,
            symlinks: Default::default(),
            file_metadata_map: Default::default(),
        }
    }

    pub fn walk(&mut self) -> Result<()> {
        info!("Walking directory: {}...", self.backup_config.input_path,);
        let start = Instant::now();

        for dir_entry_result in WalkDir::new(&self.backup_config.input_path).into_iter() {
            let dir_entry = dir_entry_result?;
            if dir_entry.path().is_dir() {
                // currently directories are useless for us
                debug!("skipping directory: {}", dir_entry.path().display());
                continue;
            }

            // TODO: how to backup and restore symlinks? wtf?
            if dir_entry.path().is_symlink() {
                self.symlinks.push(Symlink::new(
                    dir_entry.path().display().to_string(),
                    fs::read_link(dir_entry.path())?.display().to_string(),
                ));
                continue;
            }

            let file_metadata = self.file_chunker.chunk_file(dir_entry.path())?;
            if self.file_metadata_map.contains_key(&file_metadata.key())
                && file_metadata.fingerprint()
                    != self
                        .file_metadata_map
                        .get(&file_metadata.key())
                        .unwrap()
                        .fingerprint()
            {
                info!("File {} changed!", file_metadata.key());
                self.file_metadata_map
                    .insert(file_metadata.key(), file_metadata.clone());
            } else {
                self.file_metadata_map
                    .insert(file_metadata.key(), file_metadata.clone());
            }

            for (hash, file_chunk) in file_metadata.chunks {
                if !self.chunk_storage.chunk_exists(&hash) {
                    self.chunk_storage.add_chunk(Chunk {
                        hash: hash.clone(),
                        length: file_chunk.length,
                    })?
                }
            }
        }

        info!("Done walking - took {:?}", start.elapsed());
        Ok(())
    }

    pub fn backup(&mut self) -> Result<()> {
        let old_backup_metadata =
            BackupMetadata::deserialize(Path::new(&self.backup_config.output_path))?;
        self.symlinks = old_backup_metadata.symlinks;
        self.file_metadata_map = old_backup_metadata.file_metadata_map;
        self.chunk_storage
            .load_chunk_map(old_backup_metadata.chunk_map)?;
        self.walk()?;

        // let old_backup_metadata =
        //     BackupMetadata::deserialize(Path::new(&self.backup_config.output_path))?;

        // for (file_path, file_metadata) in self.file_metadata_map.iter() {
        //     if let Some(old_file_metadata) = old_backup_metadata.file_metadata_map.get(file_path) {
        //         if file_metadata.fingerprint() != old_file_metadata.fingerprint() {
        //             info!("Files: {} are not identical", file_path);
        //
        //             let chunks: Vec<_> = file_metadata
        //                 .chunks
        //                 .keys()
        //                 .filter(|key| !old_file_metadata.chunks.contains_key(*key))
        //                 .collect();
        //
        //             info!("New chunks: {:?}", chunks);
        //         } else {
        //             // info!("File: {} are identical", file_path);
        //         }
        //     }
        // }

        info!(
            "Writing backup metadata to: {}...",
            self.backup_config.output_path
        );

        let backup_metadata = BackupMetadata::new_with_data(
            self.chunk_storage.chunk_map()?,
            self.file_metadata_map.clone(),
            self.symlinks.clone().clone(),
        );
        backup_metadata.serialize(
            Path::new(&self.backup_config.output_path),
            SerializationType::MessagePack,
        )?;

        info!(
            "Done writing backup metadata to: {}",
            self.backup_config.output_path
        );
        info!(
            "Read: {} MB",
            self.chunk_storage
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
}
