use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::Result;
use fastcdc::v2020::{ChunkData, StreamCDC};
use itertools::Itertools;
use log::{debug, info};
use walkdir::WalkDir;

use crate::backup::{
    backup_metadata::BackupMetadata,
    file_metadata::{FileChunk, FileMetadata},
    symlink::Symlink,
};

use super::{
    backup_config::BackupConfig, backup_hash::split_hash_as_path, chunk_table::Chunk,
    chunk_writer::ChunkWriter,
};

pub struct BackupService<'a> {
    backup_config: &'a BackupConfig,
    chunk_writer: &'a ChunkWriter<'a>,
    backup_metadata: &'a mut BackupMetadata,
}

impl BackupService<'_> {
    pub fn new<'a>(
        backup_config: &'a BackupConfig,
        chunk_writer: &'a ChunkWriter,
        backup_metadata: &'a mut BackupMetadata,
    ) -> BackupService<'a> {
        BackupService {
            backup_config,
            backup_metadata,
            chunk_writer,
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

        for chunk_data_result in chunker {
            let chunk_data: ChunkData = chunk_data_result?;
            let chunk = Chunk::from(&chunk_data);

            if !self
                .backup_metadata
                .chunk_table
                .chunk_map
                .contains_key(&chunk.hash)
            {
                self.chunk_writer.write(&chunk, &chunk_data)?;
                self.backup_metadata
                    .chunk_table
                    .chunk_map
                    .insert(chunk.hash.clone(), chunk.clone());
            }

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
        self.walk()?;

        let old_backup_metadata =
            BackupMetadata::deserialize(Path::new(&self.backup_config.output_path))?;

        for (file_path, file_metadata) in self.backup_metadata.file_metadatas.iter() {
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

    pub fn restore(&self) -> Result<()> {
        let operator = self.backup_config.build_operator()?;
        for (output_file_path, file_metadata) in self.backup_metadata.file_metadatas.iter() {
            debug!("Restoring: {}", output_file_path);
            let moved_output_filepath =
                PathBuf::from(&self.backup_config.output_path).join(output_file_path);

            let mut writer = operator.writer(&moved_output_filepath.display().to_string())?;
            for (hash, _) in file_metadata
                .chunks
                .iter()
                .sorted_by(|(_, a), (_, b)| Ord::cmp(&a.offset, &b.offset))
            {
                let chunk_filepath =
                    split_hash_as_path(self.backup_config.input_path.clone(), hash.to_string());
                let chunk_data = fs::read(chunk_filepath)?;
                writer.write(chunk_data)?;
            }
            writer.close()?;
        }
        Ok(())
    }
}
