use std::{collections::HashMap, fs::File, path::Path};

use fastcdc::v2020::{ChunkData, StreamCDC};
use log::{error, info};
use walkdir::WalkDir;

use crate::backup::{
    backup_metadata::BackupMetadata,
    chunk_table::Chunk,
    file_metadata::{FileChunk, FileMetadata},
};

pub struct BackupConfig<'a> {
    pub average_size: u32,
    pub input_path: &'a Path,
    pub output_path: &'a Path,
}

pub struct Backup<'a> {
    backup_config: BackupConfig<'a>,
}

impl Backup<'_> {
    pub fn new(backup_config: BackupConfig) -> Backup {
        Backup { backup_config }
    }

    pub fn backup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let old_backup_metadata_result =
            BackupMetadata::deserialize(self.backup_config.output_path);
        let mut backup_metadata = BackupMetadata::new();

        for dir_entry_result in WalkDir::new(self.backup_config.input_path) {
            let entry = dir_entry_result.expect("cannot walk the path");
            let current_path = entry
                .path()
                .to_str()
                .expect("cannot unwrap path")
                .to_string();
            info!("{}", entry.path().display());

            if entry.path().is_dir() {
                continue;
            }

            backup_metadata.file_metadatas.insert(
                current_path.clone(),
                FileMetadata {
                    root_path: current_path.clone(),
                    chunks: HashMap::new(),
                },
            );

            let file = File::open(&current_path).expect("cannot open file!");
            let min_size = self.backup_config.average_size / 4;
            let max_size = self.backup_config.average_size * 4;

            let chunker = StreamCDC::new(file, min_size, self.backup_config.average_size, max_size);

            for result in chunker {
                let chunk_data: ChunkData = result.expect("failed to read chunk");
                let chunk: Chunk = Chunk::from(&chunk_data);

                if !backup_metadata
                    .chunk_table
                    .chunk_map
                    .contains_key(&chunk.hash)
                {
                    // chunk.save(&chunk_data.data, output_path);
                    backup_metadata
                        .chunk_table
                        .chunk_map
                        .insert(chunk.hash.clone(), chunk.clone());
                }

                backup_metadata
                    .file_metadatas
                    .get_mut(&current_path)
                    .expect("value must be there")
                    .chunks
                    .insert(
                        chunk.hash.clone(),
                        FileChunk {
                            hash: chunk.hash.clone(),
                            offset: chunk_data.offset.clone(),
                            length: chunk_data.length.clone(),
                        },
                    );
            }
        }

        if old_backup_metadata_result.is_ok() {
            for (file_path, file_metadata) in &backup_metadata.file_metadatas {
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

        backup_metadata.serialize(self.backup_config.output_path);

        info!(
            "Stored: {} MB",
            backup_metadata
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
