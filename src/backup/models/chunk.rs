use core::str;
use fastcdc::v2020::ChunkData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub hash: String,
    pub length: usize,
}

impl From<&ChunkData> for Chunk {
    fn from(chunk_data: &ChunkData) -> Self {
        Chunk {
            hash: blake3::hash(&chunk_data.data).to_hex().to_string(),
            length: chunk_data.length,
        }
    }
}
