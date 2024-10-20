use core::str;
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use fastcdc::v2020::ChunkData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub hash: String,
    pub length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkTable {
    // chunk hash -> Chunk
    pub chunk_map: HashMap<String, Chunk>,
}

impl Chunk {
    pub fn split_hash(&self) -> Vec<String> {
        return self
            .hash
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|chunk| chunk.iter().collect())
            .collect();
    }

    pub fn save(&self, data: &[u8], output_path: &Path) {
        let mut source_path = PathBuf::from(output_path);
        for item in self.split_hash() {
            source_path.push(item);
        }

        if let Some(parent) = source_path.parent() {
            // Create the directories "as/df/gh"
            fs::create_dir_all(parent).expect("cannot create directories");

            let mut file = fs::File::create(source_path).expect("cannot write the file");
            file.write_all(data).expect("cannot write file");
        }
    }
}

impl From<&ChunkData> for Chunk {
    fn from(value: &ChunkData) -> Self {
        Chunk {
            hash: blake3::hash(&value.data).to_hex().to_string(),
            length: value.length,
        }
    }
}
