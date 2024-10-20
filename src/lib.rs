use core::str;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use fastcdc::v2020::ChunkData;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct FileChunk {
    pub hash: u64,
    pub offset: u64,
    pub length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub hash: u64,
    pub length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub root_path: String,
    // hash -> FileChunk
    pub chunks: HashMap<u64, FileChunk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkTable {
    // hash -> Chunk
    pub chunk_map: HashMap<u64, Chunk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub chunk_table: ChunkTable,
    // file_path -> FileMetadata
    pub file_metadatas: HashMap<String, FileMetadata>,
}

impl FileMetadata {
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.chunks
            .iter()
            .sorted_by(|(_, file_chunk_a), (_, file_chunk_b)| {
                Ord::cmp(&file_chunk_a.offset, &file_chunk_b.offset)
            })
            .map(|(hash, _)| hash)
            .collect::<Vec<&u64>>()
            .hash(&mut hasher);
        hasher.finish()
    }
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

pub fn split_hash(hash: u64) -> Vec<String> {
    return format!("{:X}", hash)
        .as_bytes()
        .chunks(2)
        .map(str::from_utf8)
        .map(Result::unwrap)
        .map(String::from)
        .collect();
}

pub fn store_chunk(chunk: &ChunkData, output_path: &Path) {
    let mut source_path = PathBuf::from(output_path);
    for item in split_hash(chunk.hash) {
        source_path.push(item);
    }

    if let Some(parent) = source_path.parent() {
        // Create the directories "as/df/gh"
        fs::create_dir_all(parent).expect("cannot create directories");

        let mut file = fs::File::create(source_path).expect("cannot write the file");
        file.write_all(chunk.data.as_slice())
            .expect("cannot write file");
    }
}
