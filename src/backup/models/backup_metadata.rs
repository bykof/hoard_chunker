use crate::backup::models::file_metadata::FileMetadata;
use crate::backup::models::symlink::Symlink;
use crate::backup::services::chunk_storage::ChunkMap;
use anyhow::{Error, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Write},
    path::Path,
};

pub type FileMetadataMap = HashMap<String, FileMetadata>;

pub enum SerializationType {
    JSON,
    MessagePack,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub chunk_map: ChunkMap,
    // file_path -> FileMetadata
    pub file_metadata_map: FileMetadataMap,
    pub symlinks: Vec<Symlink>,
}

impl BackupMetadata {
    const BACKUP_METADATA_FILE: &'static str = "metadata";

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

    pub fn serialize(
        &self,
        directory_path: &Path,
        serialization_type: SerializationType,
    ) -> Result<()> {
        fs::create_dir_all(directory_path)?;
        let mut file = fs::File::create(directory_path.join(Self::BACKUP_METADATA_FILE))?;
        let bytes: Vec<u8>;

        match serialization_type {
            SerializationType::JSON => {
                let json_data = serde_json::to_vec(&self)?;
                bytes = json_data.clone()
            }
            SerializationType::MessagePack => {
                let message_pack_data = rmp_serde::to_vec(&self)?;
                bytes = message_pack_data.clone()
            }
        }

        Ok(file.write_all(bytes.as_slice())?)
    }

    pub fn deserialize(directory_path: &Path) -> Result<BackupMetadata> {
        let path = directory_path.join(Self::BACKUP_METADATA_FILE);

        if !path.exists() {
            return Ok(BackupMetadata::new());
        }

        debug!("Trying deserialize as json");
        let json_result = serde_json::from_reader::<BufReader<File>, BackupMetadata>(
            BufReader::new(File::open(path.clone())?),
        );
        if json_result.is_ok() {
            return Ok(json_result?);
        }

        debug!("Trying deserialize as messagepack");
        let msgpck_result = rmp_serde::from_read::<BufReader<File>, BackupMetadata>(
            BufReader::new(File::open(path.clone())?),
        );
        if msgpck_result.is_ok() {
            return Ok(msgpck_result?);
        }

        Err(Error::msg(
            "Could not deserialize backup metadata".to_string(),
        ))
    }

    pub fn insert_symlink(&mut self, symlink: Symlink) {
        self.symlinks.push(symlink);
    }
}
