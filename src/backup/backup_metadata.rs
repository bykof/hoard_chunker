use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufReader, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use super::{chunk_table::ChunkTable, file_metadata::FileMetadata};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub chunk_table: ChunkTable,
    // file_path -> FileMetadata
    pub file_metadatas: HashMap<String, FileMetadata>,
}

impl BackupMetadata {
    const BACKUP_METADATA_FILE: &str = "metadata.json";

    pub fn new() -> BackupMetadata {
        BackupMetadata {
            chunk_table: ChunkTable {
                chunk_map: HashMap::new(),
            },
            file_metadatas: HashMap::new(),
        }
    }

    pub fn serialize(&self, directory_path: &Path) {
        fs::create_dir_all(directory_path).expect("cannot create directories for metadata file");
        let mut file = fs::File::create(directory_path.join(Self::BACKUP_METADATA_FILE))
            .expect("cannot create metadata file");
        let json_data = serde_json::to_string(&self).expect("cannot serialize");
        file.write_all(json_data.as_bytes())
            .expect("cannot write file");
    }

    pub fn deserialize(directory_path: &Path) -> Result<BackupMetadata, Box<dyn Error>> {
        let file = fs::File::open(directory_path.join(Self::BACKUP_METADATA_FILE))?;
        let reader = BufReader::new(file);
        Ok(
            serde_json::from_reader::<BufReader<File>, BackupMetadata>(reader)
                .expect("cannot deserialize"),
        )
    }
}
