use std::{
    fs::{self, File},
    path::Path,
};

use anyhow::Result;
use fastcdc::v2020::{ChunkData, StreamCDC};
use indicatif::ProgressBar;
use log::{debug, error, info};
use walkdir::WalkDir;

use crate::backup::{
    backup_metadata::BackupMetadata,
    file_metadata::{FileChunk, FileMetadata},
    symlink::Symlink,
};

use super::backup_config::BackupConfig;

pub struct Backup {
    backup_config: BackupConfig,
    backup_metadata: BackupMetadata,
}

impl Backup {
    pub fn new(backup_config: BackupConfig) -> Backup {
        Backup {
            backup_config,
            backup_metadata: BackupMetadata::new(),
        }
    }

    pub fn chunk_file(&mut self, file_path: &Path) -> Result<FileMetadata> {
        let file = File::open(&file_path).expect("cannot open file!");
        let mut file_metadata = FileMetadata::new(file_path.display().to_string());
        let chunker = StreamCDC::new(
            file,
            self.backup_config.min_size(),
            self.backup_config.average_size,
            self.backup_config.max_size(),
        );

        for result in chunker {
            let chunk_data: ChunkData = result.expect("failed to read chunk");
            let chunk = self
                .backup_metadata
                .chunk_table
                .store_chunk_data(&chunk_data)?;

            file_metadata.chunks.insert(
                chunk.hash.clone(),
                FileChunk {
                    hash: chunk.hash.clone(),
                    offset: chunk_data.offset.clone(),
                    length: chunk_data.length.clone(),
                },
            );
        }
        return Ok(file_metadata);
    }

    pub fn walk(&mut self) -> Result<()> {
        for dir_entry_result in WalkDir::new(&self.backup_config.input_path) {
            let entry = dir_entry_result?;

            if entry.path().is_dir() {
                debug!("{}", entry.path().display());
                continue;
            }

            // TODO: how to backup and restore symlinks? wtf?
            if entry.path().is_symlink() {
                self.backup_metadata.symlinks.push(Symlink::new(
                    entry.path().display().to_string(),
                    fs::read_link(entry.path())?.display().to_string(),
                ));
                continue;
            }

            let file_metadata = self.chunk_file(entry.path())?;
            self.backup_metadata
                .file_metadatas
                .insert(file_metadata.path.clone(), file_metadata);
        }
        Ok(())
    }

    pub fn backup(&mut self) -> Result<()> {
        let old_backup_metadata_result =
            BackupMetadata::deserialize(Path::new(&self.backup_config.output_path));

        self.walk()?;

        if old_backup_metadata_result.is_ok() {
            for (file_path, file_metadata) in self.backup_metadata.file_metadatas.iter() {
                let old_backup_metadata = old_backup_metadata_result.as_ref().unwrap();

                if let Some(old_file_metadata) = old_backup_metadata.file_metadatas.get(file_path) {
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
        } else {
            error!("{:?}", old_backup_metadata_result.err())
        }

        self.backup_metadata
            .serialize(Path::new(&self.backup_config.output_path));

        info!(
            "Stored: {} MB",
            self.backup_metadata
                .chunk_table
                .chunk_map
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
