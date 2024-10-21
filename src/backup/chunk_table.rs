use core::str;
use std::collections::HashMap;

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

impl ChunkTable {
    pub fn new() -> ChunkTable {
        ChunkTable {
            chunk_map: HashMap::new(),
        }
    }
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
}

impl From<&ChunkData> for Chunk {
    fn from(chunk_data: &ChunkData) -> Self {
        Chunk {
            hash: blake3::hash(&chunk_data.data).to_hex().to_string(),
            length: chunk_data.length,
        }
    }
}
