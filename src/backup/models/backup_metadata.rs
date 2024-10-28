use crate::backup::models::file_metadata::FileMetadata;
use crate::backup::services::chunk_storage::ChunkMap;
use crate::backup::symlink::Symlink;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Write},
    path::Path,
};

pub type FileMetadataMap = HashMap<String, FileMetadata>;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub chunk_map: ChunkMap,
    // file_path -> FileMetadata
    pub file_metadata_map: FileMetadataMap,
    pub symlinks: Vec<Symlink>,
}

impl BackupMetadata {
    const BACKUP_METADATA_FILE: &'static str = "metadata.json";

    pub fn new() -> BackupMetadata {
        BackupMetadata {
            chunk_map: Default::default(),
            file_metadata_map: Default::default(),
            symlinks: Default::default(),
        }
    }

    pub fn new_with_data(
        chunk_map: ChunkMap,
        file_metadata_map: FileMetadataMap,
        symlinks: Vec<Symlink>,
    ) -> BackupMetadata {
        BackupMetadata {
            chunk_map,
            file_metadata_map,
            symlinks,
        }
    }

    pub fn serialize(&self, directory_path: &Path) -> Result<()> {
        fs::create_dir_all(directory_path)?;
        let mut file = fs::File::create(directory_path.join(Self::BACKUP_METADATA_FILE))?;
        let json_data = serde_json::to_string(&self)?;

        Ok(file.write_all(json_data.as_bytes())?)
    }

    pub fn deserialize(directory_path: &Path) -> Result<BackupMetadata> {
        let path = directory_path.join(Self::BACKUP_METADATA_FILE);

        if !path.exists() {
            return Ok(BackupMetadata::new());
        }

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        Ok(serde_json::from_reader::<BufReader<File>, BackupMetadata>(
            reader,
        )?)
    }

    pub fn insert_symlink(&mut self, symlink: Symlink) {
        self.symlinks.push(symlink);
    }
}
